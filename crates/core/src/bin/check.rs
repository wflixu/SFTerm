//! 引擎自检：不经过 Tauri，直接跑通 connect → auth → sftp → list_dir。

use sfterm_core::{AuthMethod, ConnectParams, Manager, TransferProgress};
use std::env;
use std::process::ExitCode;
use std::sync::mpsc;

struct Args {
    host: String,
    port: u16,
    user: String,
    password: Option<String>,
    key: Option<String>,
    passphrase: Option<String>,
    path: String,
    limit: usize,
    upload: Option<(String, String)>,
    download: Option<(String, String)>,
    mkdir: Option<String>,
    rename: Option<(String, String)>,
    remove: Option<(String, bool)>,
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
    let mut upload_local: Option<String> = None;
    let mut upload_remote: Option<String> = None;
    let mut download_remote: Option<String> = None;
    let mut download_local: Option<String> = None;
    let mut mkdir: Option<String> = None;
    let mut rename_from: Option<String> = None;
    let mut rename_to: Option<String> = None;
    let mut remove_path: Option<String> = None;
    let mut remove_is_dir = false;

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
            "--upload" => upload_local = Some(value()?),
            "--remote" => upload_remote = Some(value()?),
            "--download" => download_remote = Some(value()?),
            "--local" => download_local = Some(value()?),
            "--mkdir" => mkdir = Some(value()?),
            "--mv" => rename_from = Some(value()?),
            "--to" => rename_to = Some(value()?),
            "--rm" => remove_path = Some(value()?),
            "--rm-dir" => remove_is_dir = true,
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
        upload: upload_local.zip(upload_remote),
        download: download_remote.zip(download_local),
        mkdir,
        rename: rename_from.zip(rename_to),
        remove: remove_path.map(|path| (path, remove_is_dir)),
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
    println!(
        "  服务器    {}",
        info.banner.unwrap_or_else(|| "未知".into())
    );
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

    if let Some((local, remote)) = &args.upload {
        let (progress_tx, progress_rx) = mpsc::channel::<TransferProgress>();
        let printer = std::thread::spawn(move || {
            for progress in progress_rx {
                if progress.total > 0 {
                    println!(
                        "  上传 {}%",
                        (progress.done as f64 / progress.total as f64 * 100.0) as u32
                    );
                }
            }
        });
        let started = std::time::Instant::now();
        manager.upload(
            &info.session_id,
            local.clone(),
            remote.clone(),
            true,
            progress_tx,
        )?;
        printer.join().ok();
        println!("✓ 上传完成（{:?}）", started.elapsed());
    }

    if let Some((remote, local)) = &args.download {
        let (progress_tx, progress_rx) = mpsc::channel::<TransferProgress>();
        let printer = std::thread::spawn(move || {
            for progress in progress_rx {
                if progress.total > 0 {
                    println!(
                        "  下载 {}%",
                        (progress.done as f64 / progress.total as f64 * 100.0) as u32
                    );
                }
            }
        });
        let started = std::time::Instant::now();
        manager.download(
            &info.session_id,
            remote.clone(),
            local.clone(),
            true,
            progress_tx,
        )?;
        printer.join().ok();
        println!("✓ 下载完成（{:?}）", started.elapsed());
    }

    if let Some(path) = &args.mkdir {
        manager.mkdir(&info.session_id, path.clone())?;
        println!("✓ 已创建目录 {path}");
    }

    if let Some((from, to)) = &args.rename {
        manager.rename(&info.session_id, from.clone(), to.clone())?;
        println!("✓ 已重命名 {from} -> {to}");
    }

    if let Some((path, is_dir)) = &args.remove {
        manager.remove(&info.session_id, path.clone(), *is_dir)?;
        println!("✓ 已删除 {path}");
    }

    manager.disconnect(&info.session_id);
    println!("\n✅ 引擎自检通过");
    Ok(())
}
