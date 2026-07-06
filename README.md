<div align="center">
  <img src="screenshots/logo/应用程序图标设计1024x.png" alt="Files Explorer" width="80" height="80" />
  <h1>Files Explorer</h1>
  <p>
    <strong>轻量 · 极速 · 现代 · 智能</strong><br/>
    基于 <strong>Tauri 2.0</strong> + <strong>Vue 3</strong> 的跨平台桌面文件资源管理器
  </p>
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

## 💡 设计理念

- **⚡ 轻量** — Tauri 2.0 原生二进制，内存占用仅为 Electron 同类应用的 **1/10**
- **🚀 极速** — Rust 后端零开销文件操作 + 虚拟滚动 + 流式加载，万级文件 60fps
- **🎨 现代** — 7 大主题 × 3 图标风格、无边框窗口、Tauri 原生 Splashscreen
- **🧠 智能** — 内置终端 (PTY)、50+ 语言代码高亮、Office/PDF/视频/归档预览、通配符搜索、系统托盘

> 📖 **完整用户手册**：请参阅 [docs/user-manual.md](docs/user-manual.md)

---

## 🖥️ 平台支持

| 平台 | 状态 | 架构 |
|------|------|------|
| **Windows** | ✅ | x86_64 |
| **macOS** | ✅ | x86_64 + ARM64 |
| **Linux** | ✅ | x86_64 |

---

## ✨ 核心功能

### 📂 文件浏览

| 功能 | 说明 |
|------|------|
| **5 种视图** | 详细信息 / 列表 / 网格 / 树形 / 分栏（Miller Columns）|
| **虚拟滚动** | @tanstack/vue-virtual，万级文件仅渲染可见行 |
| **流式加载** | 100 条/批推送，大目录即时响应 |
| **地址栏补全** | 输入路径实时匹配子目录，下拉跳转 |
| **面包屑导航** | Windows 盘符 / Unix 路径分段导航 |
| **历史导航** | 后退/前进/向上，50 条历史快照 |

### 🖥️ 多面板工作流

| 功能 | 说明 |
|------|------|
| **多标签页** | IDE 标签体验，Ctrl+W 关闭，拖拽切换 |
| **分屏面板** | 水平/垂直拆分，分隔线可拖拽 |
| **独立状态** | 每个标签页独立记忆路径/列表/树展开/搜索 |
| **拖放支持** | 文件拖入目录、标签切换、Windows COM 拖出 |

### 🔍 智能预览

| 预览类型 | 能力 |
|----------|------|
| **代码高亮** | 50+ 语言 CodeMirror 5，行号/选区/主题适配 |
| **Markdown** | 编辑/预览，Shiki 高亮，DOMPurify XSS 防护，导出 HTML/Word/PDF |
| **Office 文档** | docx / xlsx / pptx 内联渲染 |
| **PDF** | 缩放/拖拽平移 |
| **视频** | DPlayer 播放器，支持 MP4/WebM/FLV，截图/热键/速度调节 |
| **图片** | 缩放/旋转/拖拽平移，自动读取尺寸 |
| **归档文件** | 浏览 ZIP/7z/RAR/TAR 内容，单条目解压预览 |
| **编码检测** | 不支持编码自动提示，建议外部播放器打开 |
| **独立预览窗口** | 65%×75% 自适应尺寸，文件树导航 + 工具栏 |

### 💻 内置终端

| 功能 | 说明 |
|------|------|
| **PTY 伪终端** | portable-pty 跨平台，macOS zsh / Linux bash / Windows PowerShell |
| **完整 ANSI** | 7 套主题各含 16 色调色板，`ls --color` / `git diff` 完美渲染 |
| **面板操作** | 最大化/还原 (`⊠`/`🗗`)、高度拖拽、双击 header 最大化、Esc 还原 |
| **字体缩放** | `Ctrl+=` / `Ctrl+-` / `Ctrl+0`，header `+` `−` 按钮 |
| **键盘映射** | `Ctrl+C` → SIGINT（无选中时），`Ctrl+V` → 粘贴剪贴板 |
| **目录跟随** | 文件浏览切换目录时终端自动 `cd` |
| **退出恢复** | 进程退出覆盖层提示，一键重启 |
| **入口** | `` Ctrl+` `` 快捷键 / StatusBar `>_` 按钮 |

### 📋 文件操作

| 操作 | 说明 |
|------|------|
| **新建/删除** | 文件/文件夹创建，回收站/永久删除 |
| **剪切/复制/粘贴** | 内部剪贴板 + 系统原生（CF_HDROP / NSPasteboard）|
| **复制路径** | 右键菜单 + 快捷键，多文件换行分隔 |
| **重命名** | 智能选中（不含扩展名），冲突检测 |
| **压缩/解压** | ZIP 压缩（带进度），ZIP/TAR/GZ/7z/RAR 解压（ZipSlip 防护）|
| **收藏夹** | 一键添加/移除，持久化存储 |
| **原生拖出** | Windows 向 QQ/微信/Chrome 等外部应用拖出文件 |

### 🔍 高级搜索

| 能力 | 说明 |
|------|------|
| **搜索历史** | 最近 15 条持久化，↑↓ 选择，去重 |
| **通配符** | `*.rs` / `test*` / `report?.pdf` |
| **大小过滤** | `>10MB` / `<1KB` 管道组合 |
| **流式结果** | 实时推送，可随时取消，上限 2000 条 |
| **黑名单过滤** | 自动跳过 17 个非用户目录 |

### 🎨 7 大主题 × 3 图标风格

| 主题 | 风格 |
|------|------|
| **Catppuccin Mocha/Latte** | 柔暗 / 柔亮 |
| **Nord** | 极简冷暗 |
| **Tokyo Night** | 现代暗色 |
| **One Dark Pro** | 经典暗色 |
| **Dracula** | 暗紫 |
| **Solarized Light** | 复古暖亮 |

| 图标风格 | 说明 |
|----------|------|
| **Fluent** (默认) | 12 种自绘文件形状 SVG |
| **Material** | 300+ 算法生成彩色分类图标 |
| **Material+** | 1250 Material Design 官方 SVG |

### 🖥️ 系统托盘

| 功能 | 说明 |
|------|------|
| **托盘图标** | 应用程序图标，跨平台菜单栏/任务栏 |
| **纯文字菜单** | macOS 原生风格，无 Emoji |
| **动态切换** | 显示/隐藏主窗口菜单项实时切换 |
| **快速访问** | 下载/文档一键直达 |
| **托盘行为** | 关闭窗口 → 隐藏到托盘（可设退出）|
| **单击/双击** | 单击 toggle 显隐 / 双击强制显示+聚焦 |
| **清理缓存** | 托盘菜单 + 设置双入口 |
| **Dock/任务栏** | 点击重新显示主窗口 |

### ⚙️ 设置

| 分类 | 设置项 |
|------|--------|
| **外观** | 7 主题 / 字体大小 / 3 图标风格 |
| **语言** | 中文 / English |
| **通用** | 随系统启动 / 显示系统托盘 / 关闭时退出 |
| **关于** | 版本号 / 清理缓存 |

### 🔄 原生 Splashscreen

Tauri 独立启动窗口，前后端就绪后无感切换到主窗口，零白屏、零闪烁。

### 其他特性

- ⏪ **撤销系统** — 创建/重命名/复制/剪切可撤销，50 条历史
- 💾 **会话持久化** — 窗口几何/布局/Tab/历史/面板全部恢复
- 🧹 **缓存清理** — 一键重置所有缓存数据
- ⌨️ **35+ 快捷键** — 含终端专用 6 个
- 🌐 **国际化** — 中文/English，200+ 翻译 Key

---

## 🏗️ 架构设计

```
┌──────────────────────────────────────────────────────┐
│                  Vue 3 前端层                         │
│  ┌─────────┐ ┌─────────┐ ┌────────────────┐         │
│  │ 6 Stores│ │7 Compos.│ │ 27 Components  │         │
│  └────┬────┘ └────┬────┘ └─────┬──────────┘         │
│       └───────────┼────────────┘                    │
│                   │ invoke()                          │
├───────────────────┼──────────────────────────────────┤
│                   ▼                                   │
│            39 Tauri Commands                          │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌────┐│
│  │files │ │ ops  │ │search│ │term  │ │ clip │ │sys ││
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬─┘│
│     └────────┼────────┼────────┼────────┼───────┘   │
│              ▼        ▼        ▼        ▼            │
│         15 Rust 模块                                  │
├───────────────────────────────────────────────────────┤
│   platform: Windows (Win32) │ macOS │ Linux (POSIX)   │
└───────────────────────────────────────────────────────┘
```

### Rust 后端模块（15 个）

| 模块 | 职责 |
|------|------|
| `files` | 目录列表（同步 + 流式事件）|
| `operations` | 文件 CRUD |
| `clipboard` | 多层剪贴板（内部+系统原生）|
| `search` | 搜索引擎（通配符+流式结果）|
| `system` | 系统集成（打开/终端/打印/预览/图标）|
| `drives` | 磁盘枚举（跨平台）|
| `compress` | 压缩/解压（ZIP/TAR/GZ/BZ2/XZ/7z/RAR，ZipSlip 防护）|
| `undo` | 撤销系统（50 条） |
| `tray` | 系统托盘（菜单+事件+动态切换） |
| `autostart` | 三平台自启动 |
| `native_drag` | Windows COM 原生拖放 |
| **`terminal`** | **内置终端**（PTY spawn/write/resize/kill）|
| `error` | 统一错误类型 (AppError) |
| `state` | 全局状态（Mutex + AtomicBool）|
| `types` | 共享数据结构 |

### Tauri 插件

| 插件 | 用途 |
|------|------|
| `tauri-plugin-dialog` | 文件对话框 |
| `tauri-plugin-shell` | 外部程序调用 |
| `tauri-plugin-single-instance` | 单例运行 |
| `tauri-plugin-window-state` | 窗口几何自动保存/恢复 |

### 安全机制

| 防护 | 措施 |
|------|------|
| ZipSlip | 拒绝 `..` 和绝对路径 |
| 文件大小 | 图片 2MB / Office 20MB / 文本 512KB |
| 归档 | 总 2GB / 单条目 20MB / 10000 条目 |
| 命令超时 | 外部命令 30~60 秒 kill |
| XSS | Markdown DOMPurify |
| 并发 | Mutex + AtomicBool + AtomicU64 |

---

## 📁 项目结构

```
files-explorer/
├── src/                          # Vue 3 前端
│   ├── components/               # 27 个 Vue 组件
│   │   ├── TerminalPanel.vue     # 内置终端面板
│   │   ├── Dialogs/              # 弹窗组件
│   │   └── ...
│   ├── stores/                   # 6 个 Pinia Store
│   ├── composables/              # 7 个组合式函数
│   ├── utils/                    # 工具函数
│   │   ├── tauri.ts              # 后端 API 封装 (40+)
│   │   ├── platform.ts           # 跨平台工具 + 路径处理
│   │   ├── fileTypes.ts          # 文件类型分类 (130+)
│   │   ├── fileIcons.ts          # 三主题图标引擎
│   │   ├── iconMap.ts            # 300+ 文件映射
│   │   └── session.ts            # 会话持久化
│   ├── locales/                  # i18n (zh/en)
│   └── types/                    # TypeScript 类型
├── src-tauri/                    # Rust 后端
│   ├── src/                      # 15 模块
│   │   ├── lib.rs                # 39 Commands
│   │   ├── terminal.rs           # PTY 终端管理器
│   │   ├── tray.rs               # 系统托盘
│   │   └── ...
│   ├── tauri.conf.json           # Tauri 配置
│   └── capabilities/             # 权限声明
├── public/
│   ├── splashscreen.html         # Tauri 原生启动画面
│   └── icons/                    # Material+ SVG (构建时生成)
├── scripts/
│   ├── generate-icons.cjs        # 平台图标生成
│   └── copy-material-icons.cjs   # Material SVG 复制
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## 🚀 快速开始

### 环境要求

| 依赖 | 版本 |
|------|------|
| Node.js | ≥ 18 |
| Rust | ≥ 1.77 |
| macOS | Xcode Command Line Tools |
| Linux | `webkit2gtk-4.1` `libgtk-3` |

### 开发

```bash
npm install
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

### 构建产物

| 平台 | 产物 |
|------|------|
| **macOS** | `src-tauri/target/release/bundle/dmg/Files Explorer_*.dmg` |
| **Windows** | `src-tauri/target/release/bundle/msi/Files Explorer_*.msi` |
| **Linux** | `src-tauri/target/release/bundle/deb/files-explorer_*.deb` |

---

## 🧩 核心技术栈

### 前端

| 技术 | 用途 |
|------|------|
| Vue 3 (Composition API) | UI 框架 |
| Vite + TypeScript | 构建 + 类型安全 |
| Pinia | 状态管理 (6 stores) |
| vue-i18n | 国际化 |
| @tanstack/vue-virtual | 虚拟滚动 |
| **@xterm/xterm** + **@xterm/addon-fit** | **内置终端** |
| CodeMirror 5 | 代码高亮 |
| DPlayer | 视频播放 |
| Shiki + marked + DOMPurify | Markdown 渲染 |
| @vue-office (docx/excel/pdf) | Office 文档预览 |
| material-icon-theme | 文件图标 |

### Rust 后端

| Crate | 用途 |
|-------|------|
| `tauri` 2.x (tray-icon) | 桌面框架 + 系统托盘 |
| `tauri-plugin-dialog` | 文件对话框 |
| `tauri-plugin-shell` | 外部程序调用 |
| `tauri-plugin-single-instance` | 单例运行 |
| `tauri-plugin-window-state` | 窗口状态持久化 |
| **`portable-pty`** | **跨平台伪终端** |
| `walkdir` | 目录遍历 |
| `zip`/`tar`/`flate2`/`bzip2`/`xz2`/`sevenz-rust` | 压缩归档 |
| `trash` | 回收站 |
| `arboard` | 系统剪贴板 |
| `image` | 图标编码 |
| `objc2` (macOS) | 原生集成 |
| `serde`/`serde_json` | 序列化 |

---

## 📜 更新日志

### v0.2.0

- 💻 **内置终端** — portable-pty + xterm.js，7 套主题完整 ANSI 16 色
- 🎨 **图标优化** — 工具栏 stroke 风格统一，状态栏图标更新
- 🏗️ **平台抽象重构** — `#[cfg(target_os)]` 收敛至 `platform/`
- 🎯 **统一错误类型** — 全部 `Result<T, AppError>`
- ⭐ **收藏夹增强** — 右键重命名/移除
- 🧭 **路径标准化** — `normalizePath()` / `displayPath()`
- 🐛 **面包屑修复** — Unix 根路径 `/` 导航正确
- 🐛 **编译修复** — JSON 逗号缺失、macOS `Manager` import

### v0.1.5

- 🎬 DPlayer 视频播放器
- 🪟 窗口状态持久化
- 💾 会话持久化（三条退出路径）
- 🔍 搜索历史（15 条）
- 📋 复制路径功能
- 🖥️ 托盘菜单动态切换
- 🧹 清理缓存双入口
- 🍎 macOS Dock / Windows 任务栏恢复
- ⌨️ 键盘导航（↑↓←→/Home/End/PgUp/PgDn）
- 🌳 树形键盘导航
- 📂 收藏夹优化

---

## 📄 许可证

MIT License

---

<div align="center">
  <sub>Built with ❤️ using Tauri + Vue 3 + Rust</sub>
</div>
