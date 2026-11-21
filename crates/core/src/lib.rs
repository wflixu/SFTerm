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
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
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

enum Request {
    ListDir {
        path: String,
        reply: Sender<Result<Vec<RemoteEntry>, String>>,
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
            let sessions = self.sessions.lock().map_err(|_| "会话表被污染".to_string())?;
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

fn open_session(
    params: ConnectParams,
    id: String,
) -> Result<(SessionHandle, ConnectInfo), String> {
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
        cipher: session
            .methods(MethodType::CryptCs)
            .map(|s| s.to_string()),
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
        }
    }
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
