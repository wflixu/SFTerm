//! 引擎自检：不经过 Tauri，直接跑通 connect → auth → sftp → list_dir。

use sfterm_core::{AuthMethod, ConnectParams, Manager};
use std::env;
use std::process::ExitCode;

struct Args {
    host: String,
    port: u16,
    user: String,
    password: Option<String>,
    key: Option<String>,
    passphrase: Option<String>,
    path: String,
    limit: usize,
}

fn parse() -> Result<Args, String> {
    let mut host = None;
    let mut port = 22u16;
    let mut user = None;
    let mut password = None;
    let mut key = None;
    let mut passphrase = None;
    let mut path = "/".to_string();
    let mut limit = 20usize;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{arg} 缺少值"));
        match arg.as_str() {
            "--host" => host = Some(value()?),
            "--port" => port = value()?.parse().map_err(|_| "--port 需要数字")?,
            "--user" => user = Some(value()?),
            "--password" => password = Some(value()?),
            "--key" => key = Some(value()?),
            "--passphrase" => passphrase = Some(value()?),
            "--path" => path = value()?,
            "--limit" => limit = value()?.parse().map_err(|_| "--limit 需要数字")?,
            other => return Err(format!("未知参数：{other}")),
        }
    }

    Ok(Args {
        host: host.ok_or("缺少 --host")?,
        port,
        user: user.ok_or("缺少 --user")?,
        password,
        key,
        passphrase,
        path,
        limit,
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse()?;

    let auth = if let Some(key) = &args.key {
        AuthMethod::Key {
            path: key.clone(),
            passphrase: args.passphrase.clone(),
        }
    } else if let Some(password) = &args.password {
        AuthMethod::Password {
            password: password.clone(),
        }
    } else {
        return Err("需要 --key 或 --password".to_string());
    };

    let manager = Manager::new();
    let started = std::time::Instant::now();
    let info = manager.connect(ConnectParams {
        host: args.host,
        port: args.port,
        username: args.user,
        auth,
    })?;

    println!("✓ 已连接（{:?}）", started.elapsed());
    println!("  session   {}", info.session_id);
    println!("  服务器    {}", info.banner.unwrap_or_else(|| "未知".into()));
    println!("  主机密钥  {}", info.host_key_type);
    println!("  指纹      {}", info.fingerprint_sha256);
    println!("  KEX       {:?}", info.kex);
    println!("  加密      {:?}", info.cipher);

    let started = std::time::Instant::now();
    let entries = manager.list_dir(&info.session_id, &args.path)?;
    println!(
        "\n✓ {} 共 {} 项（{:?}）",
        args.path,
        entries.len(),
        started.elapsed()
    );

    for entry in entries.iter().take(args.limit) {
        let kind = if entry.is_dir {
            "d"
        } else if entry.is_symlink {
            "l"
        } else {
            "-"
        };
        println!(
            "  {} {:>10}  {}",
            kind,
            if entry.is_dir {
                "---".to_string()
            } else {
                entry.size.to_string()
            },
            entry.name
        );
    }

    manager.disconnect(&info.session_id);
    println!("\n✅ 引擎自检通过");
    Ok(())
}
