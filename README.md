<!-- 项目LOGO与标题 -->
<div align="center">
  <a href="https://github.com/wflixu/SFTerm">
    <img src="docs/logo.png" alt="SFTerm Logo" width="120" height="120">
  </a>
  <h1 align="center">SFTerm</h1>
  <p align="center">
    🔐 Secure & Fast Terminal - 基于现代技术的SSH/SFTP一体化客户端
    <br />
    <a href="https://github.com/wflixu/SFTerm/issues">问题反馈</a>
    · <a href="https://github.com/wflixu/SFTerm/discussions">功能建议</a>
    · <a href="https://github.com/wflixu/SFTerm/releases/latest">下载最新版</a>
  </p>
  <!-- 动态徽章 -->
  [![Rust Build](https://img.shields.io/github/actions/workflow/status/wflixu/SFTerm/rust.yml?label=Nightly%20Build)](https://github.com/wflixu/SFTerm/actions)
  [![Crates.io](https://img.shields.io/crates/v/sfterm)](https://crates.io/crates/sfterm)
  [![Downloads](https://img.shields.io/github/downloads/wflixu/SFTerm/total?color=blue)](https://github.com/wflixu/SFTerm/releases)
</div>

---

## 🌟 特性亮点

### 🔑 安全为先
- **军事级加密**：采用Rust实现的AES-256-GCM算法保护密钥存储
- **生物认证**：支持Windows Hello/macOS TouchID登录密钥库
- **安全审计**：集成[cargo-audit](https://crates.io/crates/cargo-audit)自动扫描依赖漏洞

### 🚀 极致性能
- **零延迟渲染**：基于WebAssembly加速的xterm.js 5.0+
- **多路复用**：单进程支持50+并发SSH会话（压力测试证明）
- **智能压缩**：zlib压缩SSH流量，降低网络消耗达40%

### 📁 SFTP增强
- **双向同步**：本地与远程目录实时同步（类似rsync）
- **文件预览**：直接查看图片/文本文件（支持hex模式）
- **批量操作**：正则表达式匹配多文件传输

---

## 🖥️ 界面预览
<div align="center">
  <img src="docs/ui-preview.gif" alt="交互演示" width="800">
  <p>多标签管理 + 暗色主题 + 实时性能监控</p>
</div>

---

## 🛠️ 技术栈

<div align="center">
  
| 层级       | 技术组件                          |
|------------|----------------------------------|
| **前端**   | Vue3 + TypeScript + xterm.js     |
| **后端**   | Rust + Tokio + ssh2-rs           |
| **构建**   | Tauri + Webpack + wasm-pack      |
| **安全**   | ring(aes-gcm) + system keyring   |
| **工具链** | Clippy + ESLint + GitHub Actions |

</div>

---

## 🚀 快速安装

### 二进制安装
```bash
# macOS
brew tap wflixu/sfterm
brew install sfterm

# Windows
winget install SFTerm

# Linux
curl -sSL https://install.sfterm.dev | bash
```

### 从源码构建
```bash
git clone https://github.com/wflixu/SFTerm.git
cd SFTerm

# 开发模式
cargo tauri dev --features debug-console

# 生产构建
cargo tauri build --release
```

---

## 📚 使用示例

### 命令行模式
```bash
sfterm connect user@host --key ~/.ssh/id_ed25519

# 批量执行命令
sfterm exec -h host1,host2 "uname -a" --parallel

# SFTP同步目录
sfterm sync ./local/pdir user@host:/remote/path --delete --exclude "*.tmp"
```

### 配置文件示例
```yaml
# ~/.sfterm/config.yaml
profiles:
  - name: "生产服务器"
    host: "10.1.1.100"
    user: "admin"
    auth:
      type: "key"
      key_path: "~/.ssh/prod.key"
    sftp:
      default_dir: "/var/log"
      sync_filters: ["*.log", "!error.log"]
```

---

## 🤝 贡献指南

我们欢迎以下类型的贡献：
- 🌐 国际化：帮助翻译[UI字符串](src/locales)
- 📦 插件开发：扩展[插件系统](docs/plugins.md)
- 🐞 Bug捕获：提交可复现的[测试用例](tests)

**代码规范**：
```bash
# Rust代码质量检查
cargo clippy --all-targets -- -D warnings

# TypeScript规范检查
npm run lint:fix

# 提交前自动化测试
cargo test && npm run test
```

---

## 📜 开源协议
本项目采用 **Apache 2.0** 与 **MIT** 双重授权，商业使用需保留版权声明。完整条款见 [LICENSE](LICENSE)。


