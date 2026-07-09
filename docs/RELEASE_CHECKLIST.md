# 发布检查清单

> 提供两套发布方式，任选其一：
> - **A. CI 自动发布**（推荐）— 推送 tag，GitHub Actions 自动构建 + 上传 + 更新元数据
> - **B. 纯手动发布**（备用）— 本地构建，手动上传

---

## 发布方式选择

- [ ] 确定本次发布方式：**A. CI 自动** / **B. 纯手动**
  - CI 自动：只需跑 [A 流程](#a-ci-自动发布推荐)
  - 纯手动：需跑 [B 流程](#b-纯手动发布)

---

## 通用 — 发布前准备

- [ ] 确认 `~/.tauri/updater.key` 密钥存在且有效
- [ ] 确认公钥已填入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`

---

# A. CI 自动发布（推荐）

## A1 — 更新版本号

**三处同步修改：**

| 文件 | 当前值示例 | 改为 |
|---|---|---|
| `src-tauri/Cargo.toml` | `version = "0.2.0"` | `version = "0.3.0"` |
| `src-tauri/tauri.conf.json` | `"version": "0.2.0"` | `"version": "0.3.0"` |
| `package.json` | `"version": "0.2.0"` | `"version": "0.3.0"` |

- [ ] `Cargo.toml` 版本号已更新
- [ ] `tauri.conf.json` 版本号已更新
- [ ] `package.json` 版本号已更新

## A2 — 提交并推送 Tag

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json
git commit -m "chore: bump version to v0.3.0"
git tag v0.3.0
# 推送 tag 到 Gitee（GitHub 会自动同步）
git push origin master --tags
```

- [ ] 版本号已提交
- [ ] Tag 已推送

## A3 — 等待 CI 完成

推送 tag 后，GitHub Actions 自动执行：

1. 打开 https://github.com/hhyd/files-explorer/actions 查看进度
2. CI 会依次：
   - 创建 Gitee Release
   - 构建 macOS Intel / macOS ARM / Windows / Linux 四个平台的安装包
   - 上传安装包到 Gitee Releases + GitHub Releases
   - 收集签名 → 更新 `updater.json` → 提交到 Gitee
3. 全部完成约需 **15–25 分钟**

- [ ] 四平台构建全部绿色（✅）
- [ ] Gitee Releases 已有安装包
- [ ] GitHub Releases 已有安装包
- [ ] `updater.json` 已自动更新

## A4 — 验证

- [ ] 打开旧版本，触发检查更新
- [ ] 确认弹出 v0.3.0 更新提示
- [ ] 确认更新内容正确
- [ ] 确认安装后新版本正常运行

> ✅ **CI 发布完成，无需手动上传。**

---

# B. 纯手动发布（备用）

> 当 CI 不可用或需要特殊处理时使用。

## 发布前准备

- [ ] 确认所有要发布的代码已合并到 `main` 分支
- [ ] 确认本地 `main` 分支是最新的：`git pull origin master`

---

## 发布前准备

- [ ] 确认所有要发布的代码已合并到 `main` 分支
- [ ] 确认本地 `main` 分支是最新的：`git pull origin main`
- [ ] 确认 `~/.tauri/updater.key` 密钥存在且有效
- [ ] 确认公钥已填入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`

---

## Step 1 — 更新版本号

**三处同步修改：**

| 文件 | 当前值示例 | 改为 |
|---|---|---|
| `src-tauri/Cargo.toml` | `version = "0.2.0"` | `version = "0.3.0"` |
| `src-tauri/tauri.conf.json` | `"version": "0.2.0"` | `"version": "0.3.0"` |
| `package.json` | `"version": "0.2.0"` | `"version": "0.3.0"` |

- [ ] `Cargo.toml` 版本号已更新
- [ ] `tauri.conf.json` 版本号已更新
- [ ] `package.json` 版本号已更新

---

## Step 2 — 提交版本号变更并打 Tag

```bash
# 提交版本号变更
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json
git commit -m "chore: bump version to v0.3.0"

# 打 tag（格式必须为 v 开头）
git tag v0.3.0

# 推送到 Gitee
git push origin main --tags
```

- [ ] 版本号变更已提交
- [ ] Tag 已推送（`git push origin main --tags`）

---

## Step 3 — 构建各平台安装包

### macOS（Intel + Apple Silicon）

```bash
# x86_64
cargo tauri build --target x86_64-apple-darwin

# arm64
cargo tauri build --target aarch64-apple-darwin
```

产物位置：
- `src-tauri/target/release/bundle/dmg/Files Explorer_x64.dmg`
- `src-tauri/target/release/bundle/dmg/Files Explorer_aarch64.dmg`

签名文件：
- `src-tauri/target/release/Files.Explorer_x64.dmg.sig`
- `src-tauri/target/release/Files.Explorer_aarch64.dmg.sig`

### Windows

```bash
cargo tauri build
```

产物位置：
- `src-tauri/target/release/bundle/msi/Files Explorer_0.3.0_x64_en-US.msi`

签名文件：
- `src-tauri/target/release/Files Explorer_x64-setup.exe.sig`

### Linux

```bash
cargo tauri build
```

产物位置：
- `src-tauri/target/release/bundle/appimage/Files Explorer_0.3.0_amd64.AppImage`

签名文件：
- `src-tauri/target/release/Files.Explorer_amd64.AppImage.sig`

- [ ] macOS x86_64 构建成功
- [ ] macOS arm64 构建成功
- [ ] Windows 构建成功
- [ ] Linux 构建成功
- [ ] 各平台 `.sig` 签名文件已获取

---

## Step 4 — 创建 Gitee Release

1. 打开 https://gitee.com/hhyd/files-explorer/releases/new
2. 选择刚推送的 Tag（如 `v0.3.0`）
3. 标题填写：`v0.3.0`
4. 正文写更新日志（参考模板）
5. **上传安装包**：将 Step 3 构建的安装包拖入附件区
   - `Files Explorer_x64.dmg`（Intel Mac）
   - `Files Explorer_aarch64.dmg`（Apple Silicon Mac）
   - `Files Explorer_0.3.0_x64_en-US.msi`（Windows）
   - `Files Explorer_0.3.0_amd64.AppImage`（Linux）
6. 点击「创建 Release」

- [ ] Gitee Release 已创建
- [ ] 各平台安装包已上传

---

## Step 5 — 更新 updater.json

1. 打开仓库根目录的 `updater.json`
2. 更新以下字段：

| 字段 | 操作 |
|---|---|
| `version` | 改为新版本号（如 `"0.3.0"`） |
| `pub_date` | 改为当前时间（格式 `2026-07-09T18:00:00+08:00`） |
| `notes` | 改为更新日志（支持 Markdown） |
| `platforms.*.signature` | 改为对应 `.sig` 文件的内容（用 `cat` 读出后粘贴） |
| `platforms.*.url` | 确认下载链接格式正确 |

**获取签名值：**

```bash
# macOS x86_64
cat src-tauri/target/release/Files.Explorer_x64.dmg.sig

# macOS arm64
cat src-tauri/target/release/Files.Explorer_aarch64.dmg.sig

# Windows
cat "src-tauri/target/release/Files Explorer_x64-setup.exe.sig"

# Linux
cat src-tauri/target/release/Files.Explorer_amd64.AppImage.sig
```

**下载链接格式：**

```
https://gitee.com/hhyd/files-explorer/releases/download/v0.3.0/文件名
```

- [ ] `version` 已更新
- [ ] `pub_date` 已更新
- [ ] `notes` 已更新
- [ ] 各平台 `signature` 已填入正确值
- [ ] 各平台 `url` 指向正确的 Gitee 下载链接

---

## Step 6 — 提交 updater.json

```bash
git add updater.json
git commit -m "chore: 更新 updater.json 至 v0.3.0"
git push origin main
```

- [ ] `updater.json` 已提交并推送到 `main`

---

## Step 7 — 验证更新流程

- [ ] 打开已安装的旧版本
- [ ] 等待自动检查更新（或重启应用触发检查）
- [ ] 确认弹出更新提示，版本号 = `v0.3.0`
- [ ] 确认更新内容显示正确
- [ ] 点击「立即更新」，确认下载进度正常
- [ ] 下载完成后确认弹出重启提示
- [ ] 重启后确认新版本运行正常

---

## 更新日志模板

```markdown
## v0.3.0 — 2026-07-09

### 🚀 新增
- xxx

### 🐛 修复
- xxx

### 🔧 优化
- xxx

### ⚠️ 注意
- 本次更新需要重新启动应用
```

---

## 密钥管理

- [ ] 确认 `~/.tauri/updater.key` 已备份到密码管理器
- [ ] 确认 `pubkey`（公钥）已填写到 `tauri.conf.json`

> ⚠️ **密钥丢失 = 无法向所有旧版本推送更新。** 每次发版前确认密钥还在。

---

## CI 相关 Secrets 配置

以下 Secrets 需要在 GitHub 仓库设置中配置（Settings → Secrets and variables → Actions）：

| Secret | 说明 | 获取方式 |
|--------|------|---------|
| `TAURI_PRIVATE_KEY` | Tauri 签名私钥内容 | `cat ~/.tauri/updater.key` 复制全文 |
| `TAURI_KEY_PASSWORD` | 私钥密码（如有） | 创建密钥时设置的密码 |
| `GITEE_TOKEN` | Gitee 个人访问令牌 | Gitee → 设置 → 私人令牌 → 创建（勾选 repo） |

> 配置完成后，每次发布只需`git tag v0.x.x && git push origin master --tags`，CI 自动处理后续全部流程。
