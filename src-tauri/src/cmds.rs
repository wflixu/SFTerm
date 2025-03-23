use ssh2::Session;
use std::{net::TcpStream, path::PathBuf};

#[tauri::command]
pub fn ssh_connect() -> Result<(), String> {
    let ip = "";
    let username = "ubuntu";
    let private_key_path = PathBuf::from("");
    println!("I was invoked from JavaScript!");
   
    // 1. 建立 TCP 连接
    let tcp = TcpStream::connect(format!("{}:22", ip)).expect("❌ TCP 连接失败");

    // 2. 初始化 SSH 会话
    let mut sess = Session::new().expect("❌ SSH 会话初始化失败");
    sess.set_tcp_stream(tcp);

    // 3. 握手协议（网页1中的 handshake 逻辑）
    sess.handshake().map_err(|e| e.to_string()).expect("❌ 握手失败");

    // 4. 密钥认证（网页2中的公钥文件认证逻辑）
    //  
    sess.userauth_pubkey_file(
        username,
        None, // 不使用公钥文件路径
        &private_key_path,
        None, // 私钥密码（如有需要可传入）
    )
    .map_err(|e| e.to_string())?;

    // 5. 验证认证状态
    if sess.authenticated() {
        println!("✅ SSH 认证成功");
        Ok(())
    } else {
        Err("❌ 密钥认证失败".to_string())
    }
}
