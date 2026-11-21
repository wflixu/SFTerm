//! SFTerm 连接探针
//!
//! 用途：在接入 UI 之前，先确认一台（可能是很老的）服务器能不能连、协商出什么算法、
//! 认证方式有哪些、SFTP 能不能列目录。
//!
//! 用法：
//!   cargo run --manifest-path tools/probe/Cargo.toml -- \
//!     --host 1.2.3.4 --user root --password 'secret' --path /var/www
//!
//!   cargo run --manifest-path tools/probe/Cargo.toml -- \
//!     --host 1.2.3.4 --user root --key ~/.ssh/id_rsa --passphrase 'xxx'
//!
//! 密码/口令也可以通过环境变量传，避免进 shell 历史：
//!   SFT_PROBE_PASSWORD=secret cargo run --manifest-path tools/probe/Cargo.toml -- --host ... --user ...

use ssh2::{FileStat, HashType, MethodType, Session};
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

const USAGE: &str = "\
SFTerm 连接探针

用法:
  sft-probe --host <HOST> --user <USER> (--password <PW> | --key <PATH>) [选项]

必填:
  --host <HOST>        主机名或 IP
  --user <USER>        用户名
  --password <PW>      密码认证（或环境变量 SFT_PROBE_PASSWORD）
  --key <PATH>         私钥文件（或与 --password 二选一）
  --passphrase <PW>    私钥口令（或环境变量 SFT_PROBE_PASSPHRASE）

可选:
  --port <PORT>        端口，默认 22
  --path <PATH>        要列出的远端目录，默认 /
  --limit <N>          最多打印多少条，默认 20
  --put <LOCAL>        上传本地文件（需配合 --remote <远端路径>）
  --get <REMOTE>       下载远端文件（需配合 --local <本地路径>）
  --remote <PATH>      --put 的目标路径
  --local <PATH>       --get 的保存路径
  --rm <REMOTE>        删除远端文件（测完清理用）
  --compress           启用 zlib 压缩（测带宽受限场景）
  -h, --help           显示本帮助
";

struct Args {
    host: String,
    port: u16,
    user: String,
    password: Option<String>,
    key: Option<PathBuf>,
    passphrase: Option<String>,
    path: String,
    limit: usize,
    put: Option<PathBuf>,
    get: Option<String>,
    remote: Option<String>,
    local: Option<PathBuf>,
    rm: Option<String>,
    compress: bool,
}

fn next_arg(it: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    it.next().ok_or_else(|| format!("{flag} 后面缺少值"))
}

fn parse_args() -> Result<Args, String> {
    let mut host = None;
    let mut port = 22u16;
    let mut user = None;
    let mut password = None;
    let mut key = None;
    let mut passphrase = None;
    let mut path = "/".to_string();
    let mut limit = 20usize;
    let mut put = None;
    let mut get = None;
    let mut remote = None;
    let mut local = None;
    let mut rm = None;
    let mut compress = false;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--host" | "-H" => host = Some(next_arg(&mut it, "--host")?),
            "--port" | "-p" => {
                port = next_arg(&mut it, "--port")?
                    .parse()
                    .map_err(|_| "--port 需要数字".to_string())?
            }
            "--user" | "-u" => user = Some(next_arg(&mut it, "--user")?),
            "--password" => password = Some(next_arg(&mut it, "--password")?),
            "--key" | "-i" => key = Some(PathBuf::from(next_arg(&mut it, "--key")?)),
            "--passphrase" => passphrase = Some(next_arg(&mut it, "--passphrase")?),
            "--path" => path = next_arg(&mut it, "--path")?,
            "--put" => put = Some(PathBuf::from(next_arg(&mut it, "--put")?)),
            "--get" => get = Some(next_arg(&mut it, "--get")?),
            "--remote" => remote = Some(next_arg(&mut it, "--remote")?),
            "--local" => local = Some(PathBuf::from(next_arg(&mut it, "--local")?)),
            "--rm" => rm = Some(next_arg(&mut it, "--rm")?),
            "--compress" => compress = true,
            "--limit" => {
                limit = next_arg(&mut it, "--limit")?
                    .parse()
                    .map_err(|_| "--limit 需要数字".to_string())?
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("未知参数：{other}（用 -h 查看用法）")),
        }
    }

    Ok(Args {
        host: host.ok_or("缺少 --host")?,
        port,
        user: user.ok_or("缺少 --user")?,
        password: password.or_else(|| env::var("SFT_PROBE_PASSWORD").ok()),
        key,
        passphrase: passphrase.or_else(|| env::var("SFT_PROBE_PASSPHRASE").ok()),
        path,
        limit,
        put,
        get,
        remote,
        local,
        rm,
        compress,
    })
}

fn client_algs(sess: &Session, mt: MethodType) -> String {
    match sess.supported_algs(mt) {
        Ok(v) => v.join(", "),
        Err(e) => format!("<无法获取: {e}>"),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn mode_string(stat: &FileStat) -> String {
    let mode = stat.perm.unwrap_or(0);
    let kind = if stat.is_dir() {
        'd'
    } else if stat.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };
    let mut s = String::with_capacity(10);
    s.push(kind);
    for shift in [6, 3, 0] {
        let bits = (mode >> shift) & 0o7;
        s.push(if bits & 0o4 != 0 { 'r' } else { '-' });
        s.push(if bits & 0o2 != 0 { 'w' } else { '-' });
        s.push(if bits & 0o1 != 0 { 'x' } else { '-' });
    }
    s
}

fn human_size(stat: &FileStat) -> String {
    if stat.is_dir() {
        return "---".to_string();
    }
    match stat.size {
        Some(n) if n < 1024 => format!("{n} B"),
        Some(n) if n < 1024 * 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        Some(n) if n < 1024 * 1024 * 1024 => format!("{:.1} MB", n as f64 / 1_048_576.0),
        Some(n) => format!("{:.1} GB", n as f64 / 1_073_741_824.0),
        None => "?".to_string(),
    }
}

fn report(op: &str, bytes: u64, elapsed: Duration) {
    let secs = elapsed.as_secs_f64().max(0.001);
    let mb = bytes as f64 / 1_048_576.0;
    println!(
        "  ✓ {op} {:.2} MB，用时 {:.2}s，{:.2} MB/s",
        mb,
        secs,
        mb / secs
    );
}

fn put_file(sftp: &ssh2::Sftp, local: &Path, remote: &str) -> Result<(), String> {
    let mut src = std::fs::File::open(local).map_err(|e| format!("打开本地文件失败：{e}"))?;
    let mut dst = sftp
        .create(Path::new(remote))
        .map_err(|e| format!("创建远端文件失败：{e}"))?;
    let mut buf = vec![0u8; 32 * 1024];
    let started = Instant::now();
    let mut sent = 0u64;
    loop {
        let n = src.read(&mut buf).map_err(|e| format!("读本地失败：{e}"))?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])
            .map_err(|e| format!("写远端失败：{e}"))?;
        sent += n as u64;
    }
    dst.close().map_err(|e| format!("关闭远端文件失败：{e}"))?;
    report("上传", sent, started.elapsed());
    Ok(())
}

fn get_file(sftp: &ssh2::Sftp, remote: &str, local: &Path) -> Result<(), String> {
    let mut src = sftp
        .open(Path::new(remote))
        .map_err(|e| format!("打开远端文件失败：{e}"))?;
    let mut dst = std::fs::File::create(local).map_err(|e| format!("创建本地文件失败：{e}"))?;
    let mut buf = vec![0u8; 32 * 1024];
    let started = Instant::now();
    let mut received = 0u64;
    loop {
        let n = src.read(&mut buf).map_err(|e| format!("读远端失败：{e}"))?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])
            .map_err(|e| format!("写本地失败：{e}"))?;
        received += n as u64;
    }
    report("下载", received, started.elapsed());
    Ok(())
}

fn run() -> Result<(), String> {
    let args = parse_args()?;

    println!("→ TCP {}:{}", args.host, args.port);
    let t0 = Instant::now();
    let tcp = TcpStream::connect((args.host.as_str(), args.port))
        .map_err(|e| format!("TCP 连接失败：{e}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(15))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(15))).ok();
    println!("  ✓ 已连接（{:?}）", t0.elapsed());

    let mut sess = Session::new().map_err(|e| format!("创建会话失败：{e}"))?;
    sess.set_tcp_stream(tcp);
    sess.set_timeout(15_000);

    if args.compress {
        sess.set_compress(true);
        println!("  压缩：已请求（zlib）");
    }
    println!("→ SSH 握手");
    if let Err(e) = sess.handshake() {
        eprintln!("\n❌ 握手失败：{e}");
        eprintln!("\n客户端支持的 KEX：\n  {}", client_algs(&sess, MethodType::Kex));
        eprintln!("\n客户端支持的加密：\n  {}", client_algs(&sess, MethodType::CryptCs));
        eprintln!("\n客户端支持的主机密钥：\n  {}", client_algs(&sess, MethodType::HostKey));
        eprintln!("\n提示：如果是 “no matching key exchange method”，说明服务器只提供老算法，");
        eprintln!("      需要在客户端放开对应算法，而不是改服务器。");
        return Err("握手失败".to_string());
    }
    println!("  ✓ 握手完成（累计 {:?}）", t0.elapsed());

    let banner = sess.banner().unwrap_or("<未提供>");
    println!("\n服务器版本：{banner}");

    println!("\n协商结果：");
    let methods = [
        ("KEX", MethodType::Kex),
        ("HostKey", MethodType::HostKey),
        ("Cipher C→S", MethodType::CryptCs),
        ("Cipher S→C", MethodType::CryptSc),
        ("MAC C→S", MethodType::MacCs),
        ("MAC S→C", MethodType::MacSc),
        ("Comp C→S", MethodType::CompCs),
        ("SignAlgo", MethodType::SignAlgo),
    ];
    for (label, mt) in methods {
        println!("  {label:<12} {:?}", sess.methods(mt));
    }

    if let Some((key, kind)) = sess.host_key() {
        println!("\n主机密钥类型：{kind:?}（{} 字节）", key.len());
    }
    if let Some(hash) = sess.host_key_hash(HashType::Sha256) {
        println!("SHA256 指纹：{}", hex(hash));
    }

    match sess.auth_methods(&args.user) {
        Ok(m) => println!("\n服务器允许的认证方式：{m}"),
        Err(e) => println!("\n认证方式探测失败：{e}"),
    }

    println!("\n→ 认证");
    let t1 = Instant::now();
    if let Some(key) = &args.key {
        println!("  使用密钥 {}", key.display());
        sess.userauth_pubkey_file(&args.user, None, key, args.passphrase.as_deref())
            .map_err(|e| format!("密钥认证失败：{e}"))?;
    } else if let Some(pw) = &args.password {
        println!("  使用密码");
        sess.userauth_password(&args.user, pw)
            .map_err(|e| format!("密码认证失败：{e}"))?;
    } else {
        return Err("需要 --password 或 --key（或设置 SFT_PROBE_PASSWORD）".to_string());
    }
    if !sess.authenticated() {
        return Err("认证未通过".to_string());
    }
    println!("  ✓ 认证成功（{:?}）", t1.elapsed());

    println!("\n→ 打开 SFTP 子系统");
    let t2 = Instant::now();
    let sftp = sess.sftp().map_err(|e| format!("SFTP 初始化失败：{e}"))?;
    println!("  ✓ 已打开（{:?}）", t2.elapsed());

    println!("\n→ 列目录 {}", args.path);
    let t3 = Instant::now();
    let mut entries = sftp
        .readdir(Path::new(&args.path))
        .map_err(|e| format!("列目录失败：{e}"))?;
    let elapsed = t3.elapsed();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let dirs = entries.iter().filter(|(_, s)| s.is_dir()).count();
    println!(
        "  ✓ {} 项（{} 目录 / {} 文件，{:?}）",
        entries.len(),
        dirs,
        entries.len() - dirs,
        elapsed
    );

    if entries.is_empty() {
        println!("\n（目录为空）");
    } else {
        println!(
            "\n{:<11} {:>10}  {:<16}  {}",
            "权限", "大小", "修改时间(epoch)", "名称"
        );
        for (path, stat) in entries.iter().take(args.limit) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            let mtime = stat
                .mtime
                .map(|t| t.to_string())
                .unwrap_or_else(|| "?".to_string());
            println!(
                "{:<11} {:>10}  {:<16}  {}",
                mode_string(stat),
                human_size(stat),
                mtime,
                name
            );
        }
        if entries.len() > args.limit {
            println!("… 还有 {} 项", entries.len() - args.limit);
        }
    }

    if let Some(local_path) = &args.put {
        let remote_path = args
            .remote
            .as_deref()
            .ok_or("--put 需要配合 --remote <远端路径>")?;
        println!("\n→ 上传 {} → {}", local_path.display(), remote_path);
        put_file(&sftp, local_path, remote_path)?;
    }

    if let Some(remote_path) = &args.get {
        let local_path = args
            .local
            .as_deref()
            .ok_or("--get 需要配合 --local <本地路径>")?;
        println!("\n→ 下载 {} → {}", remote_path, local_path.display());
        get_file(&sftp, remote_path, local_path)?;
    }

    if let Some(remote_path) = &args.rm {
        println!("\n→ 删除 {}", remote_path);
        sftp.unlink(Path::new(remote_path))
            .map_err(|e| format!("删除失败：{e}"))?;
        println!("  ✓ 已删除");
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("\n✅ 探针完成");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("\n❌ {e}");
            ExitCode::FAILURE
        }
    }
}
