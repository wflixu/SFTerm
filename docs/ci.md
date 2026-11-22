# CI / CD 与发布

- **状态**：随 0.1.0 落地
- **日期**：2026-11-22
- **相关**：[roadmap](roadmap.md) · [README](../README.md)

---

## 流水线

| 工作流 | 触发 | 做什么 |
|---|---|---|
| [CI](../.github/workflows/ci.yml) | PR、手动 | `pnpm build`（类型检查 + 构建）、Vitest、`cargo test -p sfterm-core`、`cargo check -p app` |
| [Release](../.github/workflows/release.yml) | 推送 `v*` tag、手动 | 构建 macOS arm64 的 `.app` 与 `.dmg`，签名、公证，发 GitHub Release |

两个工作流都跑在 `macos-15`（arm64）上。macOS 14 的 runner 已在 2026-11-02 退役。

## 发布产物

每次发版上传三个文件：

```
SFTerm_<version>_arm64.dmg        # 推荐给用户下载
SFTerm_<version>_arm64.app.zip    # 直接要 .app 的人用
SHA256SUMS.txt
```

- **只有 arm64**：`tauri build --target aarch64-apple-darwin`，没有 Intel 版本，也不做 universal。
- **系统要求 macOS 15.0+**：配置在 `bundle.macOS.minimumSystemVersion`。

## 发布步骤

```bash
# 1. 改版本：package.json 和 src-tauri/tauri.conf.json 必须一致
# 2. 提交
git commit -am "chore(release): v0.2.0"
# 3. 先推分支，再打 tag 推送（tag 必须是 v + 版本号）
git push
git tag v0.2.0
git push origin --tags
```

工作流会先校验 `tag == v<tauri.conf.json.version>`，不一致直接失败，不会产出半成品。

## 需要的 GitHub Secrets

仓库 → Settings → Secrets and variables → Actions → **Repository secrets** → New repository secret。

用 **Repository secrets**，不要用 Environment secrets：两个工作流都没有声明 `environment:`，Environment secrets 在未引用对应环境的 job 里读不到，会表现为「secret 全是空字符串」。除非以后给发版 job 加环境审批，才需要改成 `environment: release` + Environment secrets。

签名与公证都建议配置：不签名用户第一次打开会被 Gatekeeper 拦，不公证则联网校验时仍可能提示。工作流允许两者缺失，但会打印警告。

### 签名

| Secret | 内容 |
|---|---|
| `APPLE_CERTIFICATE` | `Developer ID Application` 证书导出的 `.p12`，再转成单行 base64 |
| `APPLE_CERTIFICATE_PASSWORD` | 导出 `.p12` 时设置的密码 |
| `APPLE_SIGNING_IDENTITY` | 证书名，形如 `Developer ID Application: 你的名字 (TEAMID)` |

证书怎么导出见下一节。

### 公证：App Store Connect API Key（推荐）

CI 里不用 Apple ID，改用 App Store Connect 的 API Key，不会因为账号开双重认证或在别的机器登录而失效。

| Secret | 内容 |
|---|---|
| `APPLE_API_ISSUER` | Issuer ID，UUID 形式 |
| `APPLE_API_KEY` | Key ID，10 位 |
| `APPLE_API_KEY_P8` | `.p8` 私钥文件的**全文** |

工作流只把**非空**的 secret 导出成环境变量——GitHub 会把未设置的 secret 变成空字符串，而 Tauri 用「变量是否存在」判断凭据，空串会被误判成已配置，从而走错公证分支。

配置好之后工作流会：

1. 把 `APPLE_API_KEY_P8` 写到 `$RUNNER_TEMP/AuthKey.p8`；
2. 导出 `APPLE_API_KEY_PATH` 给 Tauri；
3. `tauri build` 自动提交公证并 staple，日志里能看到 `公证：使用 App Store Connect API Key`。

**创建步骤**

1. 打开 [App Store Connect](https://appstoreconnect.apple.com) → 用户和访问（Users and Access）→ 集成（Integrations）→ **App Store Connect API** → 密钥（Keys）→ **+**。
2. 名称随意（比如 `sfterm-ci`），角色选 **Developer** 就够提交公证；创建密钥这个动作本身需要 Admin 或 Account Holder。
3. 生成后记下 **Key ID**（10 位），页面顶部还有 **Issuer ID**（UUID）。
4. 下载 `.p8` 文件——**只能下载一次**，存到安全的地方。
5. 在 GitHub 仓库添加三个 Secret：
   - `APPLE_API_ISSUER` = Issuer ID
   - `APPLE_API_KEY` = Key ID
   - `APPLE_API_KEY_P8` = 用文本编辑器打开 `.p8`，整份内容粘进去（包含 `-----BEGIN PRIVATE KEY-----` 和 `-----END PRIVATE KEY-----` 两行）。
     不要转 base64，也不要只粘中间那段。
6. 推一个 tag，构建日志里确认打印的是「公证：使用 App Store Connect API Key」。

密钥丢了只能撤销重建；如果怀疑泄露，立刻在 App Store Connect 里 Revoke。

### 公证备选：Apple ID + App 专用密码

个人机器上更简单，但账号开启双重认证、改密码或异地登录时可能失效。

| Secret | 内容 |
|---|---|
| `APPLE_ID` | Apple Developer 账号邮箱 |
| `APPLE_PASSWORD` | **App 专用密码**，不是账号登录密码 |
| `APPLE_TEAM_ID` | 10 位 Team ID（developer.apple.com → Membership 里看） |

生成专用密码：[appleid.apple.com](https://appleid.apple.com) → 登录与安全 → App 专用密码 → 生成，得到 `xxxx-xxxx-xxxx-xxxx`。

两套公证凭据同时存在时，工作流优先用 API Key。

## 怎么导出签名证书

1. 钥匙串访问 → 证书助理 → 从证书颁发机构请求证书，存到磁盘（得到 `.certSigningRequest`）。
2. developer.apple.com → Certificates, Identifiers & Profiles → Certificates → **+** → 选 **Developer ID Application**（只有 Account Holder 能创建），上传 CSR，下载 `.cer`。
3. 双击 `.cer` 导入「登录」钥匙串。`.cer` 只有证书、没有私钥，必须从钥匙串里把 **证书 + 私钥** 一起导出成 `.p12`：钥匙串访问 → 登录 → 我的证书 → 展开目标证书 → 右键「导出…」→ 格式选 Personal Information Exchange (.p12) → 设置密码。
   如果钥匙串里有多个同名证书（比如新旧两张 Developer ID Application），先用「显示简介」核对序列号与到期时间，别导错。
4. 转 base64：

   ```bash
   openssl base64 -A -in DeveloperID.p12 -out certificate-base64.txt
   ```

   把 `certificate-base64.txt` 的整行内容填进 `APPLE_CERTIFICATE`，`.p12` 密码填进 `APPLE_CERTIFICATE_PASSWORD`。

5. 查 identity：

   ```bash
   security find-identity -v -p codesigning
   ```

   复制 `Developer ID Application: ... (TEAMID)` 整串填进 `APPLE_SIGNING_IDENTITY`。

## 签名是可选的，但不建议省

如果三个签名 secret 都没配，工作流会打印警告并产出未签名的包：能装、能被开发者手动放行，但用户第一次打开会遇到「无法验证开发者」。公证凭据没配则会签名但不公证，Gatekeeper 仍可能提示。

本地开发不受影响：签名只在 `tauri build` 时发生，`pnpm tauri dev` 不需要证书。

## 常见失败

| 现象 | 原因 |
|---|---|
| `certificate ... does not match provided identity` | `APPLE_SIGNING_IDENTITY` 与 `.p12` 不是同一张证书 |
| 公证报 401 / 认证失败 | API Key 的 Issuer ID / Key ID 与 `.p8` 不匹配，或密钥已被撤销；用 Apple ID 方式时则是用了账号密码而不是 App 专用密码 |
| 公证报权限不足 | API Key 的角色低于 Developer，重新建一个 Developer 或 Admin 角色的密钥 |
| 日志里没有「公证」字样 | 三个 API Key secret 名字拼错，或 `.p8` 内容被截断；工作流会打印实际走的是哪种方式 |
| `spctl` 校验不过 | 没签名、或公证还没 staple 完；看构建日志里的 notarization 输出 |
| tag 校验失败 | tag 不是 `v<version>`，或 `package.json` 与 `tauri.conf.json` 版本不一致 |
| 上传 `.app` 后签名失效 | 用 `zip` 压的；必须用 `ditto -c -k --keepParent`（工作流已处理） |

## Bundle identifier

`bundle.identifier` 现在是 `cn.wflixu.sfterm`。它决定 macOS 上的应用身份与应用数据目录：

- 改它等于换一个 App：WebView 的 localStorage（主机、密钥）不会跟着走。
- 第一次发版后尽量不要再改；真要改，得配合导出 / 导入或 0.2.0 的存储迁移。

Tauri CLI 会拒绝用默认的 `com.tauri.dev` 打包，所以这一项是发版的硬性前提。

## CI 耗时说明

0.1.0 第一次跑 CI 的实测（macos-15 arm64，无排队）：

| 阶段 | 耗时 |
|---|---|
| 前端安装 + 类型检查/构建 + Vitest | ~22s |
| 安装 Rust 1.98.1 | ~7s |
| `cargo test -p sfterm-core` | 91s |
| `cargo check -p app` | 147s |
| 保存 Rust 缓存 | ~20s |
| **合计** | **约 5 分钟** |

几个容易误判的点：

- Actions 页面上的「总时长」包含排队。macOS arm64 runner 有容量限制时会先排队，排队时间不算在 job 里，但你会感觉「跑了 10 分钟」。
- 冷启动慢的主因是 Rust 依赖要编两遍：以前 `crates/core` 和 `src-tauri` 各有一个 `Cargo.lock`、各自的 `target/`，vendored OpenSSL 也要编两次。现在根目录是 workspace，共享一个 `target/` 和 `Cargo.lock`，依赖只编一次。
- `Swatinem/rust-cache` 命中后，第二次开始主要只剩增量编译，通常能压到 1–2 分钟。

`tauri.conf.json`、`Cargo.toml` 或依赖变更会让缓存失效，那一次会回到冷启动耗时。
