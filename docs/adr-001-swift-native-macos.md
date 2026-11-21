# ADR-001：SFTerm 转为 macOS 原生 Swift 应用（libghostty + Citadel）

- **状态**：**已被 [ADR-002](adr-002-stay-on-tauri.md) 取代（Superseded）**
- **日期**：2026-11-21
- **取代原因**：产品定位改为「SFTP 优先、终端为辅」，终端保真度不再是选型主变量；App Store 沙盒约束对两种技术栈同等成立；协议层（密钥 + 密码 + SFTP）在 Rust 侧反而更成熟。
- **决策者**：lx
- **替代对象**：当前 Tauri 2 + Vue 3 + xterm.js 原型（`src/`、`src-tauri/`）
- **关联**：[README.md](../README.md)

---

## 1. 背景

当前仓库是 Tauri 桌面应用原型：Vue 3 + PrimeVue + xterm.js 做界面，Rust 侧只有一个占位命令 `ssh_connect`（[src-tauri/src/cmds.rs](../src-tauri/src/cmds.rs)），硬编码空 ip / 空私钥路径，SSH 通路尚未打通；主机 CRUD 走 localStorage（[src/stores/system.ts](../src/stores/system.ts)）。

触发这次重新选型的是 Ghostty 生态的两个新事实：

1. Ghostty 作者 Mitchell Hashimoto 把终端仿真内核抽成了 **libghostty**，1.0 于 2026-11-06 发布（MIT，C 库，开始按 semver 发版）。它**只做 VT 解析与终端状态**，不含 PTY、网络、SSH/SFTP。
2. Rex 是 Superlogical 的闭源终端 + 多路复用器，其"远程登录"走自家 rex server 的真实系统登录，**不是 SSH**，没有可复用代码。能复用的只有 libghostty 这一层。

在 Apple 平台上承载 libghostty 的是第三方封装 [GhosttyKit / libghostty-spm](https://github.com/Lakr233/libghostty-spm)（MIT）：静态 XCFramework，含 Metal 渲染、CoreText 字体、键盘/鼠标/IME、文本选择、搜索、485 套主题，并额外提供 **host-managed I/O 后端**（`GHOSTTY_SURFACE_IO_BACKEND_HOST_MANAGED`），即"宿主自己提供字节流和尺寸"的模式——正好对应"字节来自 SSH 而不是本地 PTY"。

### 本次拍板的约束

| 编号 | 决策 | 直接含义 |
|---|---|---|
| D1 | **只做 macOS** | 放弃 Windows/Linux；UI 栈定为 SwiftUI + AppKit 互操作 |
| D2 | **只支持最新算法** | 不做 RSA 用户密钥 / ssh-rsa / DH-group14 / AES-CTR 兼容；不支持老设备 |
| D3 | **未来上架 Mac App Store** | 必须 App Sandbox；不能读 `~/.ssh`、不能用 ssh-agent、不能 spawn `ssh` |
| D4 | **接受 GhosttyKit 绑定 Ghostty main** | 周一跟版、周更二进制 XCFramework；需要 pin + 自建 fork 兜底能力 |
| D5 | **不考虑 MFA** | 不实现 keyboard-interactive（RFC 4256）；只支持 password 与 publickey（含带口令私钥） |

---

## 2. 决策

采用**纯 Swift 原生**方案，新建 macOS 应用工程，现有 Tauri 代码转为参考实现。

| 层 | 选型 | 版本策略 |
|---|---|---|
| 终端视图 | GhosttyKit（`libghostty-spm` 的 `GhosttyTerminal` 产品） | 精确 pin（`from: "1.5.2"` 起步），经自有 `TerminalBackend` 协议隔离 |
| SSH 传输 | [Citadel](https://github.com/orlandos-nl/Citadel)（MIT）over [SwiftNIO SSH](https://github.com/apple/swift-nio-ssh)（Apache-2.0） | pin；fork 一份到自有 org 以防上游失修 |
| SFTP | Citadel `SFTPClient` 起步 + **自有传输引擎**（队列/pipeline/断点/原子 rename） | 同上 |
| 密钥 | 应用内生成 ed25519（swift-crypto）、导入 OpenSSH/PEM；口令进 Keychain | — |
| 持久化 | 主机/标签/历史 → SQLite（GRDB 或 SwiftData）；密钥 → Keychain；全在 App 容器内 | — |
| 分发 | Mac App Store 为主；同一代码库出 Developer ID 版 | — |

### 明确否决的方案

| 方案 | 否决理由 |
|---|---|
| [Traversio](https://github.com/GitSwiftHQ/Traversio) | 功能最全（PTY、SFTP、SCP、ProxyJump、keyboard-interactive、agent），但 **AGPL-3.0 或商业授权**：AGPL 与闭源 + App Store 分发冲突，等于必须买商业授权；且 11★、2026-05 建仓、1.0.x 早期。买授权后可复议。 |
| 自研 SSH 传输层 | KEX、算法协商、包加密、时序侧信道全在安全红线上；Ghostty 自己都没做 SSH。 |
| Rust 核心（russh）+ SwiftUI 壳 | 两套工具链、静态库公证、跨语言崩溃边界；省下的协议工作被工程复杂度吃掉。 |
| 继续 Tauri + `russh`/`russh-sftp` | 协议栈最成熟、跨平台，但终端渲染与原生手感上限低，且与 D1（只做 macOS）冲突。**保留为备选路线。** |

---

## 3. 决策理由

1. **终端这一层买到天花板**：Ghostty 的 VT 兼容性、Metal 渲染、IME、搜索、主题是长期积累，自研或 xterm.js 都难追。libghostty 是 MIT，闭源商用可嵌入。
2. **接缝天然匹配**：GhosttyKit 的 host-managed 后端本来就是为"非 PTY、沙盒安全"设计的，SSH 字节流直接喂进去，终端与网络彻底解耦。
3. **Citadel 省掉 2–4 周**：SFTP 客户端、OpenSSH 私钥解析、连接池、jump host 都是现成的（`Sources/Citadel/SFTP/*`、`OpenSSHKey.swift`、`SSHConnectionPoolSettings.swift`）。
4. **许可证干净**：Ghostty/libghostty-spm MIT、Citadel MIT、SwiftNIO SSH Apache-2.0，闭源 + App Store 均无障碍（Ghostty 上游的 bash/zsh shell-integration 是 GPLv3，但 GhosttyKit 自带 MIT 版本规避，见其 README）。
5. **单语言**：Swift 一处调试，无 FFI、无 JS bridge、无 IPC 序列化。

---

## 4. 后果

### 正面

- 终端体验直接对标 Ghostty；窗口、输入法、选择、搜索、主题全部免费获得。
- 单一语言、单一构建系统；沙盒模型干净（无子进程、无非沙盒文件访问）。
- 未来 iOS/iPadOS 复用大部分非 UI 代码（Citadel/GhosttyKit 都支持）。

### 负面（必须接受）

| 代价 | 来源 | 影响 |
|---|---|---|
| 无法用 RSA 用户密钥登录 | D2 | 用户拿 `id_rsa` 会失败；必须明确提示并引导生成 ed25519 |
| 无法连只提供 RSA 主机密钥的服务器 | D2 | 少数网络设备/老镜像连不上；需在错误文案里给出可诊断信息 |
| **不做 keyboard-interactive** | D5 | 已明确排除。残留边缘情况：少数服务器把普通密码也包在 keyboard-interactive 里（`PasswordAuthentication no` + 仅 PAM），这类会连不上，需给出可读错误 |
| 无 ssh-agent、无 forwarding | D3 沙盒 | 与"只做现代算法"其实自洽，但企业流水线用户会痛 |
| 无 `~/.ssh`、无 `ssh_config`、无 `known_hosts` 复用 | D3 沙盒 | 密钥必须导入/新建；已知主机库自建；ProxyJump 需手填 |
| 依赖第三方周更二进制包 | D4 | API 可能破坏；App Store 需要可复现构建；包体积增加 |
| 只有 macOS | D1 | 失去跨平台用户 |

### 范围外（v1 明确不做）

- keyboard-interactive / MFA（D5）
- RSA 用户密钥、`ssh-rsa`、DH-group14、AES-CTR 等旧算法（D2）
- ssh-agent / agent forwarding、X11 forwarding、FIDO security key（D3 沙盒 + 范围）
- 直接读取 `~/.ssh`、解析 `ssh_config`（D3 沙盒）
- 本地 PTY / 本地 shell（产品只做远程会话）
- 私钥云同步（主机列表可同步，私钥不同步）

---

## 5. 风险登记

| # | 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| R1 | ~~MFA/keyboard-interactive 缺失~~ → **范围外（D5）** | — | — | 不实现。仅在认证失败时识别服务器给出的 `keyboard-interactive` 方法，提示"该服务器要求交互式验证，当前版本不支持"，避免被误判为密码错误 |
| R2 | Citadel 失修（最近提交 2026-06，35 open issues） | 中 | 中 | 立即 fork 到自有 org；只用其稳定面；SFTP 层准备自研替换 |
| R3 | SFTP 吞吐不达标（v3 单请求、窗口小、无 pipeline） | 中 | 中 | 自研 `TransferEngine`：并发请求、read-ahead、大包、`limits@openssh.com` 协商 |
| R4 | GhosttyKit 周更破坏 API | 中 | 中 | 精确 pin tag；隔离在 `TerminalBackend` 协议后；必要时自建 XCFramework |
| R5 | App Store 审核/沙盒踩坑（隐私清单、二进制体积、权限文案） | 中 | 中 | 第一天就开沙盒构建；提交前跑 `privacy manifest` 清单；参考 Secure ShellFish 的可行形态 |
| R6 | 主机密钥校验做错导致 MITM | 低 | 极高 | 默认严格校验；首次连接必须指纹确认；不做 `acceptAnything()` |
| R7 | 长连接内存/句柄泄漏（每 tab 一个 channel） | 中 | 中 | 连接池 + channel 生命周期测试 + 压力用例 |

---

## 6. 目标架构

```
SwiftUI 界面层
├── HostsView / HostEditView          主机 CRUD（对应现有 HostView.vue、HostEdit.vue）
├── TerminalTabsView                  多标签
├── SFTPBrowserView                   双栏 / 队列 / 权限
└── SettingsView                      主题、算法、密钥、隐私
        │
Application 层（actor，无 UI 依赖）
├── SessionManager                    连接生命周期、重连、标签状态
├── HostStore                         SQLite
├── Vault                             Keychain（口令 / 私钥口令 / 生物识别 ACL）
└── TransferEngine                    SFTP 队列、并发、断点、校验
        │
Transport 层（协议抽象，可替换）
├── TerminalBackend  ← GhosttyKit 适配（TerminalSurfaceOptions.backend = .inMemory(session)）
│      write(data)  → SSH channel stdin
│      resize(cols, rows, px) → SSH window-change
└── SSHTransport     ← Citadel 适配
       ├── ShellChannel   PTY + shell + resize + 信号
       ├── SFTPChannel    subsystem sftp（独立 channel，复用同一 TCP 连接）
       └── HostKeyVerifier
```

关键数据流：

```
用户按键 → GhosttyTerminal 编码（含 kitty/modifyOtherKeys 模式）
        → InMemoryTerminalSession.write(bytes)
        → SSHChannelWriter → 远端
远端输出 → SSH channel handler → terminal.feed(bytes) → Metal 渲染
窗口尺寸变化 → resize 回调 → SSHChannelRequestEvent.WindowChangeRequest
```

### 集成细节与坑（GhosttyKit + SSH 特有）

1. **TERM 与 terminfo**：GhosttyKit 自带 `xterm-ghostty` terminfo，但**远端服务器没有**。SSH 请求 PTY 时应发 `TERM=xterm-256color`（默认），或提供"在远端安装 terminfo"动作；否则会出现 `terminal not found` 类报错。
2. **OSC 133 shell integration**：GhosttyKit 的"跳转上/下一个 prompt"依赖 OSC 133 标记。`.exec` 后端由包内 MIT 脚本自动注入；**host-managed（SSH）后端不会**，需要远端 shell 自身输出 OSC 133（可在连接时可选注入一段 bash/zsh 片段）。
3. **像素尺寸**：PTY 请求除行列外还有像素宽高，部分程序（图像协议）依赖；从 `resize` 回调把 px 一起带上。
4. **剪贴板**：OSC 52 与粘贴保护要走 `TerminalViewState.onClipboardConfirmationRequest`，沙盒下也必须由宿主决定放行，不能静默。
5. **链接打开**：macOS 上 Ghostty 自带的 opener 会拒绝终端里识别出的任意 URL，需要宿主实现 `onOpenURL` 策略（只放行 http/https/mailto）。
6. **粘贴/拖拽文件**：落盘走 `TerminalFileStaging`，沙盒下注意容器路径，退出时清理。

---

## 7. 里程碑

| 里程碑 | 内容 | 预估 | 出口标准 |
|---|---|---|---|
| M0 | 技术验证 spike | 1 周 | 见 §8，全部通过 |
| M1 | 连接 + 终端 | 3 周 | 多标签、ed25519/密码/带口令私钥、known_hosts、vim/htop/resize 正常 |
| M2 | SFTP 引擎 | 3 周 | 1GB 文件 ≥ 40MB/s（局域网）、1 万目录 < 2s、断点续传、原子 rename |
| M3 | 产品 UI | 4 周 | 主机管理、双栏浏览、传输队列、权限编辑、密钥管理 |
| M4 | 上架硬化 | 2 周 | 沙盒全功能、隐私清单、公证 + App Store 提交包、崩溃/日志接入 |
| **合计** | | **约 13 周** | 1.0 beta |

合计不含 MFA（D5）。若未来范围变更需要 keyboard-interactive，追加 2–4 周。

---

## 8. M0 spike 任务与验收（不写产品代码）

| 天 | 任务 | 通过标准 |
|---|---|---|
| D1 | GhosttyKit 集成（SPM 拉取、窗口、输入） | 中文、IME、真彩色、滚动、搜索、主题切换正常；`InMemoryTerminalSession` 能回显本地字节 |
| D2 | Citadel 连接真机 | ed25519 / 密码 / 带口令私钥三种认证成功；错误口令有可读错误；known_hosts 首连弹指纹、二次静默通过 |
| D3 | PTY 双向 + resize | `vim`、`htop`、`tmux` 不错位；拖动窗口后 `stty size` 跟随；Ctrl-C/信号正常 |
| D4 | SFTP 压测 | 1GB 上下行吞吐、1 万文件 `listDirectory` 延迟、1000 个并发小文件请求不崩、断点续传可实现 |
| D5 | 沙盒化 + 边界用例 | 开 App Sandbox 后网络与 Keychain 正常；通过 `NSOpenPanel` 导入私钥可用；找一台**仅 keyboard-interactive** 的服务器和一台 **RSA-only** 的服务器各连一次，确认失败文案可读、不误导为密码错误 |

### Kill criteria（任一条成立则回退到 Tauri + Rust 路线）

- D3 的 PTY 交互存在不可修复的错位/丢字节；
- D4 的 SFTP 吞吐在局域网 < 10MB/s 且无优化路径；
- D5 发现 App Sandbox 下无法建立 SSH 连接（概率极低，但要实测）；
- GhosttyKit 在沙盒内无法渲染（Metal + 沙盒冲突）。

---

## 9. 未决问题

1. ~~keyboard-interactive（MFA）是否 v1 必须？~~ **已决策：范围外（D5）**。若未来范围变更，需给 NIOSSH 实现 RFC 4256（约 2–4 周）并新增多轮问答 UI。
2. **RSA 用户密钥**：D2 已排除，但建议在导入界面显式识别 RSA 并提示"请改用 ed25519"，而不是连接时才报错。
3. **是否同时发 Developer ID 版**（非沙盒）：可获得 `~/.ssh`、ssh-agent、`ssh_config` 能力，但两套能力矩阵的测试成本要算进去。建议 v1 只做沙盒版，保持单一实现。
4. **密钥是否上云同步**：主机列表可 iCloud 同步；私钥建议不同步，或仅同步 Secure Enclave 包裹后的密文。

---

## 10. 参考

- libghostty 1.0 发布：https://mitchellh.com/writing/libghostty-1-0
- GhosttyKit（libghostty-spm）：https://github.com/Lakr233/libghostty-spm
- SwiftNIO SSH：https://github.com/apple/swift-nio-ssh
- Citadel：https://github.com/orlandos-nl/Citadel
- Traversio（AGPL / 商业授权）：https://github.com/GitSwiftHQ/Traversio
- Rex 定位（非 SSH、闭源）：https://www.stork.ai/blog/rex-wants-to-make-tmux-obsolete
- App Store 上的原生 SSH/SFTP 先例：https://apps.apple.com/us/app/ssh-files-secure-shellfish/id1336634154
