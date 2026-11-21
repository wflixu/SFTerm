# ADR-002：保持 Tauri 技术栈，产品以 SFTP 为核心

- **状态**：已接受（Accepted）
- **日期**：2026-11-21
- **取代**：[ADR-001](adr-001-swift-native-macos.md)（Swift 原生方案）

---

## 1. 背景

ADR-001 决定把项目改写为 macOS 原生 Swift 应用（libghostty + Citadel），前提是「终端体验是核心竞争力」。随后对产品定位的澄清推翻了该前提：

- **SSH 是低频功能**，终端只作辅助；
- **SFTP 是必须功能**，也是主要使用场景；
- **交互和设计**才是产品差异化的来源，技术栈本身不是用户感知的价值；
- 认证只需要「密钥 + 密码」，不需要 MFA、不需要 RSA 兼容。

在这组约束下，Swift 方案的主要优势（Ghostty 级终端、原生集成）与主要劣势（协议层要自建、UI 迭代慢、12 周重写）都失去了决定性。

## 2. 决策

**继续使用 Tauri 技术栈**，不做原生重写：

| 层 | 选型 |
|---|---|
| 壳 / 构建 | Tauri 2（macOS 为主，保留跨平台可能） |
| UI | Vue 3 + TypeScript + PrimeVue 4（Aura）+ Pinia |
| 终端 | xterm.js 5 + addon-fit |
| SSH / SFTP | Rust `ssh2`（libssh2），经 Tauri command + event 暴露 |
| 存储 | 主机/设置 → SQLite（或过渡期 JSON）；密钥与口令 → Keychain |

## 3. 理由

1. **产品定位决定选型**：SFTP 优先的产品里，决定体验的是文件浏览、拖拽、队列、预览、键盘流——全部是 UI 工程，与终端内核无关。
2. **协议功能无缺口**：`ssh2` 原生支持密码、公钥、带口令私钥、SFTP v3、断点、压缩，正好覆盖「密钥 + 密码」的需求。
3. **交互迭代速度**：设计驱动型产品需要高频试错，Web UI 的热更新与 CSS 表现力是优势。
4. **已有投入可复用**：主界面骨架、主题、主机 CRUD 已存在（[src/](../src/)），重写等于把唯一能快速验证交互的脚手架拆掉。
5. **换栈不减少工作量**：SFTP 双栏/队列/预览/拖拽的 4–6 周工作在两套栈里都要重做，换栈只增加协议层重建成本。
6. **App Store 沙盒对两者同等约束**（本地文件访问受限、无自更新），不构成换栈理由。

## 4. 后果与代价

| 代价 | 缓解 |
|---|---|
| 拖出到 Finder 不能只靠 HTML5 DnD | 使用原生插件：[drag-rs](https://github.com/crabnebula-dev/drag-rs) 或 [tauri-plugin-dragout](https://github.com/alexqqqqqq777/tauri-plugin-dragout) |
| 无系统 Quick Look 面板 | 自绘预览（图片/文本/PDF/音视频），视觉更统一；必要时写小型原生桥 |
| 终端能力上限为 xterm.js | 对低频 SSH 足够；如未来终端变核心，重审本 ADR |
| 无原生控件手感 | 通过 Rust + `objc2` 补平台能力，或用插件生态 |
| Mac App Store 构建有额外工作 | Tauri 的 `mas` feature + 沙盒 entitlement，预留 1–2 周 |

## 5. 重审触发条件

出现以下任一情况时，重新评估原生方案：

1. 终端成为主功能（用户重度使用 SSH）；
2. 需要 Finder 挂载、同步盘、系统服务集成；
3. App Store 是唯一渠道且插件摩擦严重影响体验；
4. 实测 WebView 的输入延迟、滚动、字体渲染不可接受。

## 6. 技术落地要点（后续细化）

- **不要在主线程跑 SSH**：`ssh2` 是阻塞 API，需放进专用线程/任务，用 channel 回传。
- **二进制数据走 raw channel**：终端数据用 `tauri::ipc::Channel` 或自定义协议传输，避免 base64/JSON 膨胀。
- **会话与通道分离**：一个 TCP 连接可挂多个 channel（shell + sftp），减少重复认证。
- **命令层薄**：`connect / open_sftp / list_dir / upload / download / open_shell / resize` 等少量命令 + 事件流。
- **凭据**：密码与私钥口令存 Keychain（`security-framework` crate 或 Tauri stronghold），不落 JSON。
- **传输引擎独立于 UI**：队列、并发、断点在 Rust 侧实现，UI 只订阅进度事件。
