# Files Explorer User Manual

## Table of Contents

1. [Interface Layout](#1-interface-layout)
2. [File Browsing](#2-file-browsing)
3. [File Operations](#3-file-operations)
4. [Multi-Pane Workflow](#4-multi-pane-workflow)
5. [Preview Features](#5-preview-features)
6. [Built-in Terminal](#6-built-in-terminal)
7. [Search](#7-search)
8. [Favorites](#8-favorites)
9. [System Tray](#9-system-tray)
10. [Keyboard Shortcuts](#10-keyboard-shortcuts)
11. [Settings](#11-settings)

---

<div align="center">
  <img src="../screenshots/dark.png" alt="Dark theme interface" width="90%" />
  <br/>
  <em>Dark theme (Catppuccin Mocha)</em>
  <br/><br/>
  <img src="../screenshots/light.png" alt="Light theme interface" width="90%" />
  <br/>
  <em>Light theme (Catppuccin Latte)</em>
  <br/><br/>
  <img src="../screenshots/dark1.png" alt="File browsing interface" width="90%" />
  <br/>
  <em>File browsing and preview panel</em>
  <br/><br/>
  <img src="../screenshots/dark2.png" alt="Standalone preview window" width="90%" />
  <br/>
  <em>Standalone preview window</em>
</div>

## 1. Interface Layout

```
┌──────────────────────────────────────────────────┐
│  TitleBar (custom title bar)                       │
├──────────────────────────────────────────────────┤
│  Toolbar (← → ↑ ↻ | Address bar | Search)         │
├──────────────────────────────────────────────────┤
│  RibbonToolbar (New/Cut/Copy/Paste/Delete)         │
├────────┬─────────────────────┬──────────────────┤
│ Sidebar│    PanesArea        │  PropertiesPanel │
│ ─────  │  ┌────────────────┐ │  ┌──────────────┐│
│ This PC│  │ Tab1 │ Tab2 │+│ │  │ Preview area  ││
│ Drives │  ├────────────────┤ │  │              ││
│ Quick  │  │ Breadcrumb     │ │  │ ──────────── ││
│ Access │  ├────────────────┤ │  │ Properties   ││
│ Favor. │  │ FileList       │ │  │              ││
│        │  └────────────────┘ │  └──────────────┘│
├────────┴─────────────────────┴──────────────────┤
│  ┌─ Terminal Panel ─ Ctrl+` ──────────────────┐  │
│  │ ▸ zsh ｜ ~/Documents  ● 13px [−][+][⊠][⟳][✕]│  │
│  │ apple@mac ~/D $ █                          │  │
│  └────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────┤
│  StatusBar (/path  [>_] [▯│] [N items])          │
└──────────────────────────────────────────────────┘
```

### Area Descriptions

| Area | Function |
|------|----------|
| **TitleBar** | Window title bar, drag to move window |
| **Toolbar** | Navigation buttons + address bar (autocomplete) + search box (history) |
| **RibbonToolbar** | Quick file operation shortcut buttons |
| **Sidebar** | This PC, drives, quick access, favorites |
| **PanesArea** | Core file browsing area: tabs + split panes |
| **PropertiesPanel** | Right-side preview/properties panel, draggable to adjust width |
| **Terminal Panel** | Bottom terminal panel, collapsible/maximizable/draggable height |
| **StatusBar** | Current path + terminal/properties toggle + file count |

---

## 2. File Browsing

### 2.1 5 View Modes

| View | Description |
|------|-------------|
| **Details** | Table: Name/Date Modified/Date Created/Type/Size, draggable column widths |
| **List** | Compact list, ideal for quickly scanning large numbers of files |
| **Grid** | Icon mode, thumbnails and color-coded icons |
| **Tree** | Left tree + right content, deep navigation |
| **Columns** | Miller Columns, multi-column drill-down, macOS Finder style |

### 2.2 Address Bar

- Displays the current full path
- Auto-completes subdirectory names as you type
- Supports cross-platform formats: `C:\`, `C:/Users`, `/home/user`

### 2.3 Breadcrumb Navigation

- Path displayed in segments; click any segment to jump to that directory
- Automatically recognizes Windows drive letters (`C:`) and Unix root (`/`)

### 2.4 Navigation History

- Back / Forward / Up buttons
- `Alt+←` / `Alt+→` keyboard shortcuts
- Up to 50 history entries

---

## 3. File Operations

### 3.1 Basic Operations

| Operation | Method | Shortcut |
|-----------|--------|----------|
| Open | Double-click / Enter | `Enter` / `Space` |
| Go up | Up button | `←` / `Backspace` |
| New folder | Context menu / Ribbon | `Ctrl+Shift+N` |
| New file | Context menu / Ribbon | `Ctrl+N` |
| Rename | Context menu | `F2` |
| Delete | Context menu → Delete | `Delete` |
| Permanently delete | — | `Shift+Delete` |
| Refresh | — | `F5` |

### 3.2 Selection Operations

| Operation | Description |
|-----------|-------------|
| **Click** | Select a single file |
| **Ctrl+Click** | Toggle selection of a single file |
| **Shift+Click** | Select a range from the last selection to the current file |
| **Ctrl+A** | Select all files in the current directory |

### 3.3 Clipboard

| Operation | Shortcut | Description |
|-----------|----------|-------------|
| Cut | `Ctrl+X` | Mark for cut; moves on paste |
| Copy | `Ctrl+C` | Copy to internal clipboard |
| Paste | `Ctrl+V` | Paste into current directory |
| Copy path | Context menu | Newline-separated for multiple files |
| Cancel cut | `Escape` | — |

### 3.4 Compress/Extract

- **Compress**: Select files/folders → Context menu → Compress → ZIP
  - Single selection: default filename is the selected file's name (without extension)
  - Multi-selection: default filename is the parent directory name
- **Extract**: Select one or more archives → Context menu → Extract
  - Supports extracting multiple archives simultaneously
  - Auto-extracts into a directory named after the archive

### 3.5 Drag & Drop

- Drag files into directories to move them
- Drag tabs to reorder
- Windows: drag out to external apps like QQ, WeChat, Chrome

---

## 4. Multi-Pane Workflow

### 4.1 Tabs

| Operation | Shortcut |
|-----------|----------|
| New | Click `+` |
| Close | `Ctrl+W` |
| Switch | `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| Reorder | Drag tab |

### 4.2 Split Panes

| Operation | Description |
|-----------|-------------|
| Split | Context menu → Split → Left/Right/Top/Bottom |
| Resize | Drag the divider |

---

## 5. Preview Features

### 5.1 Right Properties Panel

Preview is shown automatically when a file is selected:

| File Type | Capability |
|-----------|------------|
| Images | Zoom, rotate, pan with drag |
| PDF | Zoom, pan |
| Office | Inline rendering of docx/xlsx/pptx |
| Video | DPlayer, screenshots, playback speed control |
| Code | 50+ language syntax highlighting, line numbers, selection, auto-detect encoding (GBK/Shift-JIS/UTF-16, etc.) |
| Markdown | Edit/preview, Shiki highlighting, export |
| Text | Plain text display, auto-detect encoding |
| Archives | Browse ZIP/7z/RAR/TAR contents |

### 5.2 Standalone Preview Window

- 65%×75% adaptive window
- Left file tree + right preview
- Toolbar: Open / Rename / Save As / Delete / Copy Path

---

## 6. Built-in Terminal

### 6.1 Open/Close

| Method | Description |
|--------|-------------|
| `` Ctrl+` `` | Global shortcut to toggle |
| StatusBar `>_` | Click to toggle |
| Panel `✕` | Close panel |

### 6.2 Panel Elements

```
┌─ ═══ Drag ═══ ──────────────────────────────────┐
│  ▸ zsh ｜ ~/Documents    ● 13px [−][+][⊠][⟳][✕]│
├──────────────────────────────────────────────────┤
│  apple@mac ~/D $ █                              │
└──────────────────────────────────────────────────┘
```

| Element | Description |
|---------|-------------|
| `▸ zsh` | Current shell (macOS zsh / Linux bash / Win PowerShell) |
| `｜path` | Current working directory |
| `●` | Green=running / Yellow=starting / Red=exited |
| `13px` | Font size |
| `[−][+]` | Decrease/increase font size |
| `[⊠]`/`[🗗]` | Maximize/restore |
| `[⟳]` | Restart terminal |
| `[✕]` | Close panel |

### 6.3 Terminal Keyboard

| Key | Behavior |
|-----|----------|
| Regular typing | Passed to shell |
| `Ctrl+C` (nothing selected) | Sends SIGINT to interrupt command |
| `Ctrl+C` (text selected) | Copies selected text |
| `Ctrl+V` | Pastes clipboard |

### 6.4 Panel Operations

| Operation | Method |
|-----------|--------|
| Maximize | `⊠` / double-click header |
| Restore | `🗗` / `Esc` / double-click header / StatusBar indicator |
| Adjust height | Drag the top edge |
| Adjust font | `Ctrl+=`/`Ctrl+-`/`Ctrl+0` or `+`/`−` buttons |

### 6.5 Directory Following

When you switch directories in the file browser, the terminal automatically `cd`s to the new path.

### 6.6 Crash Recovery

When the process exits, an overlay appears. Click `⟳` or "Restart Terminal" to recover.

### 6.7 Theme Colors

Terminal colors follow the application theme automatically. Each of the 7 themes includes a full ANSI 16-color palette:

| Theme | Background |
|-------|-----------|
| Catppuccin Dark | `#1e1e2e` |
| Catppuccin Light | `#eff1f5` |
| Nord | `#2e3440` |
| Tokyo Night | `#1a1b26` |
| One Dark Pro | `#282c34` |
| Dracula | `#282a36` |
| Solarized Light | `#fdf6e3` |

### 6.8 Cross-Platform Shell

| Platform | Default |
|----------|---------|
| macOS | `$SHELL` / `/bin/zsh` |
| Linux | `$SHELL` / `/bin/bash` |
| Windows | `%COMSPEC%` / `powershell.exe` |

---

## 7. Search

### 7.1 Basic Search

Type in the toolbar search box and press `Enter` to start searching.

### 7.2 Wildcards

| Symbol | Description | Example |
|--------|-------------|---------|
| `*` | Any characters | `*.rs` |
| `?` | Single character | `report?.pdf` |
| `\|` | OR | `*.vue \| *.ts` |

### 7.3 Size Filter

| Example | Description |
|---------|-------------|
| `>10MB` | Larger than 10MB |
| `<1KB` | Smaller than 1KB |

### 7.4 Search History

- Last 15 queries persisted
- ↑↓ to select, Enter to confirm
- Supports clearing all history

---

## 8. Favorites

### 8.1 Add/Remove

- Right-click a folder → "Add to / Remove from Favorites"
- Right-click a sidebar bookmark → Remove with confirmation dialog

### 8.2 Navigation

- Click a bookmark to navigate to it
- The bookmark for the current directory is automatically highlighted

---

## 9. System Tray

### 9.1 Tray Menu

```
 Show/Hide Main Window (dynamic toggle)
 ────────────
 Downloads
 Documents
 ────────────
 Clear Cache
 ────────────
 Settings…
 ────────────
 Quit
```

### 9.2 Tray Behavior

| Action | Behavior |
|--------|----------|
| Left-click | Toggle show/hide |
| Left-double-click | Force show + focus |
| Close window | Hide to tray (configurable to quit) |

---

## 10. Keyboard Shortcuts

### 10.1 File List

| Key | Function |
|-----|----------|
| `↑`/`↓` | Move focus |
| `→` | Enter directory |
| `←` | Go up |
| `Home`/`End` | First/last item |
| `PageUp`/`PageDown` | Page up/down |
| `Shift+↑/↓` | Keyboard range selection |
| **`Shift+Click`** | **Mouse range selection** |
| **`Ctrl+Click`** | **Toggle selection** |

### 10.2 Global

| Shortcut | Function |
|----------|----------|
| `Enter`/`Space` | Open |
| `Ctrl+C`/`X`/`V`/`A` | Copy/cut/paste/select all |
| `Delete`/`Shift+Delete` | Delete/permanently delete |
| `F2` | Rename |
| `F5` | Refresh |
| `Ctrl+Z` | Undo |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Switch tab |
| `Ctrl+N`/`Ctrl+Shift+N` | New file/new folder |
| `Ctrl+P` | Properties panel |
| `Ctrl+,` | Settings |
| `Backspace` | Go up |
| `Escape` | Cancel cut / exit terminal maximize / close dialog |

### 10.3 Terminal-Specific

| Shortcut | Function |
|----------|----------|
| `` Ctrl+` `` | Toggle terminal panel |
| `Ctrl+=`/`Ctrl+-`/`Ctrl+0` | Font increase/decrease/reset |
| `Ctrl+C` (nothing selected) | SIGINT |
| `Ctrl+V` | Paste |
| `Esc` | Exit maximize |

---

## 11. Settings

### 11.1 Appearance

| Setting | Options |
|---------|---------|
| Theme | Catppuccin Dark/Light, Nord, Tokyo Night, One Dark Pro, Dracula, Solarized |
| Font Size | Small / Medium / Large |
| Icon Style | Fluent / Material / Material+ |

### 11.2 Language

- 中文 / English

### 11.3 General

| Setting | Description |
|---------|-------------|
| Launch on startup | Auto-run when computer starts |
| Show system tray | Taskbar/menubar icon |
| Quit on close | Quit directly / hide to tray |

### 11.4 About

- Version number
- Clear cache (session/search/video/favorites)
