<div align="center">
  <img src="src-tauri/icons/icon.png" alt="Files Explorer" width="80" height="80" />
  <h1>Files Explorer</h1>
  <p>基于 <strong>Tauri 2.0</strong> 和 <strong>Vue 3</strong> 构建的现代化桌面文件资源管理器</p>
  <p>
    <img src="https://img.shields.io/badge/Tauri-2.0-%23FFC131" alt="Tauri 2.0" />
    <img src="https://img.shields.io/badge/Vue-3.4-%234FC08D" alt="Vue 3.4" />
    <img src="https://img.shields.io/badge/TypeScript-5.3-%233178C6" alt="TypeScript 5.3" />
    <img src="https://img.shields.io/badge/Rust-1.85-%23DEA584" alt="Rust" />
    <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
  </p>
</div>

---

## 📖 概述

**Files Explorer** 是一款跨平台桌面文件管理器，将 **Rust** 后端的高性能文件操作能力与 **Vue 3** 丰富的交互界面相结合。它提供熟悉的 Windows 11 风格界面，支持多标签页浏览、分屏面板、文件操作、搜索以及完整的中英文国际化。

## ✨ 功能特性

### 📂 文件浏览与导航
- **目录列表** — 支持按名称、修改日期、类型、大小排序
- **三种视图模式** — 详细信息、列表、网格（文件类型按颜色编码）
- **地址栏** — 手动输入路径和面包屑导航
- **历史导航** — 后退 / 前进 / 向上，50 条历史记录
- **面包屑** — 每个面板独立显示路径，点击可跳转
- **刷新** — 重新加载当前目录

### 🖥️ 多面板与标签页
- **多标签页** — 每个面板可打开多个标签页，支持中键关闭
- **分屏面板** — 水平或垂直拆分面板，各自独立浏览
- **标签悬停切换** — 拖拽标签悬停 500ms 自动切换
- **独立状态** — 每个标签页记住自己的路径、文件和选中状态

### 📋 文件操作
| 操作 | 说明 |
|---|---|
| **新建文件夹 / 文件** | 创建空文件或文件夹 |
| **重命名** | 内联重命名，自动选中文件名（不含扩展名），快捷键 `F2` |
| **复制 / 剪切 / 粘贴** | 内部剪贴板，剪切状态视觉反馈（✂ 图标） |
| **删除** | 移入回收站（`Delete`）或永久删除（`Shift+Delete`），带确认对话框 |
| **打开** | 使用系统默认程序打开文件 |
| **在终端中打开** | 在当前目录打开终端（Windows Terminal / cmd、macOS Terminal、Linux 自动检测） |
| **拖放** | 拖拽文件到其他目录实现移动 |

### 🔍 搜索
- 使用 `walkdir` 递归搜索文件名，300ms 防抖
- 搜索结果与文件列表独立显示
- 点击清除按钮取消搜索

### 🎨 视觉设计
- **Windows 11 风格** — Fluent Design 图标，基于类型分色（代码、图片、音频、视频、压缩包、PDF、应用、网页）
- **深色 / 浅色主题** — Catppuccin Mocha（深色）和 Catppuccin Latte（浅色）配色方案
- **三种字体大小** — 小 / 中 / 大
- **文件类型颜色编码** — 9 种颜色分类，深色/浅色主题各自适配

### 🌐 国际化
- **简体中文** 和 **English** 双语 — 覆盖所有 UI 文本、右键菜单、对话框、文件类型标签
- 切换即时生效，无需重启

### ⌨️ 快捷键
| 快捷键 | 功能 |
|---|---|
| `Enter` | 打开选中文件 / 进入目录 |
| `Backspace` | 返回上级目录 |
| `Ctrl+N` | 新建文件 |
| `Ctrl+Shift+N` | 新建文件夹 |
| `Ctrl+C` / `Ctrl+X` | 复制 / 剪切 |
| `Ctrl+V` | 粘贴 |
| `Ctrl+A` | 全选 |
| `Delete` | 移入回收站（带确认） |
| `Shift+Delete` | 永久删除（带确认） |
| `F2` | 重命名 |
| `F5` | 刷新 |
| `Ctrl+W` | 关闭当前标签页 |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | 切换标签页 |
| `Ctrl+Z` | 撤销上次操作（重命名、新建、复制） |
| `Esc` | 取消剪切状态 |
| `Ctrl+,` | 打开设置 |

### ⏪ 撤销历史
- 自动记录文件操作（新建、重命名、复制）
- `Ctrl+Z` 撤销：重命名 → 恢复原名，新建 → 删除新项目，复制 → 移除副本
- 删除操作出于安全考虑不可撤销

---

## 🏗️ 技术栈

| 层级 | 技术 |
|---|---|
| **桌面框架** | [Tauri 2.0](https://v2.tauri.app) |
| **前端框架** | [Vue 3](https://vuejs.org)（Composition API + `<script setup>`） |
| **状态管理** | [Pinia](https://pinia.vuejs.org) |
| **国际化** | [vue-i18n](https://vue-i18n.intlify.dev) |
| **构建工具** | [Vite 5](https://vitejs.dev) |
| **后端语言** | [Rust](https://www.rust-lang.org) |
| **关键 Rust 库** | `walkdir`、`chrono`、`trash`、`opener`、`serde` |

## 📁 项目结构

```
files-explorer/
├── src/                          # 前端 (Vue 3)
│   ├── components/               # Vue 组件
│   │   ├── Breadcrumb.vue        # 面包屑导航
│   │   ├── ContextMenu.vue       # 右键上下文菜单
│   │   ├── FileItem.vue          # 单个文件/文件夹行
│   │   ├── FileList.vue          # 文件列表（详情/列表/网格）
│   │   ├── PaneNode.vue          # 递归分屏面板容器
│   │   ├── RibbonToolbar.vue     # Ribbon 风格操作栏
│   │   ├── Sidebar.vue           # 侧栏（驱动器 + 快速访问）
│   │   ├── StatusBar.vue         # 状态栏
│   │   ├── Toolbar.vue           # 工具栏（导航按钮 + 地址栏）
│   │   └── Dialogs/              # 模态对话框
│   │       ├── DeleteConfirmDialog.vue  # 删除确认
│   │       ├── NewItemDialog.vue        # 新建文件/文件夹
│   │       ├── RenameDialog.vue         # 重命名
│   │       └── SettingsDialog.vue       # 设置
│   ├── stores/                   # Pinia 状态仓库
│   │   ├── fileStore.ts          # 文件浏览与操作状态
│   │   ├── settingsStore.ts      # 主题、语言、字体大小
│   │   └── tabStore.ts           # 标签页与分屏布局状态
│   ├── locales/                  # 国际化翻译
│   │   ├── en.ts                 # English
│   │   └── zh.ts                 # 简体中文
│   ├── types/                    # TypeScript 类型定义
│   │   └── index.ts
│   ├── utils/                    # Tauri IPC 工具函数
│   │   └── tauri.ts
│   ├── App.vue                   # 根组件
│   ├── i18n.ts                   # 国际化配置
│   ├── main.ts                   # 应用入口
│   └── style.css                 # 全局样式与主题变量
│
├── src-tauri/                    # 后端 (Rust)
│   ├── src/
│   │   ├── lib.rs                # Tauri 命令与业务逻辑
│   │   └── main.rs               # 入口文件
│   ├── capabilities/             # 权限配置
│   │   └── default.json
│   ├── icons/                    # 应用图标
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
│
├── index.html                    # HTML 入口
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
├── package.json                  # Node.js 依赖
└── README.md                     # 本文件
```

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org) ≥ 18
- [Rust](https://rustup.rs) ≥ 1.77
- [Tauri 2.0 系统依赖](https://v2.tauri.app/start/prerequisites/)（各平台不同）

### 开发模式

```bash
# 安装前端依赖
npm install

# 启动开发模式（热重载）
npm run tauri dev
```

### 构建

```bash
# 构建生产版本可执行文件
npm run tauri build
```

构建产物位置：

```
src-tauri/target/release/files-explorer.exe   # Windows
src-tauri/target/release/files-explorer       # macOS / Linux
```

---

## 🧩 架构说明

### 数据流

```
用户操作（点击、按键、拖拽）
        │
        ▼
  Vue 组件 ──emit──► App.vue（事件处理器）
        │                        │
        │                   ┌─────┴──────┐
        │                   ▼            ▼
        │            tabStore        fileStore
        │          (面板/标签)     (文件/历史)
        │                   │            │
        │                   └─────┬──────┘
        │                         ▼
        │                    通过 watch 同步
        │                    (currentPath)
        │                         │
        ▼                         ▼
  FileList 读取             Tauri IPC invoke()
  活动标签页数据            ────────────────►
  (tabStore 数据)               Rust 命令
                                      │
                              ┌───────┴───────┐
                              ▼               ▼
                         文件系统操作      剪贴板管理
```

### 三仓库模式（Three-Stores Pattern）

| 仓库 | 职责 |
|---|---|
| **`fileStore`** | 当前路径、文件列表、选中状态、搜索结果、导航历史、剪切状态、删除确认、撤销状态 |
| **`tabStore`** | 面板布局树（递归分片节点）、标签页管理（增删切换拖拽）、按标签页持久化文件状态 |
| **`settingsStore`** | 主题（深色/浅色）、语言、字体大小，持久化到 `localStorage` |

### Rust 后端命令（共 19 个）

| 命令 | 说明 |
|---|---|
| `list_directory` | 列出目录内容并排序 |
| `get_drives` | 获取磁盘驱动器（Windows Win32 API / Unix `statvfs`） |
| `get_parent_directory` | 获取父目录路径 |
| `create_directory` | 递归创建目录（记录撤销历史） |
| `create_file` | 创建空文件（记录撤销历史） |
| `delete_item` | 移入回收站或永久删除 |
| `rename_item` | 重命名文件/目录（记录撤销历史） |
| `copy_clipboard` | 复制到内部剪贴板 |
| `cut_clipboard` | 剪切到内部剪贴板 |
| `paste_clipboard` | 从剪贴板粘贴（复制或移动） |
| `get_file_info` | 获取文件元数据 |
| `open_file` | 使用系统默认程序打开 |
| `open_in_terminal` | 在当前目录打开终端 |
| `search_files` | 递归搜索文件名 |
| `path_exists` | 检查路径是否存在 |
| `get_special_dirs` | 获取用户特殊目录（桌面、文档等） |
| `get_clipboard_info` | 查询剪贴板状态 |
| `undo_last_action` | 撤销上一次文件操作 |
| `get_undo_info` | 查询撤销栈顶信息 |

---

## 🖼️ 截图

<!-- 请在此处添加截图 -->

| 深色主题 | 浅色主题 |
|---|---|
| ![深色主题](screenshots/dark.png) | ![浅色主题](screenshots/light.png) |

---

## 📄 许可证

本项目基于 MIT 许可证开源。详见 [LICENSE](LICENSE) 文件。

---

<div align="center">
  使用 ❤️ 和 Tauri + Vue 构建
</div>
