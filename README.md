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

**Files Explorer** 是一款跨平台桌面文件管理器，将 **Rust** 后端的高性能文件操作能力与 **Vue 3** 丰富的交互界面相结合。采用 **模块化架构**（前端 6 Pinia Stores + 后端 12 Rust 模块），支持多标签页、分屏面板、完整文件操作、高级搜索、智能文件预览、归档浏览以及中英文国际化。

### 🖥️ 平台支持

| 平台 | 状态 | 原生架构 |
|------|------|---------|
| **Windows** | ✅ 完全支持 | x86_64 |
| **macOS** | ✅ 完全支持 | Universal (x86_64 + ARM64) |
| **Linux** | ✅ 完全支持 | x86_64 |

---

## ✨ 功能特性

### 📂 文件浏览与导航
- **五种视图模式** — 详细信息、列表、网格、树形（递归展开）、分栏（Miller Columns）
- **虚拟滚动** — 大目录（10,000+ 文件）仅渲染可见行，保持 60fps
- **流式目录加载** — 100 条/批流式渲染，大目录无卡顿
- **地址栏自动补全** — 输入路径时实时查询子目录并下拉匹配
- **面包屑导航** — 每个面板独立显示，点击路径段跳转
- **历史导航** — 后退 / 前进 / 向上，50 条历史记录
- **预览面板** — 右侧栏：智能内容预览 + 文件属性（`Ctrl+P`）

### 🖥️ 多面板与标签页
- **多标签页** — 每个面板可打开多个标签页，`Ctrl+W` 关闭
- **分屏面板** — 水平或垂直拆分面板（最多 4 个方向），各自独立浏览，分隔线可拖拽调整
- **标签悬停切换** — 拖拽文件悬停标签 500ms 自动切换
- **独立状态** — 每个标签页记忆自己的路径、文件列表、树展开和选中状态

### 🔍 智能文件预览

#### 文本 / 代码预览（47 种语言）
- **VS Code 质量语法高亮** — 基于 Shiki（VS Code 同款引擎），支持 47 种编程语言
- **行号栏** — 独立 gutter 列，右键对齐，背景区分
- **行悬停高亮** — hover 当前行浅色背景
- **智能文本检测** — 内容级识别（null 字节率 + UTF-8 有效性），不依赖扩展名
- **支持 90+ 扩展名** — 覆盖 JS/TS/Python/Rust/Go/Java/C/C++/Swift/Kotlin/Dart/Ruby/PHP/Lua/R/Perl/C#/F#/VB/PowerShell/Haskell/Erlang/Elixir/Clojure/Zig/Nim/Verilog/VHDL/Solidity/OCaml 及所有 Web/Shell/Config 格式
- **Markdown 预览** — DOMPurify 消毒 + marked 实时渲染

#### Office 文档预览
| 格式 | 实现 | 渲染质量 |
|------|------|---------|
| `.docx` | `@vue-office/docx` | 高保真：表格/图片/样式/页眉页脚 |
| `.xlsx` | `@vue-office/excel` | 高保真：合并单元格/公式/样式 |
| `.pdf` | `@vue-office/pdf` | Canvas 渲染 |
| `.pptx` | `pptx-preview` | 逐页预览，内置翻页控制 |

#### 图片预览
- **原生协议加载** — 通过 Tauri `asset://` 协议直接加载，零 IPC 传输，无大小限制
- **尺寸提取** — 自动显示分辨率

#### 归档文件浏览
| 格式 | 支持 | 实现 |
|------|------|------|
| `.zip` | ✅ 列表 + 提取预览 | `zip` crate |
| `.tar` | ✅ | `tar` crate |
| `.tar.gz` `.tgz` | ✅ | `tar` + `flate2` |
| `.tar.bz2` `.tbz2` | ✅ | `tar` + `bzip2` |
| `.tar.xz` `.txz` | ✅ | `tar` + `xz2-lzma` |
| `.7z` | ✅ | `sevenz-rust`（纯 Rust） |
| `.rar` | ✅ | 系统 `unrar` / `7z` |

- **树形列表** — 分层展示归档内容，文件大小右对齐
- **内文件双击预览** — 自动解压到临时目录 → 复用全部预览组件
- **返回机制** — 顶部返回栏 + 右上角浮动图标，可返回归档列表浏览其他文件

#### 独立预览窗口
- ↗ 按钮弹出原生窗口，可拖到副屏 / 最大化
- 左侧 **VS Code 风格文件树** — 懒加载展开、文件夹 chevron、高度递归、颜色分类图标
- 右键菜单 — 打开 / Finder 定位 / 终端打开 / 复制路径
- 树与预览区之间 **可拖拽调整宽度**
- 同文件不重复创建窗口，自动聚焦已有

### 📋 文件操作
| 操作 | 说明 | 平台适配 |
|------|------|---------|
| **新建文件夹 / 文件** | 创建空文件或文件夹 | 全平台 |
| **重命名** | 内联重命名，`F2` | 全平台 |
| **复制 / 剪切 / 粘贴** | 内部剪贴板 + 系统剪贴板双向互通 | Win32 CF_HDROP / macOS NSPasteboard |
| **删除** | 移入回收站 / 永久删除，带确认对话框 | `trash` crate |
| **打开文件** | 系统默认程序打开 | `opener` crate |
| **压缩 / 解压** | 选中文件 → 右键「压缩到...」；右键归档文件「解压到...」 | `zip` + `tar` + `flate2` |
| **文件属性** | 内联属性面板 + 系统属性对话框 | Win32 SHObjectProperties / macOS Finder |
| **在 Finder/资源管理器 中显示** | 定位到文件所在位置 | `explorer /select` / `open -R` / `xdg-open` |
| **在终端中打开** | 在当前目录启动终端 | Windows Terminal / Terminal.app / 自动检测 |
| **拖放** | 内部拖拽移动 + 原生拖出到其他应用 | COM DoDragDrop (Win) / text/uri-list |

### 🔍 高级搜索
- 使用 `walkdir` 递归搜索，独立 OS 线程 + `AtomicBool` 取消
- 自动跳过无关目录（`node_modules`、`.git`、`target`、`__pycache__` 等 17 个）
- **文件名搜索** — 三种匹配模式（支持 `|` 分隔的 OR 组合）：
  | 模式 | 示例 | 说明 |
  |------|------|------|
  | 子串匹配 | `readme` | 大小写不敏感 |
  | 通配符 | `*.rs`, `test?.*` | `*` 任意字符, `?` 单个字符 |
  | 大小过滤 | `>10MB`, `<1GB` | 支持 B/KB/MB/GB/TB |
- **内容搜索** — 点击 🔍 切换按钮，直接在文件内容中搜索文本
- 分批流式返回结果（500 条/批），上限 2000 条
- 搜索结果独立标签页展示

### ⭐ 收藏夹
- 右键文件夹 →「添加到收藏夹」
- 侧边栏 ⭐ Favorites 区域展示所有收藏
- 右键书签 → 确认移除
- 持久化到 `localStorage`

### ⏪ 撤销系统
- 自动记录操作历史（最多 50 条）
- `Ctrl+Z` 撤销支持：重命名、新建文件/文件夹、复制粘贴、剪切粘贴
- 删除操作出于安全考虑 **不可撤销**

### 💾 会话持久化
- 窗口尺寸 / 位置 / 视图模式 / 面板布局自动保存
- 重启后恢复标签页路径和分屏结构
- 侧边栏和预览面板宽度持久化

### ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Enter` | 打开选中文件 / 进入目录 |
| `Backspace` | 返回上级目录 |
| `Space` | 预览 / 打开选中文件 |
| `Ctrl+N` | 新建文件 |
| `Ctrl+Shift+N` | 新建文件夹 |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | 复制 / 剪切 / 粘贴 |
| `Ctrl+A` | 全选 |
| `Delete` / `Shift+Delete` | 回收站 / 永久删除（带确认） |
| `F2` | 重命名 |
| `F5` | 刷新 |
| `Ctrl+P` | 切换预览面板 |
| `Ctrl+W` | 关闭当前标签页 |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | 切换标签页 |
| `Ctrl+Z` | 撤销上次操作 |
| `Ctrl+[` / `Ctrl+]` | 后退 / 前进 |
| `Cmd+↑` / `Cmd+↓` | 上级目录 / 打开选中项 |
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
│  │ 22 组件   │  │ 7 Composables │  │ 6 Pinia Stores│ │
│  └──────────┘  └──────────────┘  └──────────────┘  │
├────────────────────────────────────────────────────┤
│                Tauri IPC Bridge                     │
│        invoke() / listen() / emit()                 │
├────────────────────────────────────────────────────┤
│                Rust 后端层 (src-tauri/)              │
│  ┌──────────────────────────────────────────────┐  │
│  │  12 模块 / 31 个命令                            │  │
│  │  types / files / drives / operations          │  │
│  │  clipboard / search / compress / system / undo │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

### 六仓库模式（Six-Stores Pattern）

| 仓库 | 职责 | 关键状态 |
|------|------|---------|
| **`fileStore`** | 文件浏览核心 | `currentPath`、`files`、`drives`、`loading`、`error`、`isSearching` |
| **`tabStore`** | 分屏布局与标签页 | 递归 `LayoutNode` 树（Pane/Split）、标签增删切换、拖放 |
| **`selectionStore`** | 文件选择与剪贴板 | `selectedFiles`、`cutFiles`、`isCutPending`、copy/cut/paste |
| **`viewStore`** | 视图模式与树形 | `viewMode`、`treeExpanded`、`treeChildrenCache`、分栏操作 |
| **`navigationStore`** | 导航历史 | `history`(50条)、`historyIndex`、back/forward/up/home |
| **`settingsStore`** | 用户偏好与书签 | `theme`、`locale`、`fontSize`、`bookmarks` |

### Composables 事件编排层 (7 个)

| Composable | 职责 |
|-----------|------|
| `useFileActions` | 文件操作调度（open/cut/copy/paste/delete/rename/new/refresh/compress/extract/bookmark） |
| `usePanelNavigation` | 面板/标签页导航（切换/新建/关闭/分割/侧栏跳转） |
| `useKeyboardShortcuts` | 全局快捷键注册与分发（20+ 快捷键） |
| `useSearchService` | 搜索生命周期管理（创建搜索标签、监听 progress 事件、内容搜索） |
| `useDragDrop` | 内部文件拖放移动 + 标签拖放导入 |
| `useContextMenu` | 右键菜单动态生成（根据选中状态/文件类型/路径上下文） |
| `useToast` | 全局 Toast 消息队列（多消息堆叠） |

### Rust 后端命令（共 31 个）

| 分类 | 命令 | 说明 |
|------|------|------|
| **目录浏览** | `list_directory` | 列出目录内容并排序（目录优先） |
| | `list_directory_streamed` | 流式目录加载，100条/批发射事件 |
| | `get_drives` | 获取磁盘驱动器信息 |
| **导航** | `get_parent_directory` | 获取父目录路径 |
| | `path_exists` | 检查路径是否存在 |
| | `get_special_dirs` | 获取用户特殊目录 |
| **文件操作** | `create_directory` / `create_file` | 递归创建 + 撤销记录 |
| | `delete_item` | 移入回收站 / 永久删除 |
| | `rename_item` | 重命名 + 撤销记录 |
| | `move_files` | 移动/复制文件 |
| | `compress_files` | 压缩为 zip（流式进度） |
| | `extract_archive_cmd` | 解压 zip/tar/tar.gz |
| **剪贴板** | `copy_clipboard` / `cut_clipboard` | 内部 + 系统剪贴板双向 |
| | `paste_clipboard` | 粘贴（优先系统 → 回退内部） |
| | `get_clipboard_info` | 查询剪贴板状态 |
| **文件打开** | `open_file` | 系统默认程序打开 |
| | `show_in_explorer` | 在文件管理器中定位 |
| | `show_file_properties` | 显示系统属性面板 |
| | `open_in_terminal` | 在当前目录打开终端 |
| **预览** | `get_file_preview` | 智能文本检测 + 内容预览 |
| | `get_file_icon` | Windows 原生文件图标 |
| | `read_file_bytes` | 读取原始字节（Office/PDF 预览） |
| **归档** | `list_archive_contents` | 列出 zip/tar/7z/rar 内容 |
| | `extract_archive_entry` | 提取归档内单个文件 |
| **搜索** | `search_files` | 递归搜索（子串/通配符/大小过滤/内容搜索） |
| | `cancel_search` | AtomicBool 取消搜索线程 |
| **拖放** | `start_native_drag_cmd` | Windows COM 原生拖拽 |
| **撤销** | `undo_last_action` | 撤销最近操作 |
| | `get_undo_info` | 查询撤销栈顶信息 |

### Rust 模块拆分

```
src-tauri/src/
├── main.rs          # 入口 + 日志轮转
├── lib.rs           # 模块声明 + #[command] 包装 + run()
├── types.rs         # 数据结构（FileEntry, DiskInfo, ArchiveEntry, FsError...）
├── state.rs         # AppState（剪贴板、撤销历史、搜索取消）
├── error.rs         # 结构化错误类型
├── files.rs         # 目录列表 / 文件信息
├── drives.rs        # 磁盘枚举 / 特殊目录
├── operations.rs    # 文件增删改移 + 冲突解决
├── clipboard.rs     # 剪贴板（macOS NSPasteboard / Windows CF_HDROP）
├── search.rs        # 搜索（通配符/过滤下/内容匹配）
├── compress.rs      # 压缩/解压（流式进度事件）
├── system.rs        # 系统命令 + 文件预览 + 归档操作
├── undo.rs          # 撤销系统
└── native_drag.rs   # Windows COM 原生拖拽
```

---

## 📁 项目结构

```
files-explorer/
├── src/                              # 前端 (Vue 3 + TypeScript)
│   ├── components/                   # Vue 组件 (22个)
│   │   ├── Breadcrumb.vue            # 面包屑导航
│   │   ├── CodePreview.vue           # VS Code 风格代码预览（Shiki + 行号栏）
│   │   ├── ColumnContainer.vue       # 分栏视图容器
│   │   ├── ColumnPane.vue            # 分栏视图单列
│   │   ├── ContextMenu.vue           # 右键上下文菜单
│   │   ├── DetailsListView.vue       # 详细信息视图（虚拟滚动）
│   │   ├── FileItem.vue              # 文件/文件夹项
│   │   ├── FileList.vue              # 文件列表容器
│   │   ├── GridView.vue              # 网格视图
│   │   ├── PaneNode.vue              # 递归分屏面板容器
│   │   ├── PptxPreview.vue           # PPTX 预览组件
│   │   ├── PreviewWindow.vue         # 独立预览窗口（文件树 + 树脂）
│   │   ├── PropertiesPanel.vue       # 预览面板（预览置顶 + 属性置底）
│   │   ├── RibbonToolbar.vue         # Ribbon 风格操作栏
│   │   ├── Sidebar.vue               # 侧栏（驱动器/收藏夹/快速访问）
│   │   ├── StatusBar.vue             # 底栏状态栏
│   │   ├── ThisPcView.vue            # "此电脑" 驱动器卡片视图
│   │   ├── TitleBar.vue              # 自定义窗口标题栏
│   │   ├── Toolbar.vue               # 工具栏（导航/地址栏/搜索）
│   │   ├── TreeView.vue              # 树形视图（递归展开）
│   │   └── Dialogs/                  # 模态对话框
│   │       ├── DeleteConfirmDialog.vue
│   │       ├── NewItemDialog.vue
│   │       ├── RenameDialog.vue
│   │       └── SettingsDialog.vue
│   ├── composables/                  # 组合式函数 (7个)
│   │   ├── useContextMenu.ts
│   │   ├── useDragDrop.ts
│   │   ├── useFileActions.ts
│   │   ├── useKeyboardShortcuts.ts
│   │   ├── usePanelNavigation.ts
│   │   ├── useSearchService.ts
│   │   └── useToast.ts
│   ├── stores/                       # Pinia 状态仓库 (6个)
│   │   ├── fileStore.ts
│   │   ├── tabStore.ts
│   │   ├── selectionStore.ts
│   │   ├── viewStore.ts
│   │   ├── navigationStore.ts
│   │   └── settingsStore.ts
│   ├── locales/                      # 国际化
│   │   ├── en.ts                     # English
│   │   └── zh.ts                     # 简体中文
│   ├── types/
│   │   └── index.ts
│   ├── utils/
│   │   ├── fileIcons.ts
│   │   ├── fileTypes.ts
│   │   ├── session.ts               # 会话持久化
│   │   └── tauri.ts                  # Tauri IPC 封装
│   ├── App.vue                       # 根组件（事件编排 + 预览窗口路由）
│   ├── i18n.ts
│   ├── main.ts
│   └── style.css
│
├── src-tauri/                        # 后端 (Rust)
│   ├── src/                          # 12 个模块
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   ├── tests/
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
│
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```

---

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org) ≥ 18
- [Rust](https://rustup.rs) ≥ 1.77
- [Tauri 2.0 系统依赖](https://v2.tauri.app/start/prerequisites/)

**macOS 额外依赖：**
```bash
xcode-select --install
```

**Linux 额外依赖：**
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

### 开发模式

```bash
npm install
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

### 构建产物

```
src-tauri/target/release/bundle/dmg/Files Explorer_0.1.4_x64.dmg   # macOS DMG
src-tauri/target/release/bundle/macos/Files Explorer.app           # macOS .app
src-tauri/target/release/files-explorer.exe                        # Windows
```

---

## 🧩 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.4 | UI 框架 |
| Pinia | 2 | 状态管理 |
| @tanstack/vue-virtual | 3 | 虚拟滚动 |
| vue-i18n | 9 | 国际化 |
| Shiki | 4 | 代码语法高亮（VS Code 引擎） |
| @vue-office/docx | — | DOCX 文档预览 |
| @vue-office/excel | — | XLSX 表格预览 |
| @vue-office/pdf | — | PDF 预览 |
| pptx-preview | — | PPTX 幻灯片预览 |
| marked | 18 | Markdown 渲染 |
| DOMPurify | 3 | XSS 防护 |
| Vite | 5 | 构建工具 |

### Rust 后端关键依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| `tauri` | 2 | 桌面框架 |
| `walkdir` | 2 | 递归目录遍历 |
| `zip` | 2 | ZIP 压缩/解压/浏览 |
| `tar` | 0.4 | TAR 归档 |
| `flate2` | 1 | Gzip 压缩/解压 |
| `bzip2` | 0.4 | Bzip2 解压 |
| `xz2` | 0.1 | XZ/LZMA 解压 |
| `sevenz-rust` | 0.6 | 7z 归档（纯 Rust） |
| `trash` | 3 | 回收站操作 |
| `opener` | 0.7 | 文件打开 |
| `arboard` | 3 | 系统剪贴板 |
| `image` | 0.25 | 图标编码 |
| `log` / `simplelog` | 0.4 / 0.12 | 日志系统 |

---

## 📄 许可证

MIT License. 详见 [LICENSE](LICENSE) 文件。

---

<div align="center">
  使用 ❤️ 和 Tauri + Vue + Rust 构建
</div>
