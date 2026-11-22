//! SFTerm 核心引擎：SSH 连接与 SFTP 操作。
//!
//! 设计要点：
//! - 不依赖 Tauri，可以单独编译、单独测试（见 `src/bin/check.rs`）。
//! - 每个 SSH 会话跑在独立线程里，`ssh2` 的阻塞调用不会卡住 UI 线程。
//! - SFTP 子系统只在会话建立时打开一次（在低配服务器上，打开一次要几百毫秒）。
//! - 会话之间互不共享锁，命令通过 channel 投递。

use serde::{Deserialize, Serialize};
use ssh2::{HashType, MethodType, Session};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const IO_TIMEOUT_MS: u32 = 15_000;
const KEEPALIVE_SECS: u32 = 60;

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectParams {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
}

fn default_port() -> u16 {
    22
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthMethod {
    Password {
        password: String,
    },
    Key {
        path: String,
        #[serde(default)]
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectInfo {
    pub session_id: String,
    pub banner: Option<String>,
    pub host_key_type: String,
    pub fingerprint_sha256: String,
    pub kex: Option<String>,
    pub cipher: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub perm: Option<u32>,
    pub mtime: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TransferProgress {
    pub done: u64,
    pub total: u64,
}

enum Request {
    ListDir {
        path: String,
        reply: Sender<Result<Vec<RemoteEntry>, String>>,
    },
    RealPath {
        reply: Sender<Result<String, String>>,
    },
    Upload {
        local: String,
        remote: String,
        overwrite: bool,
        progress: Sender<TransferProgress>,
        reply: Sender<Result<(), String>>,
    },
    Download {
        remote: String,
        local: String,
        overwrite: bool,
        progress: Sender<TransferProgress>,
        reply: Sender<Result<(), String>>,
    },
    Mkdir {
        path: String,
        reply: Sender<Result<(), String>>,
    },
    Rename {
        from: String,
        to: String,
        reply: Sender<Result<(), String>>,
    },
    Remove {
        path: String,
        is_dir: bool,
        reply: Sender<Result<(), String>>,
    },
}

struct SessionHandle {
    tx: Sender<Request>,
}

/// 会话管理器。跨线程共享，内部只锁很短的时间。
#[derive(Default)]
pub struct Manager {
    sessions: Mutex<HashMap<String, SessionHandle>>,
    next_id: AtomicU64,
}

impl Manager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 建立连接、认证、打开 SFTP，并启动会话线程。
    pub fn connect(&self, params: ConnectParams) -> Result<ConnectInfo, String> {
        let id = format!("s{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let (handle, info) = open_session(params, id)?;
        self.sessions
            .lock()
            .map_err(|_| "会话表被污染".to_string())?
            .insert(info.session_id.clone(), handle);
        Ok(info)
    }

    /// 列出远端目录。阻塞，调用方应放在工作线程里。
    pub fn list_dir(&self, session_id: &str, path: &str) -> Result<Vec<RemoteEntry>, String> {
        let tx = {
            let sessions = self
                .sessions
                .lock()
                .map_err(|_| "会话表被污染".to_string())?;
            sessions
                .get(session_id)
                .ok_or_else(|| format!("会话不存在：{session_id}"))?
                .tx
                .clone()
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        tx.send(Request::ListDir {
            path: path.to_string(),
            reply: reply_tx,
        })
        .map_err(|_| "会话已关闭".to_string())?;
        reply_rx.recv().map_err(|_| "会话已关闭".to_string())?
    }

    /// 远端会话的初始目录（通常是登录用户的主目录）。
    pub fn remote_home(&self, session_id: &str) -> Result<String, String> {
        let tx = {
            let sessions = self
                .sessions
                .lock()
                .map_err(|_| "会话表被污染".to_string())?;
            sessions
                .get(session_id)
                .ok_or_else(|| format!("会话不存在：{session_id}"))?
                .tx
                .clone()
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        tx.send(Request::RealPath { reply: reply_tx })
            .map_err(|_| "会话已关闭".to_string())?;
        reply_rx.recv().map_err(|_| "会话已关闭".to_string())?
    }

    /// 上传本地文件到远端。阻塞，进度通过 `progress` 回传。
    pub fn upload(
        &self,
        session_id: &str,
        local: String,
        remote: String,
        overwrite: bool,
        progress: Sender<TransferProgress>,
    ) -> Result<(), String> {
        self.transfer(session_id, |reply| Request::Upload {
            local,
            remote,
            overwrite,
            progress,
            reply,
        })
    }

    /// 从远端下载文件到本地。阻塞，进度通过 `progress` 回传。
    pub fn download(
        &self,
        session_id: &str,
        remote: String,
        local: String,
        overwrite: bool,
        progress: Sender<TransferProgress>,
    ) -> Result<(), String> {
        self.transfer(session_id, |reply| Request::Download {
            remote,
            local,
            overwrite,
            progress,
            reply,
        })
    }

    /// 新建远端目录。
    pub fn mkdir(&self, session_id: &str, path: String) -> Result<(), String> {
        self.transfer(session_id, |reply| Request::Mkdir { path, reply })
    }

    /// 重命名 / 移动远端文件。
    pub fn rename(&self, session_id: &str, from: String, to: String) -> Result<(), String> {
        self.transfer(session_id, |reply| Request::Rename { from, to, reply })
    }

    /// 删除远端文件或目录（目录递归删除）。
    pub fn remove(&self, session_id: &str, path: String, is_dir: bool) -> Result<(), String> {
        self.transfer(session_id, |reply| Request::Remove {
            path,
            is_dir,
            reply,
        })
    }

    fn transfer(
        &self,
        session_id: &str,
        build: impl FnOnce(Sender<Result<(), String>>) -> Request,
    ) -> Result<(), String> {
        let tx = {
            let sessions = self
                .sessions
                .lock()
                .map_err(|_| "会话表被污染".to_string())?;
            sessions
                .get(session_id)
                .ok_or_else(|| format!("会话不存在：{session_id}"))?
                .tx
                .clone()
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        tx.send(build(reply_tx))
            .map_err(|_| "会话已关闭".to_string())?;
        reply_rx.recv().map_err(|_| "会话已关闭".to_string())?
    }

    pub fn disconnect(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .map(|mut map| map.remove(session_id).is_some())
            .unwrap_or(false)
    }

    pub fn session_ids(&self) -> Vec<String> {
        self.sessions
            .lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// 终端会话的命令通道。
pub enum ShellCommand {
    Write(Vec<u8>),
    Resize(u32, u32),
    Close,
}

/// 可以被前端写入 / 调整大小的终端句柄。
pub struct ShellHandle {
    tx: Sender<ShellCommand>,
}

impl ShellHandle {
    pub fn write(&self, bytes: Vec<u8>) -> Result<(), String> {
        self.tx
            .send(ShellCommand::Write(bytes))
            .map_err(|_| "终端已关闭".to_string())
    }

    pub fn resize(&self, cols: u32, rows: u32) -> Result<(), String> {
        self.tx
            .send(ShellCommand::Resize(cols, rows))
            .map_err(|_| "终端已关闭".to_string())
    }

    pub fn close(&self) {
        let _ = self.tx.send(ShellCommand::Close);
    }
}

/// 打开一个交互式 shell。输出通过回调以字节块的形式推送。
///
/// 终端使用**独立的 SSH 会话**：libssh2 的阻塞读会锁住整个会话，
/// 若与 SFTP 共用一个连接，终端一读就会卡住文件操作。
pub fn open_shell<F>(
    params: ConnectParams,
    cols: u32,
    rows: u32,
    on_output: F,
) -> Result<ShellHandle, String>
where
    F: FnMut(&[u8]) + Send + 'static,
{
    let session = connect_raw(params)?;
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("sfterm-shell".to_string())
        .spawn(move || shell_loop(session, cols, rows, rx, on_output))
        .map_err(|e| format!("启动终端线程失败：{e}"))?;
    Ok(ShellHandle { tx })
}

fn shell_loop<F>(session: Session, cols: u32, rows: u32, rx: Receiver<ShellCommand>, mut on_output: F)
where
    F: FnMut(&[u8]),
{
    // 建通道 / 申请 PTY / 启动 shell 必须在阻塞模式下完成，
    // 之后再切非阻塞，否则这些调用会直接返回 WouldBlock。
    let mut channel = match session.channel_session() {
        Ok(channel) => channel,
        Err(_) => return,
    };
    if channel
        .request_pty("xterm-256color", None, Some((cols, rows, 0, 0)))
        .is_err()
    {
        return;
    }
    if channel.shell().is_err() {
        return;
    }

    session.set_blocking(false);

    let mut pending: Vec<u8> = Vec::new();
    let mut resize: Option<(u32, u32)> = None;
    let mut buf = [0u8; 8192];
    let mut closing = false;

    loop {
        loop {
            match rx.try_recv() {
                Ok(ShellCommand::Write(bytes)) => pending.extend_from_slice(&bytes),
                Ok(ShellCommand::Resize(c, r)) => resize = Some((c, r)),
                Ok(ShellCommand::Close) => {
                    closing = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    closing = true;
                    break;
                }
            }
        }
        if closing {
            break;
        }

        if let Some((c, r)) = resize.take() {
            let _ = channel.request_pty_size(c, r, Some(0), Some(0));
        }

        if !pending.is_empty() {
            match channel.write(&pending) {
                Ok(n) => {
                    pending.drain(..n);
                    let _ = channel.flush();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
        }

        match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => on_output(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }

        thread::sleep(Duration::from_millis(5));
    }

    let _ = channel.send_eof();
    let _ = channel.wait_close();
}

/// 只建立连接、握手、认证，不启动 SFTP 工作线程。
fn connect_raw(params: ConnectParams) -> Result<Session, String> {
    let addr = format!("{}:{}", params.host, params.port);
    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| format!("解析地址失败：{e}"))?
        .next()
        .ok_or_else(|| format!("解析不到地址：{addr}"))?;

    let tcp = TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("TCP 连接失败：{e}"))?;
    tcp.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
    tcp.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败：{e}"))?;
    session.set_tcp_stream(tcp);
    session.set_timeout(IO_TIMEOUT_MS);
    session
        .handshake()
        .map_err(|e| format!("SSH 握手失败：{e}"))?;

    match &params.auth {
        AuthMethod::Password { password } => session
            .userauth_password(&params.username, password)
            .map_err(|e| format!("密码认证失败：{e}"))?,
        AuthMethod::Key { path, passphrase } => session
            .userauth_pubkey_file(&params.username, None, Path::new(path), passphrase.as_deref())
            .map_err(|e| format!("密钥认证失败：{e}"))?,
    }

    if !session.authenticated() {
        return Err("认证未通过".to_string());
    }

    session.set_keepalive(true, KEEPALIVE_SECS);
    Ok(session)
}

fn open_session(params: ConnectParams, id: String) -> Result<(SessionHandle, ConnectInfo), String> {
    let addr = format!("{}:{}", params.host, params.port);
    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| format!("解析地址失败：{e}"))?
        .next()
        .ok_or_else(|| format!("解析不到地址：{addr}"))?;

    let tcp = TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("TCP 连接失败：{e}"))?;
    tcp.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
    tcp.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败：{e}"))?;
    session.set_tcp_stream(tcp);
    session.set_timeout(IO_TIMEOUT_MS);
    session
        .handshake()
        .map_err(|e| format!("SSH 握手失败：{e}"))?;

    match &params.auth {
        AuthMethod::Password { password } => session
            .userauth_password(&params.username, password)
            .map_err(|e| format!("密码认证失败：{e}"))?,
        AuthMethod::Key { path, passphrase } => session
            .userauth_pubkey_file(
                &params.username,
                None,
                Path::new(path),
                passphrase.as_deref(),
            )
            .map_err(|e| format!("密钥认证失败：{e}"))?,
    }

    if !session.authenticated() {
        return Err("认证未通过".to_string());
    }

    session.set_keepalive(true, KEEPALIVE_SECS);

    let info = ConnectInfo {
        session_id: id,
        banner: session.banner().map(|s| s.to_string()),
        host_key_type: session
            .host_key()
            .map(|(_, kind)| format!("{kind:?}"))
            .unwrap_or_else(|| "未知".to_string()),
        fingerprint_sha256: session
            .host_key_hash(HashType::Sha256)
            .map(hex)
            .unwrap_or_default(),
        kex: session.methods(MethodType::Kex).map(|s| s.to_string()),
        cipher: session.methods(MethodType::CryptCs).map(|s| s.to_string()),
    };

    let (tx, rx) = mpsc::channel::<Request>();
    thread::Builder::new()
        .name(format!("sfterm-ssh-{}", info.session_id))
        .spawn(move || worker(session, rx))
        .map_err(|e| format!("启动会话线程失败：{e}"))?;

    Ok((SessionHandle { tx }, info))
}

fn worker(session: Session, rx: mpsc::Receiver<Request>) {
    let sftp = match session.sftp() {
        Ok(sftp) => sftp,
        Err(e) => {
            let message = format!("SFTP 子系统不可用：{e}");
            while let Ok(request) = rx.recv() {
                match request {
                    Request::ListDir { reply, .. } => {
                        let _ = reply.send(Err(message.clone()));
                    }
                    Request::RealPath { reply } => {
                        let _ = reply.send(Err(message.clone()));
                    }
                    Request::Upload { reply, .. }
                    | Request::Download { reply, .. }
                    | Request::Mkdir { reply, .. }
                    | Request::Rename { reply, .. }
                    | Request::Remove { reply, .. } => {
                        let _ = reply.send(Err(message.clone()));
                    }
                }
            }
            return;
        }
    };

    while let Ok(request) = rx.recv() {
        match request {
            Request::ListDir { path, reply } => {
                let _ = reply.send(list_dir(&sftp, &path));
            }
            Request::RealPath { reply } => {
                let result = sftp
                    .realpath(Path::new("."))
                    .map(|path| path.to_string_lossy().into_owned())
                    .map_err(|e| format!("获取远端主目录失败：{e}"));
                let _ = reply.send(result);
            }
            Request::Mkdir { path, reply } => {
                let result = sftp
                    .mkdir(Path::new(&path), 0o755)
                    .map_err(|e| format!("新建文件夹失败：{e}"));
                let _ = reply.send(result);
            }
            Request::Rename { from, to, reply } => {
                let result = sftp
                    .rename(Path::new(&from), Path::new(&to), None)
                    .map_err(|e| format!("重命名失败：{e}"));
                let _ = reply.send(result);
            }
            Request::Remove {
                path,
                is_dir,
                reply,
            } => {
                let result = remove_remote(&sftp, &path, is_dir);
                let _ = reply.send(result);
            }
            Request::Upload {
                local,
                remote,
                overwrite,
                progress,
                reply,
            } => {
                let result = upload_file(&sftp, &local, &remote, overwrite, &progress);
                let _ = reply.send(result);
            }
            Request::Download {
                remote,
                local,
                overwrite,
                progress,
                reply,
            } => {
                let result = download_file(&sftp, &remote, &local, overwrite, &progress);
                let _ = reply.send(result);
            }
        }
    }
}

/// 删除远端文件或目录（目录会递归删除）。
fn remove_remote(sftp: &ssh2::Sftp, path: &str, is_dir: bool) -> Result<(), String> {
    if !is_dir {
        return sftp
            .unlink(Path::new(path))
            .map_err(|e| format!("删除文件失败：{e}"));
    }

    let entries = sftp
        .readdir(Path::new(path))
        .map_err(|e| format!("读取目录失败：{e}"))?;

    for (entry_path, stat) in entries {
        let child_is_dir = stat.is_dir() && !stat.file_type().is_symlink();
        remove_remote(sftp, entry_path.to_string_lossy().as_ref(), child_is_dir)?;
    }

    sftp.rmdir(Path::new(path))
        .map_err(|e| format!("删除目录失败：{e}"))
}

/// 本地目录列表。与远端使用同一份 `RemoteEntry`，前端可以复用同一个列表组件。
pub fn list_local(path: &str) -> Result<Vec<RemoteEntry>, String> {
    let read = std::fs::read_dir(path).map_err(|e| format!("读取本地目录失败：{e}"))?;
    let mut out = Vec::new();

    for item in read.flatten() {
        let entry_path = item.path();
        let meta = match std::fs::symlink_metadata(&entry_path) {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let is_dir = meta.is_dir();
        let is_symlink = meta.file_type().is_symlink();
        let size = if is_dir { 0 } else { meta.len() };

        #[cfg(unix)]
        let perm = Some(meta.mode() & 0o7777);
        #[cfg(not(unix))]
        let perm = None;

        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        out.push(RemoteEntry {
            name: item.file_name().to_string_lossy().into_owned(),
            path: entry_path.to_string_lossy().into_owned(),
            is_dir,
            is_symlink,
            size,
            perm,
            mtime,
        });
    }

    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}

/// 当前用户主目录，供前端做本地栏的默认起点。
pub fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}

const CHUNK: usize = 32 * 1024;
/// 每 256KB 汇报一次进度，避免事件过于密集。
const PROGRESS_EVERY: u64 = 256 * 1024;

fn upload_file(
    sftp: &ssh2::Sftp,
    local: &str,
    remote: &str,
    overwrite: bool,
    progress: &Sender<TransferProgress>,
) -> Result<(), String> {
    let mut src = std::fs::File::open(local).map_err(|e| format!("打开本地文件失败：{e}"))?;
    let total = src.metadata().map(|m| m.len()).unwrap_or(0);

    let exists = sftp.stat(Path::new(remote)).is_ok();
    if exists && !overwrite {
        return Err("目标文件已存在".to_string());
    }

    let mut dst = sftp
        .create(Path::new(remote))
        .map_err(|e| format!("创建远端文件失败：{e}"))?;

    let mut buf = vec![0u8; CHUNK];
    let mut done = 0u64;
    let mut last_reported = 0u64;

    loop {
        let n = src
            .read(&mut buf)
            .map_err(|e| format!("读本地文件失败：{e}"))?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])
            .map_err(|e| format!("写远端文件失败：{e}"))?;
        done += n as u64;

        if done - last_reported >= PROGRESS_EVERY || done == total {
            last_reported = done;
            let _ = progress.send(TransferProgress { done, total });
        }
    }

    dst.close().map_err(|e| format!("关闭远端文件失败：{e}"))?;
    let _ = progress.send(TransferProgress { done, total });
    Ok(())
}

fn download_file(
    sftp: &ssh2::Sftp,
    remote: &str,
    local: &str,
    overwrite: bool,
    progress: &Sender<TransferProgress>,
) -> Result<(), String> {
    if Path::new(local).exists() && !overwrite {
        return Err("本地文件已存在".to_string());
    }

    let mut src = sftp
        .open(Path::new(remote))
        .map_err(|e| format!("打开远端文件失败：{e}"))?;
    let total = sftp
        .stat(Path::new(remote))
        .ok()
        .and_then(|stat| stat.size)
        .unwrap_or(0);

    if let Some(parent) = Path::new(local).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建本地目录失败：{e}"))?;
    }
    let mut dst = std::fs::File::create(local).map_err(|e| format!("创建本地文件失败：{e}"))?;

    let mut buf = vec![0u8; CHUNK];
    let mut done = 0u64;
    let mut last_reported = 0u64;

    loop {
        let n = src
            .read(&mut buf)
            .map_err(|e| format!("读远端文件失败：{e}"))?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])
            .map_err(|e| format!("写本地文件失败：{e}"))?;
        done += n as u64;

        if done - last_reported >= PROGRESS_EVERY || done == total {
            last_reported = done;
            let _ = progress.send(TransferProgress { done, total });
        }
    }

    let _ = progress.send(TransferProgress { done, total });
    Ok(())
}

fn list_dir(sftp: &ssh2::Sftp, path: &str) -> Result<Vec<RemoteEntry>, String> {
    let entries = sftp
        .readdir(Path::new(path))
        .map_err(|e| format!("读取目录失败：{e}"))?;

    let mut out: Vec<RemoteEntry> = entries
        .into_iter()
        .map(|(entry_path, stat)| RemoteEntry {
            name: entry_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| entry_path.display().to_string()),
            path: entry_path.to_string_lossy().into_owned(),
            is_dir: stat.is_dir(),
            is_symlink: stat.file_type().is_symlink(),
            size: stat.size.unwrap_or(0),
            perm: stat.perm,
            mtime: stat.mtime,
        })
        .collect();

    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_local_directory_with_files_and_dirs() {
        // 每次用独立目录：测试失败时不会把残留状态留给下一次运行。
        let base = std::env::temp_dir().join(format!("sfterm-core-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let sub = base.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(base.join("a.txt"), b"hi").unwrap();

        let entries = list_local(base.to_str().unwrap()).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"sub"));
        assert!(entries
            .iter()
            .find(|e| e.name == "a.txt")
            .map(|e| !e.is_dir && e.size == 2)
            .unwrap_or(false));
        // 目录排在文件前面
        assert_eq!(entries.first().map(|e| e.is_dir), Some(true));

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn home_dir_is_not_empty() {
        assert!(!home_dir().is_empty());
    }
}
