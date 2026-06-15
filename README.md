<div align="center">
  <img src="src-tauri/icons/icon.png" alt="Files Explorer" width="80" height="80" />
  <h1>Files Explorer</h1>
  <p>基于 <strong>Tauri 2.0</strong> 和 <strong>Vue 3</strong> 构建的现代化跨平台桌面文件资源管理器</p>
  <p>
    <img src="https://img.shields.io/badge/Tauri-2.0-%23FFC131" alt="Tauri 2.0" />
    <img src="https://img.shields.io/badge/Vue-3.4-%234FC08D" alt="Vue 3.4" />
    <img src="https://img.shields.io/badge/TypeScript-5.3-%233178C6" alt="TypeScript 5.3" />
    <img src="https://img.shields.io/badge/Rust-1.85-%23DEA584" alt="Rust" />
    <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey" alt="Platform" />
  </p>
</div>

---

## 📖 概述

**Files Explorer** 是一款跨平台桌面文件管理器，将 **Rust** 后端的高性能文件操作能力与 **Vue 3** 丰富的交互界面相结合。它提供现代化的用户界面，支持多标签页浏览、分屏面板、完整文件操作、高级搜索以及中英文国际化。

### 🖥️ 平台支持

| 平台 | 状态 | 原生架构 |
|------|------|---------|
| **Windows** | ✅ 完全支持 | x86_64 |
| **macOS** | ✅ 完全支持 | Universal (x86_64 + ARM64) |
| **Linux** | ✅ 完全支持 | x86_64 |

---

## ✨ 功能特性

### 📂 文件浏览与导航
- **四种视图模式** — 详细信息、列表、网格（文件类型颜色编码）、树形（递归展开子目录）
- **流式目录加载** — 100 条/批流式渲染，大目录无卡顿
- **地址栏** — 手动输入路径和面包屑导航
- **历史导航** — 后退 / 前进 / 向上，50 条历史记录
- **面包屑** — 每个面板独立显示路径，点击可跳转
- **刷新** — 重新加载当前目录

### 🖥️ 多面板与标签页
- **多标签页** — 每个面板可打开多个标签页，支持中键关闭
- **分屏面板** — 水平或垂直拆分面板（最多 4 个方向），各自独立浏览
- **标签悬停切换** — 拖拽文件悬停标签 500ms 自动切换
- **独立状态** — 每个标签页记忆自己的路径、文件列表和选中状态

### 📋 文件操作
| 操作 | 说明 | 平台适配 |
|------|------|---------|
| **新建文件夹 / 文件** | 创建空文件或文件夹 | 全平台 |
| **重命名** | 内联重命名，自动选中文件名（不含扩展名），`F2` | 全平台 |
| **复制 / 剪切 / 粘贴** | 内部剪贴板 + 系统剪贴板双向互通 | Win32 CF_HDROP / macOS arboard |
| **删除** | 移入回收站 / 永久删除，带确认对话框 | `trash` crate |
| **打开文件** | 系统默认程序打开 | `opener` crate |
| **文件属性** | 查看系统属性面板 | Win32 SHObjectProperties / macOS AppleScript / Linux gio |
| **在 Finder/资源管理器 中显示** | 定位到文件所在位置 | `explorer /select` / `open -R` / `xdg-open` |
| **在终端中打开** | 在当前目录启动终端 | Windows Terminal / Terminal.app / 自动检测 |
| **拖放** | 内部拖拽移动 + 原生拖出到其他应用 | COM DoDragDrop (Win) / text/uri-list |

### 🔍 高级搜索
- 使用 `walkdir` 递归搜索，独立 OS 线程 + `AtomicBool` 取消
- **三种匹配模式** (支持 `|` 分隔的 OR 组合):
  | 模式 | 示例 | 说明 |
  |------|------|------|
  | 子串匹配 | `readme` | 大小写不敏感 |
  | 通配符 | `*.rs`, `test?.*` | `*` 任意字符, `?` 单个字符 |
  | 大小过滤 | `>10MB`, `<1GB` | 支持 B/KB/MB/GB/TB |
- 分批流式返回结果（500 条/批），上限 2000 条
- 搜索结果独立标签页展示

### ⏪ 撤销系统
- 自动记录操作历史（最多 50 条）
- `Ctrl+Z` 撤销支持：
  | 操作 | 撤销行为 |
  |------|---------|
  | 重命名 | 恢复原文件名 |
  | 新建文件/文件夹 | 删除新建项 |
  | 复制粘贴 | 移除副本 |
  | 剪切粘贴 | 回退文件到原位置 + 移除副本 |
- 删除操作出于安全考虑 **不可撤销**

### ⌨️ 快捷键
| 快捷键 | 功能 |
|--------|------|
| `Enter` | 打开选中文件 / 进入目录 |
| `Backspace` | 返回上级目录 |
| `Ctrl+N` | 新建文件 |
| `Ctrl+Shift+N` | 新建文件夹 |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | 复制 / 剪切 / 粘贴 |
| `Ctrl+A` | 全选 |
| `Delete` / `Shift+Delete` | 回收站 / 永久删除（带确认） |
| `F2` | 重命名 |
| `F5` | 刷新 |
| `Ctrl+W` | 关闭当前标签页 |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | 切换标签页 |
| `Ctrl+Z` | 撤销上次操作 |
| `Esc` | 取消剪切状态 |
| `Ctrl+,` | 打开设置 |

### 🎨 视觉设计
- **现代化风格** — Fluent Design 文件类型图标（Word/Excel/PPT/PDF/代码/图片/视频/音频/压缩包等 50+ 类型）
- **深色 / 浅色主题** — Catppuccin Mocha（深色）和 Catppuccin Latte（浅色）配色方案
- **三种字体大小** — 小 / 中 / 大
- **文件类型颜色编码** — 9 种颜色分类，深色/浅色主题自动适配
- **自定义标题栏** — 自绘窗口控件（最小化/最大化/关闭）

### 🌐 国际化
- **简体中文** 和 **English** 双语支持
- 覆盖所有 UI 文本、右键菜单、对话框、文件类型标签
- 切换即时生效，无需重启

---

## 🏗️ 架构设计

### 整体分层

```
┌────────────────────────────────────────────────────┐
│                 Vue 3 前端层 (src/)                  │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ 14 组件   │  │ 8 Composables │  │  3 Pinia Stores│ │
│  └──────────┘  └──────────────┘  └──────────────┘  │
├────────────────────────────────────────────────────┤
│                Tauri IPC Bridge                     │
│        invoke() / listen() / emit()                 │
├────────────────────────────────────────────────────┤
│                Rust 后端层 (src-tauri/)              │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ 23 个命令 │  │  AppState    │  │ native_drag   │  │
│  │ (lib.rs) │  │  (剪贴板/撤销) │  │ (Win COM)    │  │
│  └──────────┘  └──────────────┘  └──────────────┘  │
└────────────────────────────────────────────────────┘
```

### 三仓库模式（Three-Stores Pattern）

| 仓库 | 职责 | 关键状态 |
|------|------|---------|
| **`fileStore`** | 文件浏览与操作 | `currentPath`、`files`、`drives`、`selectedFiles`、`history`(50条)、`cutFiles`、`viewMode`、`treeExpanded`、`isSearching`、`canUndo` |
| **`tabStore`** | 分屏布局与标签页 | 递归 `LayoutNode` 树（Pane/Split）、标签增删切换、拖拽悬停、拖放状态 |
| **`settingsStore`** | 用户偏好设置 | `theme`、`locale`、`fontSize`，全部持久化到 `localStorage` |

### Composables 事件编排层 (8 个)

| Composable | 职责 |
|-----------|------|
| `useFileActions` | 文件操作调度（open/cut/copy/paste/delete/rename/new/refresh/properties/showInExplorer） |
| `usePanelNavigation` | 面板/标签页导航（切换/新建/关闭/分割/侧栏跳转） |
| `useKeyboardShortcuts` | 全局快捷键注册与分发 |
| `useSearchService` | 搜索生命周期管理（创建搜索标签、监听 search-progress 事件、取消） |
| `useDragDrop` | 内部文件拖放移动 + 标签拖放导入 |
| `useNativeDrop` | OS 原生文件拖入（Tauri onDragDropEvent） |
| `useContextMenu` | 右键菜单动态生成（根据选中状态/路径上下文） |
| `useToast` | 全局 Toast 消息提示 |

### 数据流

```
用户操作（点击、按键、拖拽）
        │
        ▼
  Vue 组件 ──emit──► App.vue（Composables 编排）
        │                        │
        │              ┌─────────┴──────────┐
        │              ▼                    ▼
        │        tabStore              fileStore
        │      (面板/标签树)          (文件/导航/剪贴板)
        │              │                    │
        │              └─────────┬──────────┘
        │                        ▼
        ▼                  Tauri IPC invoke()
  FileList 读取         ───────────────────►
  活动标签页数据             Rust 后端命令
                                    │
                          ┌─────────┴──────────┐
                          ▼                    ▼
                     文件系统操作         AppState 管理
                  (walkdir/trash/fs)   (剪贴板/撤销/搜索取消)
```

### Rust 后端命令（共 23 个）

| 分类 | 命令 | 说明 | 平台 |
|------|------|------|------|
| **目录浏览** | `list_directory` | 列出目录内容并排序（目录优先） | 全平台 |
| | `list_directory_streamed` | 流式目录加载，100条/批发射事件 | 全平台 |
| | `get_drives` | 获取磁盘驱动器信息 | Win32 API / Unix statvfs |
| **导航** | `get_parent_directory` | 获取父目录路径 | 全平台 |
| | `path_exists` | 检查路径是否存在 | 全平台 |
| | `get_special_dirs` | 获取用户特殊目录（桌面/文档/下载等） | 全平台 |
| **文件操作** | `create_directory` | 递归创建目录 + 撤销记录 | 全平台 |
| | `create_file` | 创建空文件 + 撤销记录 | 全平台 |
| | `delete_item` | 移入回收站 / 永久删除 | trash crate |
| | `rename_item` | 重命名 + 撤销记录 | 全平台 |
| | `move_files` | 移动/复制文件（同设备 rename / 跨设备 copy+delete） | 全平台 |
| **剪贴板** | `copy_clipboard` | 复制：内部剪贴板 + 可选的系统剪贴板 | Win32 CF_HDROP / arboard |
| | `cut_clipboard` | 剪切：内部剪贴板 + 可选的系统剪贴板 | 同上 |
| | `paste_clipboard` | 粘贴（优先系统剪贴板 → 回退内部），名冲突自动解决 | 同上 |
| | `get_clipboard_info` | 查询剪贴板状态 | 全平台 |
| **文件打开** | `open_file` | 系统默认程序打开 | opener crate |
| | `show_in_explorer` | 在文件管理器中定位 | `explorer /select` / `open -R` / `xdg-open` |
| | `show_file_properties` | 显示系统属性面板 | SHObjectProperties / AppleScript / gio |
| | `open_in_terminal` | 在当前目录打开终端 | wt/cmd / Terminal.app / 自动检测 |
| **搜索** | `search_files` | 递归搜索（子串/通配符/大小过滤，OR组合） | walkdir |
| | `cancel_search` | 通过 AtomicBool 取消搜索线程 | 全平台 |
| **拖放** | `start_native_drag_cmd` | 原生拖出到其他应用 | Windows COM DoDragDrop |
| **撤销** | `undo_last_action` | 撤销最近操作（重命名/新建/复制/剪切） | 全平台 |
| | `get_undo_info` | 查询撤销栈顶信息 | 全平台 |

---

## 📁 项目结构

```
files-explorer/
├── src/                              # 前端 (Vue 3 + TypeScript)
│   ├── components/                   # Vue 组件 (14个)
│   │   ├── Breadcrumb.vue            # 面包屑导航
│   │   ├── ContextMenu.vue           # 右键上下文菜单（支持子菜单/分隔符/快捷键）
│   │   ├── DetailsListView.vue       # 详细信息视图（列排序/拖拽调整列宽）
│   │   ├── FileItem.vue              # 单个文件/文件夹项（三种视图共用）
│   │   ├── FileList.vue              # 文件列表容器（排序/列宽/拖放目标）
│   │   ├── GridView.vue              # 网格视图（文件类型颜色编码）
│   │   ├── PaneNode.vue              # 递归分屏面板容器
│   │   ├── RibbonToolbar.vue         # Ribbon 风格操作栏
│   │   ├── Sidebar.vue               # 侧栏（驱动器 + 快速访问）
│   │   ├── StatusBar.vue             # 底栏状态栏
│   │   ├── ThisPcView.vue            # "此电脑" 驱动器卡片视图
│   │   ├── TitleBar.vue              # 自定义窗口标题栏
│   │   ├── Toolbar.vue               # 工具栏（导航按钮 + 地址栏 + 搜索框）
│   │   ├── TreeView.vue              # 树形视图（递归展开）
│   │   └── Dialogs/                  # 模态对话框
│   │       ├── DeleteConfirmDialog.vue
│   │       ├── NewItemDialog.vue
│   │       ├── RenameDialog.vue
│   │       └── SettingsDialog.vue
│   ├── composables/                  # 组合式函数 (8个)
│   │   ├── useContextMenu.ts         # 右键菜单逻辑
│   │   ├── useDragDrop.ts            # 内部拖放（文件移动/标签导入）
│   │   ├── useFileActions.ts         # 文件操作调度
│   │   ├── useKeyboardShortcuts.ts   # 全局快捷键
│   │   ├── useNativeDrop.ts          # OS 原生文件拖入
│   │   ├── usePanelNavigation.ts     # 面板/标签导航
│   │   ├── useSearchService.ts       # 搜索服务
│   │   └── useToast.ts              # Toast 提示
│   ├── stores/                       # Pinia 状态仓库 (3个)
│   │   ├── fileStore.ts              # 文件浏览/操作/历史/剪切/搜索/撤销
│   │   ├── settingsStore.ts          # 主题/语言/字体大小
│   │   └── tabStore.ts               # 分屏布局树/标签页管理
│   ├── locales/                      # 国际化
│   │   ├── en.ts                     # English
│   │   └── zh.ts                     # 简体中文
│   ├── types/
│   │   └── index.ts                  # TypeScript 类型定义
│   ├── utils/
│   │   ├── fileIcons.ts              # Fluent 文件图标（50+ 类型映射）
│   │   ├── fileTypes.ts              # 文件类型颜色编码
│   │   ├── logger.ts                 # 前端日志
│   │   └── tauri.ts                  # Tauri IPC 封装（20+ 函数）
│   ├── App.vue                       # 根组件（事件编排中心）
│   ├── i18n.ts                       # vue-i18n 配置
│   ├── main.ts                       # 应用入口
│   └── style.css                     # 全局样式 + Catppuccin 主题变量
│
├── src-tauri/                        # 后端 (Rust)
│   ├── src/
│   │   ├── lib.rs                    # 23 个 Tauri 命令 (~1420 行)
│   │   ├── main.rs                   # 入口 + 日志初始化
│   │   └── native_drag.rs            # Windows COM DoDragDrop (~220 行)
│   ├── capabilities/
│   │   └── default.json              # 权限配置
│   ├── icons/                        # 应用图标
│   ├── tests/                        # Rust 测试
│   ├── Cargo.toml                    # Rust 依赖
│   ├── build.rs                      # Tauri 构建脚本
│   └── tauri.conf.json               # Tauri 配置（无边框窗口 + 拖放启用）
│
├── screenshots/                      # 截图
├── scripts/                          # 构建脚本
├── index.html                        # HTML 入口
├── vite.config.ts                    # Vite 配置（别名 @ → src）
├── tsconfig.json                     # TypeScript 配置
└── package.json                      # Node.js 依赖
```

---

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org) ≥ 18
- [Rust](https://rustup.rs) ≥ 1.77
- [Tauri 2.0 系统依赖](https://v2.tauri.app/start/prerequisites/)

**macOS 额外依赖：**
```bash
xcode-select --install  # Xcode Command Line Tools
```

**Linux 额外依赖：**
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev  # Debian/Ubuntu
```

### 开发模式

```bash
# 安装前端依赖
npm install

# 启动开发模式（热重载）
npm run tauri dev
```

### 构建

```bash
# 标准构建（当前平台）
npm run tauri build

# macOS 通用二进制（Intel + Apple Silicon）
npm run tauri build -- --target universal-apple-darwin

# 仅构建 .app（跳过 DMG）
npm run tauri build -- --bundles app

# 构建 DMG 安装镜像
npm run tauri build -- --bundles dmg

# 未签名构建（跳过代码签名）
npm run tauri build -- --no-sign
```

### 构建产物位置

```
# 标准构建
src-tauri/target/release/bundle/macos/Files Explorer.app    # macOS
src-tauri/target/release/files-explorer.exe                 # Windows

# Universal 构建 (macOS)
src-tauri/target/universal-apple-darwin/release/bundle/
├── macos/Files Explorer.app                               # Universal .app
└── dmg/Files Explorer_0.1.0_universal.dmg                 # DMG 安装镜像
```

### 首次运行（macOS 未签名）

```bash
# 绕过 Gatekeeper
xattr -cr "Files Explorer.app"
open "Files Explorer.app"
```

---

## 🧩 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **桌面框架** | [Tauri 2.0](https://v2.tauri.app) | Rust 后端 + WebView 前端 |
| **前端框架** | [Vue 3.4](https://vuejs.org) | Composition API + `<script setup>` |
| **状态管理** | [Pinia 2](https://pinia.vuejs.org) | 三仓库模式 |
| **国际化** | [vue-i18n 9](https://vue-i18n.intlify.dev) | 中英双语 |
| **构建工具** | [Vite 5](https://vitejs.dev) | 前端构建 + HMR |
| **后端语言** | [Rust](https://www.rust-lang.org) | 内存安全 + 零成本抽象 |

### 关键 Rust 依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| `tauri` | 2.x | 桌面框架核心 |
| `walkdir` | 2 | 递归目录遍历（搜索） |
| `serde` / `serde_json` | 1 | JSON 序列化 |
| `chrono` | 0.4 | 时间处理 |
| `trash` | 3 | 跨平台回收站操作 |
| `opener` | 0.7 | 跨平台文件打开 |
| `arboard` | 3 | macOS/Linux 系统剪贴板 |
| `log` / `simplelog` | 0.4 / 0.12 | 应用日志 |

---

## 🖼️ 截图

| 深色主题 | 浅色主题 |
|----------|----------|
| ![深色主题](screenshots/dark.png) | ![浅色主题](screenshots/light.png) |

---

## 📄 许可证

本项目基于 MIT 许可证开源。详见 [LICENSE](LICENSE) 文件。

---

<div align="center">
  使用 ❤️ 和 Tauri + Vue + Rust 构建
</div>
