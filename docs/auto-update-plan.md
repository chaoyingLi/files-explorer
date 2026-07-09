# 自动更新方案 — 实现步骤

> 基于 `tauri-plugin-updater` v2，更新源使用 Gitee Releases。
> 发布策略：**GitHub Actions CI 自动构建 + 上传到 Gitee + GitHub**。
> 也支持纯手动发布作为备用方案（参见 [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)）。

---

## 实现步骤总览

| 步骤 | 涉及文件 | 改动类型 |
|---|---|---|
| 1. 生成签名密钥 | 本地 `~/.tauri/updater.key` | 一次性操作 |
| 2. 后端添加依赖 | `src-tauri/Cargo.toml` | 新增 |
| 3. 注册插件 | `src-tauri/src/tauri_setup.rs` | 修改 |
| 4. 修改 tauri.conf.json | `src-tauri/tauri.conf.json` | 修改 |
| 5. 添加 capabilities 权限 | `src-tauri/capabilities/default.json` | 修改 |
| 6. 前端 updater 服务 | `src/utils/updater.ts` | 新增 |
| 7. 前端 UpdaterChecker 组件 | `src/components/UpdaterChecker.vue` | 新增 |
| 8. 集成到 App | `src/App.vue` | 修改 |
| 9. 添加 i18n 文案 | `src/locales/zh-CN.json` 等 | 修改 |
| 10. 添加 updater.json 元数据 | 仓库根目录 `updater.json` | 新增 |

---

## Step 1 — 生成签名密钥

```bash
# 生成密钥对（只需执行一次）
cargo tauri signer generate -w ~/.tauri/updater.key

# 控制台会输出公钥（pubkey），保存它，后续填入 tauri.conf.json
```

> ⚠️ **`~/.tauri/updater.key` 务必备份到密码管理器**，丢失后旧版本永远无法升级。

---

## Step 2 — 后端添加依赖

### `src-tauri/Cargo.toml`

在 `[dependencies]` 区域新增两行：

```toml
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

---

## Step 3 — 注册插件

### `src-tauri/src/tauri_setup.rs`

在插件注册链中添加：

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
.plugin(tauri_plugin_process::init())
```

---

## Step 4 — 修改 tauri.conf.json

### `src-tauri/tauri.conf.json`

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    // ... 已有配置不变 ...
  },
  "plugins": {
    // ... 已有 shell 等配置 ...
    "updater": {
      "endpoints": [
        "https://gitee.com/hhyd/files-explorer/raw/main/updater.json"
      ],
      "pubkey": "（这里填入 Step 1 生成的公钥）"
    }
  }
}
```

---

## Step 5 — 添加 capabilities 权限

### `src-tauri/capabilities/default.json`

新增 updater 和 process 权限：

```json
{
  "identifier": "default",
  "description": "默认权限集",
  "windows": ["main", "splashscreen"],
  "permissions": [
    // ... 已有权限 ...
    "updater:default",
    "process:default",
    "process:allow-restart",
    "process:allow-exit"
  ]
}
```

---

## Step 6 — 前端 updater 服务

### `src/utils/updater.ts`

新增服务文件，包含：

- **类型定义**：`UpdateState`、`UpdateTaskState`、`AvailableUpdateRef` 等
- **`checkForUpdates()`** — 调用 `@tauri-apps/plugin-updater` 的 `check()`
- **`startBackgroundInstall()`** — 后台下载并安装
- **`relaunchAfterUpdate()`** — 重启应用
- **状态管理**：`subscribeUpdateTask()` 监听更新进度
- **Dev Mock**：`enableMock()` / `disableMock()` 方便开发测试

核心 API：

```typescript
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
```

---

## Step 7 — 前端 UpdaterChecker 组件

### `src/components/UpdaterChecker.vue`

一个 Vue 3 组件，自动挂载在 App 中：

- **启动时检查更新**（可通过 store 配置开关）
- **有更新时**：弹出对话框显示版本号 + 更新内容，提供「立即更新」和「稍后提醒」
- **正在下载时**：显示进度状态
- **安装完成**：弹出重启确认对话框
- **dev 模式**：在 `window.__updaterTest` 挂载测试辅助方法

对话框使用已有 UI 组件或原生 `confirm` 弹窗（为保持极简）。

---

## Step 8 — 集成到 App

### `src/App.vue`

```vue
<script setup lang="ts">
import UpdaterChecker from './components/UpdaterChecker.vue';
</script>

<template>
  <!-- 已有 layout -->
  <UpdaterChecker />
</template>
```

---

## Step 9 — 添加 i18n 文案

### `src/locales/zh-CN.json`

```json
{
  "updater": {
    "checking": "正在检查更新...",
    "available": "发现新版本 {version}",
    "updateNow": "立即更新",
    "updateLater": "稍后提醒",
    "downloading": "正在下载更新...",
    "installing": "正在安装更新...",
    "restartNow": "立即重启",
    "restartLater": "稍后重启",
    "restartPrompt": "更新已安装，重启应用以生效",
    "noUpdate": "当前已是最新版本",
    "updateFailed": "更新失败",
    "updateBody": "更新内容"
  }
}
```

其他语言文件同步添加。

---

## Step 10 — updater.json 元数据

### 仓库根目录 `updater.json`

```json
{
  "version": "0.2.0",
  "notes": "## 更新内容\n\n请访问 Gitee Releases 查看详情。",
  "pub_date": "2026-07-09T18:00:00+08:00",
  "platforms": {
    "darwin-x86_64": {
      "signature": "",
      "url": "https://gitee.com/hhyd/files-explorer/releases/download/v0.2.0/Files.Explorer_x64.dmg"
    },
    "darwin-aarch64": {
      "signature": "",
      "url": "https://gitee.com/hhyd/files-explorer/releases/download/v0.2.0/Files.Explorer_aarch64.dmg"
    },
    "windows-x86_64": {
      "signature": "",
      "url": "https://gitee.com/hhyd/files-explorer/releases/download/v0.2.0/Files.Explorer_x64-setup.exe"
    },
    "linux-x86_64": {
      "signature": "",
      "url": "https://gitee.com/hhyd/files-explorer/releases/download/v0.2.0/Files.Explorer_amd64.AppImage"
    }
  }
}
```

> 发布新版本时同步更新 `version`、`signature`、`url`、`pub_date`、`notes`。

---

## 验证清单

| # | 验证项 | 预期结果 |
|---|---|---|
| 1 | `cargo tauri build` 编译通过 | ✅ 无错误 |
| 2 | 启动应用，检查终端日志 | ✅ updater 插件初始化成功 |
| 3 | Dev 模式 Mock：模拟有更新 | ✅ 弹出更新对话框 |
| 4 | Dev 模式 Mock：模拟无更新 | ✅ 静默跳过 |
| 5 | Dev 模式 Mock：模拟下载失败 | ✅ 弹出错误提示 |
| 6 | 发布到 Gitee Release，正式检查更新 | ✅ 能检测到新版本 |
| 7 | 点击「立即更新」下载安装 | ✅ 下载后提示重启 |
| 8 | 点击「立即重启」 | ✅ 应用关闭并重启为新版本 |
