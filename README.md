<div align="center">
  <img src="src-tauri/icons/icon.pngfiles-explorer/blob/master/screenshots/logo/应用程序图标设计1024x.png" alt="Files Explorer" width="80" height="80" />
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

Files Explorer 重新思考了桌面文件管理器的体验：

- **⚡ 轻量** — Tauri 2.0 原生二进制不到 10MB，内存占用仅为 Electron 同类应用的 **1/10**
- **🚀 极速** — Rust 后端零开销文件操作 + 虚拟滚动 + 流式加载，万级文件目录保持 60fps
- **🎨 现代** — Catppuccin 暗/亮双主题、无边框窗口、Fluid UI 图标体系、5 种视图模式
- **🧠 智能** — 50+ 语言代码高亮、Office/PDF/归档文件内联预览、通配符+内容全文搜索、跨窗口状态同步

---

## 🖥️ 平台支持

| 平台 | 状态 | 架构 |
|------|------|------|
| **Windows** | ✅ 完全支持 | x86_64 |
| **macOS** | ✅ 完全支持 | x86_64 + ARM64 |
| **Linux** | ✅ 完全支持 | x86_64 |

---

## ✨ 核心功能

### 📂 文件浏览

| 功能 | 说明 |
|------|------|
| **5 种视图** | 详细信息 / 列表 / 网格 / 树形 / 分栏（Miller Columns）|
| **虚拟滚动** | 基于 @tanstack/vue-virtual，万级文件仅渲染可见行 |
| **流式加载** | 100 条/批推送，大目录即时响应不卡顿 |
| **地址栏补全** | 输入路径实时匹配子目录，下拉快捷跳转 |
| **面包屑导航** | 支持 Windows 盘符和 Unix 路径的分段导航 |
| **历史导航** | 后退/前进/向上，保留 50 条历史快照 |

### 🖥️ 多面板工作流

| 功能 | 说明 |
|------|------|
| **多标签页** | 复制 IDE 标签体验，`Ctrl+W` 关闭，拖拽切换 |
| **分屏面板** | 水平/垂直任意拆分，分隔线可拖拽，最多嵌套 4 层 |
| **独立状态** | 每个标签页独立记忆路径、文件列表、树展开、搜索、选中状态 |
| **拖放支持** | 文件拖入目录、拖到标签页切换、Windows 原生拖出（COM DragDrop）|

### 🔍 智能预览

| 预览类型 | 能力 |
|----------|------|
| **代码高亮** | 50+ 语言（CodeMirror 5），行号、选区、主题适配 |
| **Markdown** | 编辑/预览双模式，Shiki 高亮代码块，DOMPurify XSS 防护，支持导出 HTML/Word/PDF |
| **Office 文档** | docx / xlsx / pptx 内联渲染 |
| **PDF** | 缩放/拖拽平移预览 |
| **图片** | 缩放/旋转/拖拽平移，自动读取尺寸 |
| **归档文件** | 浏览 ZIP/7z/RAR/TAR 内容，支持单条目解压预览 |
| **独立预览窗口** | 文件树导航 + 多功能工具栏（打开/重命名/另存为/删除/复制路径/在 Finder 中定位）|

### 📋 文件操作

| 操作 | 说明 |
|------|------|
| **新建/删除** | 文件/文件夹创建，支持普通删除和永久删除 |
| **剪切/复制/粘贴** | 内部剪贴板 + 系统原生剪贴板（Windows CF_HDROP / macOS NSPasteboard）|
| **重命名** | 智能选中（不含扩展名），冲突检测 |
| **属性** | 类型/大小/修改日期/创建日期/完整路径，路径可点击在资源管理器中定位 |
| **压缩/解压** | ZIP 压缩（带进度），ZIP/TAR/TAR.GZ 解压（含 ZipSlip 路径穿越防护）|
| **收藏夹** | 一键添加/移除，持久化存储 |
| **原生拖出** | Windows 平台支持向 QQ/微信/Chrome 等外部应用拖出文件 |

### 🔍 高级搜索

| 能力 | 说明 |
|------|------|
| **通配符** | `*.rs` / `test*` / `report?.pdf` |
| **大小过滤** | `>10MB` / `<1KB` 管道组合 |
| **全文搜索** | 自动跳过 `node_modules` 等 17 个黑名单目录，≤1MB 文件内容检索 |
| **流式结果** | 每条结果实时推送，可随时取消，上限 2000 条 |

### ⏪ 撤销系统

支持创建/重命名/复制/剪切操作的撤销（删除不可撤销），保留 50 条历史。

### 💾 会话持久化

关闭应用时自动保存全部布局（面板/Tab/路径/树展开状态），下次启动无缝恢复。

### ⌨️ 全局快捷键

| 快捷键 | 功能 |
|--------|------|
| `Enter` | 打开选中文件 |
| `Ctrl+C/X/V` | 复制/剪切/粘贴 |
| `Ctrl+A` | 全选 |
| `Delete` | 删除至回收站 |
| `Shift+Delete` | 永久删除 |
| `F2` | 重命名 |
| `F5` | 刷新 |
| `Ctrl+P` | 切换属性面板 |
| `Ctrl+N` | 新建文件 |
| `Ctrl+Shift+N` | 新建文件夹 |
| `Ctrl+Z` | 撤销 |
| `Ctrl+W` | 关闭当前标签页 |
| `Ctrl+[/]` | 历史导航前进/后退 |
| `Backspace` | 向上导航 |

### 🎨 视觉设计

| 特性 | 说明 |
|------|------|
| **暗/亮双主题** | Catppuccin Mocha & Latte，`Ctrl+Shift+T` 切换 |
| **无边框窗口** | 自绘标题栏，macOS 红绿灯 / Windows 最小化最大化关闭 |
| **Fluid UI 图标** | 按文件类别（文档/代码/图片/音视频/压缩包/可执行文件）着色渲染 |
| **字体调节** | 小/中/大三档，全局生效 |

### 🌐 国际化

中文（简体）、英语，覆盖 180+ 翻译 Key。

---

## 🏗️ 架构设计

```
┌──────────────────────────────────────────────────────┐
│                  Vue 3 前端层                         │
│  ┌─────────┐ ┌─────────┐ ┌────────────┐             │
│  │ 6 Stores│ │7 Compos.│ │ 26 Comp.   │             │
│  │ (Pinia) │ │ (Hooks) │ │ (Views)    │             │
│  └────┬────┘ └────┬────┘ └─────┬──────┘             │
│       └───────────┼────────────┘                    │
│                   │ invoke()                          │
├───────────────────┼──────────────────────────────────┤
│                   ▼                                   │
│              31 Tauri Commands                        │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐       │
│  │files │ │ ops  │ │search│ │ clip │ │ sys  │  ...  │
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘       │
│     └────────┼────────┼────────┼────────┘            │
│              ▼        ▼        ▼                      │
│         12 Rust 模块 (零成本抽象)                       │
├───────────────────────────────────────────────────────┤
│           平台抽象层                                    │
│  Windows (Win32) │ macOS (Foundation) │ Linux (POSIX) │
└───────────────────────────────────────────────────────┘
```

### 前端六仓库模式（Six-Stores）

| Store | 职责 |
|-------|------|
| `fileStore` | 核心数据：路径/文件列表/驱动器/最近访问/撤销状态 |
| `navigationStore` | 历史记录：前进/后退，容量 50 条 |
| `selectionStore` | 文件选择：单选/多选/全选，剪切状态 |
| `tabStore` | 布局管理：Panel/Tab 树形结构，增删切分 |
| `viewStore` | 视图模式：5 种视图 + 树展开/列加载 |
| `settingsStore` | 持久化设置：主题/语言/字体/书签 |

### Rust 后端模块（12 个）

| 模块 | 职责 |
|------|------|
| `files` | 目录列表（同步 + 流式事件推送）|
| `operations` | 文件 CRUD（创建/删除/重命名/移动/复制）|
| `clipboard` | 多层剪贴板（内部 + 系统原生 CF_HDROP / NSPasteboard）|
| `search` | 搜索引擎（通配符 + 全文内容，流式结果）|
| `system` | 系统集成（打开/终端/打印/预览/图标/归档）|
| `drives` | 磁盘枚举（跨平台 Win32 / statvfs）|
| `compress` | 压缩/解压（ZIP/TAR/GZ，ZipSlip 防护）|
| `undo` | 撤销系统（最多 50 条，删除不可逆）|
| `native_drag` | Windows COM 原生拖放 |
| `error` | 统一错误类型（6 种变体）|
| `state` | 全局状态管理（Mutex 保护）|
| `types` | 共享数据结构 |

### 安全机制

| 防护 | 措施 |
|------|------|
| **路径穿越 (ZipSlip)** | 拒绝 `..` 和绝对路径 |
| **文件大小限制** | 图片 2MB / Office 20MB / 文本 1MB / 搜索 1MB |
| **归档解压上限** | 总大小 2GB，单条目 20MB，最多 10000 条目 |
| **命令超时** | 外部命令（7z/unrar 等）30~60 秒超时 kill |
| **XSS 防护** | Markdown 预览使用 DOMPurify |
| **搜索黑名单** | 自动跳过 17 个非用户目录 |
| **并发安全** | Mutex + AtomicBool + AtomicU64 |

---

## 📁 项目结构

```
files-explorer/
├── src/                          # Vue 3 前端
│   ├── components/               # 26 个 Vue 组件
│   │   ├── Dialogs/              # 弹窗组件
│   │   ├── App.vue               # 应用根组件
│   │   ├── FileList.vue          # 文件列表容器
│   │   ├── PropertiesPanel.vue   # 属性/预览面板
│   │   └── PreviewWindow.vue     # 独立预览窗口
│   ├── stores/                   # 6 个 Pinia Store
│   ├── composables/              # 7 个组合式函数
│   ├── utils/                    # 工具函数
│   │   ├── tauri.ts              # 背端 API 封装 (40+ 函数)
│   │   ├── fileTypes.ts          # 文件类型分类 (130+ 扩展名)
│   │   ├── fileIcons.ts          # Fluid UI 图标 (12 种)
│   │   └── session.ts            # 会话持久化
│   ├── locales/                  # i18n
│   │   ├── zh.ts                 # 中文 (180+ keys)
│   │   └── en.ts                 # English (180+ keys)
│   └── types/                    # TypeScript 类型定义
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # 入口 + 31 Commands 注册
│   │   └── main.rs               # 二进制入口 + 日志系统
│   ├── tauri.conf.json           # Tauri 配置
│   └── capabilities/default.json # 权限声明
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
| Linux | `webkit2gtk-4.1` `libgtk-3` `libayatana-appindicator3` |

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
# 生产构建 + 打包
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

| 技术 | 版本 |
|------|------|
| Vue 3 (Composition API) | ^3.4 |
| Vite | ^5.0 |
| TypeScript (strict) | ^5.3 |
| Pinia | ^2.1 |
| vue-i18n | ^9.14 |
| @tanstack/vue-virtual | ^3.13 |
| CodeMirror 5 | ^5.65 |
| Shiki | ^4.3 |
| marked + DOMPurify | ^18 / ^3.4 |
| @vue-office (docx/excel/pdf) | — |

### Rust 后端

| Crate | 用途 |
|-------|------|
| `tauri` 2.x | 桌面应用框架 |
| `walkdir` | 递归目录遍历 |
| `zip` / `tar` / `flate2` / `bzip2` / `xz2` / `sevenz-rust` | 压缩/归档 |
| `trash` | 跨平台回收站 |
| `arboard` | 系统剪贴板 |
| `image` | 图标 PNG 编码 |
| `objc2` (macOS) | NSPasteboard 原生集成 |
| `serde` / `serde_json` | 序列化 |

---

## 📄 许可证

MIT License

---

<div align="center">
  <sub>Built with ❤️ using Tauri + Vue 3 + Rust</sub>
</div>
