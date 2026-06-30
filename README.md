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
- **🧠 智能** — 50+ 语言代码高亮、Office/PDF/归档预览、通配符搜索、系统托盘常驻

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
| **图片** | 缩放/旋转/拖拽平移，自动读取尺寸 |
| **归档文件** | 浏览 ZIP/7z/RAR/TAR 内容，单条目解压预览 |
| **独立预览窗口** | 文件树导航 + 工具栏（打开/重命名/另存为/删除/复制路径/定位）|

### 📋 文件操作

| 操作 | 说明 |
|------|------|
| **新建/删除** | 文件/文件夹创建，回收站/永久删除 |
| **剪切/复制/粘贴** | 内部剪贴板 + 系统原生（CF_HDROP / NSPasteboard）|
| **重命名** | 智能选中（不含扩展名），冲突检测 |
| **压缩/解压** | ZIP 压缩（带进度），ZIP/TAR/TAR.GZ 解压（ZipSlip 防护）|
| **收藏夹** | 一键添加/移除，持久化存储 |
| **原生拖出** | Windows 向 QQ/微信/Chrome 等外部应用拖出文件 |

### 🔍 高级搜索

| 能力 | 说明 |
|------|------|
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
| **快速访问** | 桌面/下载/文档/图片/音乐/视频一键直达 |
| **托盘行为** | 关闭窗口 → 隐藏到托盘（可设退出）|
| **左键点击** | 恢复主窗口 |
| **右键菜单** | 快速访问 + 设置 + 显示/退出 |

### ⚙️ 设置

| 分类 | 设置项 |
|------|--------|
| **外观** | 7 主题 / 字体大小 / 3 图标风格 |
| **语言** | 中文 / English |
| **通用** | 随系统启动 / 显示系统托盘 / 关闭时退出 |
| **关于** | 版本号（自动读取 package.json）/ 技术栈 |

### 🔄 原生 Splashscreen

Tauri 独立启动窗口，前后端就绪后无感切换到主窗口，零白屏、零闪烁。

### 其他特性

- ⏪ **撤销系统** — 创建/重命名/复制/剪切可撤销，50 条历史
- 💾 **会话持久化** — 布局/Tab/历史/面板全部恢复
- ⌨️ **30+ 快捷键** — Enter/Ctrl+CXV/F2/F5/Ctrl+NZ/Ctrl+W...
- 🌐 **国际化** — 中文/English，200+ 翻译 Key
- 📐 **自适应窗口** — 屏幕 65%×75%，最小 1024×680

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
│            34 Tauri Commands                          │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐       │
│  │files │ │ ops  │ │search│ │ clip │ │ sys  │  ...  │
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘       │
│     └────────┼────────┼────────┼────────┘            │
│              ▼        ▼        ▼                      │
│         14 Rust 模块 (零成本抽象)                       │
├───────────────────────────────────────────────────────┤
│   platform: Windows (Win32) │ macOS │ Linux (POSIX)   │
│   tray: TrayIconBuilder     │ autostart: 3 平台        │
└───────────────────────────────────────────────────────┘
```

### Rust 后端模块（14 个）

| 模块 | 职责 |
|------|------|
| `files` | 目录列表（同步 + 流式事件）|
| `operations` | 文件 CRUD |
| `clipboard` | 多层剪贴板（内部+系统原生）|
| `search` | 搜索引擎（通配符+流式结果）|
| `system` | 系统集成（打开/终端/打印/预览/图标/归档）|
| `drives` | 磁盘枚举（跨平台 Win32/statvfs）|
| `compress` | 压缩/解压（ZIP/TAR/GZ，ZipSlip 防护）|
| `undo` | 撤销系统（50 条） |
| `tray` | 系统托盘（菜单+事件） |
| `autostart` | 三平台自启动 |
| `native_drag` | Windows COM 原生拖放 |
| `error` | 统一错误类型 |
| `state` | 全局状态（Mutex 保护）|
| `types` | 共享数据结构 |

### 安全机制

| 防护 | 措施 |
|------|------|
| ZipSlip | 拒绝 `..` 和绝对路径 |
| 文件大小 | 图片 2MB / Office 20MB / 文本 1MB |
| 归档 | 总 2GB / 单条目 20MB / 10000 条目 |
| 命令超时 | 外部命令 30~60 秒 kill |
| XSS | Markdown DOMPurify |
| 并发 | Mutex + AtomicBool + AtomicU64 |

---

## 📁 项目结构

```
files-explorer/
├── src/                          # Vue 3 前端
│   ├── components/               # 26 个 Vue 组件
│   │   ├── Dialogs/              # 弹窗组件
│   ├── stores/                   # 6 个 Pinia Store
│   ├── composables/              # 7 个组合式函数
│   ├── utils/                    # 工具函数
│   │   ├── tauri.ts              # 后端 API 封装 (40+)
│   │   ├── fileTypes.ts          # 文件类型分类 (130+)
│   │   ├── fileIcons.ts          # 三主题图标引擎
│   │   ├── iconMap.ts            # 300+ 文件映射
│   │   └── session.ts            # 会话持久化
│   ├── locales/                  # i18n (zh/en)
│   └── types/                    # TypeScript 类型
├── src-tauri/                    # Rust 后端
│   ├── src/                      # 14 模块
│   │   ├── lib.rs                # 34 Commands + Splashscreen
│   │   ├── tray.rs               # 系统托盘
│   │   └── autostart.rs          # 自启动
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

| 技术 | 版本 |
|------|------|
| Vue 3 (Composition API) | ^3.4 |
| Vite + TypeScript (strict) | ^5 / ^5.3 |
| Pinia (6 stores) | ^2.1 |
| vue-i18n | ^9.14 |
| @tanstack/vue-virtual | ^3.13 |
| CodeMirror 5 | ^5.65 |
| Shiki + marked + DOMPurify | ^4 / ^18 / ^3.4 |
| @vue-office (docx/excel/pdf) | — |
| material-icon-theme | — |

### Rust 后端

| Crate | 用途 |
|-------|------|
| `tauri` 2.x (tray-icon) | 桌面框架 + 系统托盘 |
| `walkdir` | 目录遍历 |
| `zip`/`tar`/`flate2`/`bzip2`/`xz2`/`sevenz-rust` | 压缩归档 |
| `trash` | 回收站 |
| `arboard` | 系统剪贴板 |
| `image` | 图标编码 |
| `objc2` (macOS) | 原生集成 |
| `serde`/`serde_json` | 序列化 |

---

## 📄 许可证

MIT License

---

<div align="center">
  <sub>Built with ❤️ using Tauri + Vue 3 + Rust</sub>
</div>
