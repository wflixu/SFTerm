# SFTerm 架构说明

- **状态**：与 `v0.1.0` 代码同步
- **日期**：2026-11-22
- **范围**：前端页面 / Pinia / Tauri 命令 / Rust 引擎 / 线程与事件模型

---

## 1. 总览

```
┌───────────────────────────── Vue 3（WebView）─────────────────────────────┐
│  views: Home / SFTP / Terminal      components: DualPane / FilePane / …   │
│  stores: system · keychain · sftp · tabs · terminals · transfers          │
│  router: /  ·  /sftp  ·  /terminal/:hostId                                │
└───────────────▲───────────────────────────────────┬───────────────────────┘
        event: transfer-progress / shell-output     │ invoke(command)
┌───────────────┴───────────────────────────────────▼───────────────────────┐
│ Tauri 壳（src-tauri/src/cmds.rs）：参数校验、事件转发、句柄表               │
│   Manager<Arc> 会话管理          Shells<Arc<Mutex<HashMap<id, ShellHandle>>>> │
└───────────────▲───────────────────────────────────┬───────────────────────┘
                │ mpsc::Sender<Request>             │ mpsc::Sender<ShellCommand>
┌───────────────┴───────────────────────────────────▼───────────────────────┐
│ Rust 引擎（crates/core，独立 crate，可用 engine-check CLI 直接跑）          │
│   每个 SFTP 会话 1 个线程：Session + Sftp（阻塞 API，串行处理请求）          │
│   每个终端     1 个线程：独立 Session + PTY channel（非阻塞轮询）            │
└───────────────────────────────────────────────────────────────────────────┘
```

设计前提：`ssh2`（libssh2）是同步阻塞 API，且同一个 `Session` 不能被多个线程同时读写——**所有 SSH/SFTP 调用都必须回到会话所属的那条线程**。整个架构围绕这条约束展开。

## 2. 线程模型

### 2.1 SFTP 会话线程

- 前端 `ssh_connect` → `Manager::connect()` 建立 TCP + 握手 + 认证，随后 spawn 一条会话线程（`sfterm-ssh-<id>`）；线程启动后才打开 SFTP 子系统，并独占 `Session` 与 `Sftp`。SFTP 子系统打不开时，线程仍会消费后续请求并一律回错，避免调用方永久阻塞。
- `Manager` 里只保存 `session_id → Sender<Request>`，锁的持有时间只有一次 HashMap 取值。
- 每次调用（列目录、上传、下载、mkdir、rename、delete、realpath）都构造一个 `Request`，附带一条**一次性回复 channel**，线程处理完把结果发回去，调用方 `recv()` 等待。
- 结果：同一会话的 SFTP 操作天然串行，不需要额外加锁，也不会撞 libssh2 的“同会话不可并发”限制。

```rust
// 简化后的调用链
manager.list_dir(id, path)                  // UI 侧（spawn_blocking）
  → tx.send(Request::ListDir { reply })     // 进会话队列
  → 会话线程：sftp.readdir(path)            // 真正阻塞的地方
  → reply.send(Ok(entries))
  → 调用方 recv() 拿到结果
```

### 2.2 终端线程

- **每个终端标签一条独立 SSH 会话**，不复用 SFTP 会话。原因：libssh2 的阻塞 `read()` 会锁住整个会话，终端一边等输入一边就会让文件操作全部卡死。
- 建通道、`request_pty`、`shell()` 必须**在阻塞模式下**完成；完成后再 `set_blocking(false)`，否则这些调用会立刻返回 `WouldBlock`。
- 之后进入循环：`try_recv` 处理写入 / resize / 关闭 → `write` 发送按键 → `read` 收输出 → `thread::sleep(5ms)`。输出通过回调经 `shell-output` 事件推给前端。
- `ShellHandle` 是前端能拿到的唯一句柄，只暴露 `write` / `resize` / `close` 三个投递方法。

### 2.3 传输线程与进度

- `sftp_upload` / `sftp_download` 在 `spawn_blocking` 里再起一条传输线程，主线程只负责把 `TransferProgress` 转成 `transfer-progress` 事件。
- 进度事件按 `id`（前端生成的传输任务 id）匹配，前端 `transfers` store 更新进度条；命令返回即表示传输结束（成功或失败）。

## 3. 命令与事件

命令定义在 [cmds.rs](../src-tauri/src/cmds.rs)，能力实现在 [crates/core/src/lib.rs](../crates/core/src/lib.rs)。

| 命令 | 作用 | 备注 |
|---|---|---|
| `ssh_connect` | 建连 + 认证，启动会话线程（SFTP 在会话线程里打开） | 返回 `ConnectInfo`（banner、主机密钥类型、SHA256 指纹） |
| `secret_set` / `secret_get` / `secret_delete` | 系统钥匙串读写删 | 服务名 `cn.wflixu.sfterm`，account 由前端拼（`host:<id>:password`、`key:<id>:private`） |
| `ssh_disconnect` / `ssh_sessions` | 断开 / 列出会话 | 调试用 |
| `sftp_home` | 远端 `realpath(".")` | 登录后的默认目录；失败时前端回退 `/home` |
| `sftp_list_dir` | 列远端目录 | 返回 `perm`（数字 mode）、大小、mtime、符号链接标记；UI 暂未显示权限串 |
| `sftp_upload` / `sftp_download` | 单文件传输 | `overwrite` 决定是否覆盖；进度走事件 |
| `sftp_mkdir` / `sftp_rename` / `sftp_remove` | 远端写操作 | 删除目录递归 |
| `fs_list_local` / `fs_home` | 本地文件系统 | 只读，`fs_home` = `~/` |
| `shell_open` / `shell_write` / `shell_resize` / `shell_close` | 终端 | 输入输出都以 base64 传输 |

| 事件 | 负载 | 消费方 |
|---|---|---|
| `transfer-progress` | `{ id, done, total }` | [App.vue](../src/App.vue) → `transfers.update` |
| `shell-output` | `{ id, data(base64) }` | [App.vue](../src/App.vue) → `terminals.receive` |

事件在 `App.vue` 里只订阅一次，再分发到各 store；页面组件不直接 `listen`，避免重复订阅。

## 4. 前端状态模型

| store | 职责 | 生命周期 |
|---|---|---|
| `system` | 主机列表（增删改查）、加载时补齐缺失 / 重复 id | localStorage |
| `keychain` | 密钥条目（Label / 私钥 / 公钥） | localStorage |
| `sftp` | SFTP 页当前绑定的**主机对象**（不是 id） | 内存，重启回到未绑定 |
| `tabs` | 终端标签：`terminalId = term-${address}:${port ?? 22}`，按 IP:端口去重 | 内存 |
| `terminals` | 终端会话缓存：`shellId`、状态、输出块、写入游标 | 内存，切换标签不销毁 |
| `transfers` | 传输任务列表与进度 | 内存 |

### 4.1 标签与连接

- 顶层只有三类页面：`/`（Home，常驻）、`/sftp`（常驻）、`/terminal/:hostId`（每个主机一个）。
- 双击主机卡片 → `tabs.openTerminal(host)`；同一 `IP:端口` 已存在标签时直接复用，不重复建连。
- 标签页是纯路由状态，真正的 shell 会话在 `terminals` store 里；离开终端页只卸载 xterm 实例，**不关闭 shell**。
- 回到标签时，`TerminalView` 从 `written` 游标续写缓存块（上限 512 KB），所以切回来能看到离开期间的输出。

### 4.2 SFTP 页

- `sftp.bound` 存的是主机对象而不是 id——历史数据里存在没有 `id` 的主机，按 id 查会查不到。
- 绑定后 `DualPane` 自动建连、列本地 `~/`、列远端 `realpath(".")`；切换主机时靠 `:key` 强制重建面板。
- 本地栏只读（决策 D5），写操作只出现在远端栏的右键菜单里。

## 5. 一次操作的完整路径

**打开终端**

1. `HostCard` 双击 → `tabs.openTerminal(host)` 得到（或复用）标签 → `router.push('/terminal/:hostId')`
2. `TerminalView` 挂载 → `terminals.open(shellId, params, cols, rows)` → `invoke('shell_open')`
3. Rust 建连、起 PTY、起轮询线程 → 输出经 `shell-output` 回传
4. `terminals.receive` 解码 base64 追加到 `chunks`，`TerminalView` 的 watcher 把新块写进 xterm

**上传文件**

1. `DualPane` 选中本地文件 → 检查远端同名 → 有冲突先弹「覆盖 / 新建副本 / 跳过」
2. `transfers.add(task)` → `invoke('sftp_upload', { id, session_id, ... })`
3. Rust 传输线程 → 进度事件 → `transfers.update`；结束后命令 resolve / reject → `finish` / `fail`
4. 远端列表刷新

## 6. 已知约束与技术债

| 项 | 现状 | 影响 |
|---|---|---|
| 终端数据 base64 | 输入输出各编码一次 | 大量输出（`yes`、`cat` 大文件）时 CPU 与内存翻倍 |
| 终端轮询 | 固定 5ms sleep | 空闲时也唤醒，200Hz 空转；可改事件驱动 / 自适应退避 |
| 无背压 | 输出块直接堆在 store（512 KB 上限） | 超限会丢早期输出 |
| 死亡会话不感知 | shell 进程退出后前端仍显示「已连接」 | 需要 exit 事件与状态回收 |
| 传输不可取消 | 没有取消通道 | 只能等传完或退出标签 |
| 存储 | 密码、私钥已在系统钥匙串；主机与公钥等元数据仍在 localStorage | SQLite 持久层还没做，见 [roadmap](roadmap.md) |
| 终端历史 | 输出只在内存（512 KB），重启即丢；没有命令边界 | 0.2.0 做命令历史，边界方案见 [roadmap](roadmap.md) |
| 主机密钥 | 只显示指纹，无 TOFU 确认 | 中间人无法被拦截（个人自用可接受） |
| 目录传输 | 只支持单文件 | 递归需要引擎侧遍历 + 队列 |
