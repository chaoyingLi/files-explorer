import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, DiskInfo, SpecialDirs, ClipboardInfo } from "@/types";
import { normalizePath } from "./platform";

/**
 * Get adaptive preview window size based on current monitor.
 * Returns { width, height } at 65%×75% of screen, min 640×400, max 90%.
 */
export async function getAdaptivePreviewSize(): Promise<{
  width: number;
  height: number;
}> {
  try {
    const { currentMonitor } = await import("@tauri-apps/api/window");
    const monitor = await currentMonitor();
    if (monitor) {
      const mw = monitor.size.width;
      const mh = monitor.size.height;
      return {
        width: Math.min(
          Math.max(640, Math.round(mw * 0.65)),
          Math.round(mw * 0.9),
        ),
        height: Math.min(
          Math.max(400, Math.round(mh * 0.75)),
          Math.round(mh * 0.9),
        ),
      };
    }
  } catch {
    /* ignore */
  }
  return { width: 960, height: 680 };
}

/**
 * Join path segments using forward slash (internal standardized format).
 */
export function joinPath(base: string, name: string): string {
  const norm = normalizePath(base);
  if (norm.endsWith("/")) return norm + name;
  return norm + "/" + name;
}

export async function listDirectory(path: string): Promise<FileEntry[]> {
  return invoke("list_directory", { path: normalizePath(path) });
}

export async function listDirectoryStreamed(path: string): Promise<void> {
  return invoke("list_directory_streamed", { path: normalizePath(path) });
}

export async function getDrives(): Promise<DiskInfo[]> {
  return invoke("get_drives");
}

export async function getParentDirectory(path: string): Promise<string> {
  return invoke("get_parent_directory", { path: normalizePath(path) });
}

export async function createDirectory(path: string): Promise<void> {
  return invoke("create_directory", { path: normalizePath(path) });
}

export async function createFile(path: string): Promise<void> {
  return invoke("create_file", { path: normalizePath(path) });
}

export async function deleteItem(
  path: string,
  permanently: boolean = false,
): Promise<void> {
  return invoke("delete_item", { path: normalizePath(path), permanently });
}

export async function deleteItems(
  paths: string[],
  permanently: boolean = false,
): Promise<{ success: string[]; failed: { path: string; error: string }[] }> {
  const results: {
    success: string[];
    failed: { path: string; error: string }[];
  } = {
    success: [],
    failed: [],
  };
  for (const p of paths) {
    try {
      await deleteItem(normalizePath(p), permanently);
      results.success.push(p);
    } catch (e: any) {
      results.failed.push({ path: p, error: String(e) });
    }
  }
  return results;
}

export async function renameItem(
  oldPath: string,
  newPath: string,
): Promise<void> {
  return invoke("rename_item", {
    oldPath: normalizePath(oldPath),
    newPath: normalizePath(newPath),
  });
}

export async function copyClipboard(paths: string[]): Promise<void> {
  return invoke("copy_clipboard", { paths: paths.map(normalizePath) });
}

export async function cutClipboard(paths: string[]): Promise<void> {
  return invoke("cut_clipboard", { paths: paths.map(normalizePath) });
}

export async function pasteClipboard(destDir: string): Promise<void> {
  return invoke("paste_clipboard", { destDir: normalizePath(destDir) });
}

export async function getClipboardInfo(): Promise<ClipboardInfo> {
  return invoke("get_clipboard_info");
}

export async function getFileInfo(path: string): Promise<FileEntry> {
  return invoke("get_file_info", { path: normalizePath(path) });
}

export async function openFile(path: string): Promise<void> {
  return invoke("open_file", { path: normalizePath(path) });
}

export async function showFileProperties(path: string): Promise<void> {
  return invoke("show_file_properties", { path: normalizePath(path) });
}

export async function showInExplorer(path: string): Promise<void> {
  return invoke("show_in_explorer", { path: normalizePath(path) });
}

export async function startNativeDrag(paths: string[]): Promise<string> {
  return invoke("start_native_drag_cmd", { paths: paths.map(normalizePath) });
}

export async function openInTerminal(path: string): Promise<void> {
  return invoke("open_in_terminal", { path: normalizePath(path) });
}

export async function searchFiles(
  directory: string,
  query: string,
): Promise<void> {
  return invoke("search_files", {
    directory: normalizePath(directory),
    query,
  });
}

export async function cancelSearch(): Promise<void> {
  return invoke("cancel_search");
}

export async function getSpecialDirs(): Promise<SpecialDirs> {
  return invoke("get_special_dirs");
}

export async function setAutoStart(enabled: boolean): Promise<void> {
  return invoke("set_auto_start", { enabled });
}

export async function isAutoStartEnabled(): Promise<boolean> {
  return invoke("is_auto_start_enabled");
}

export async function setQuitOnClose(enabled: boolean): Promise<void> {
  return invoke("set_quit_on_close", { enabled });
}

export async function pathExists(path: string): Promise<boolean> {
  return invoke("path_exists", { path: normalizePath(path) });
}

export async function moveFiles(
  paths: string[],
  destDir: string,
  copy: boolean,
): Promise<void> {
  return invoke("move_files", {
    paths: paths.map(normalizePath),
    destDir: normalizePath(destDir),
    copy,
  });
}

export async function undoLastAction(): Promise<string> {
  return invoke("undo_last_action");
}

export async function getUndoInfo(): Promise<{
  kind: any;
  timestamp: number;
} | null> {
  return invoke("get_undo_info");
}

export async function getFileBase64(
  path: string,
): Promise<{ mime: string; data: string }> {
  return invoke("get_file_base64", { path: normalizePath(path) });
}

export async function compressFiles(
  paths: string[],
  dest: string,
): Promise<void> {
  return invoke("compress_files", {
    paths: paths.map(normalizePath),
    dest: normalizePath(dest),
  });
}

export async function extractArchive(
  archive: string,
  destDir: string,
): Promise<void> {
  return invoke("extract_archive_cmd", {
    archive: normalizePath(archive),
    destDir: normalizePath(destDir),
  });
}

export async function getFilePreview(
  path: string,
): Promise<{ type: string; content?: string; data?: string; ext?: string }> {
  return invoke("get_file_preview", { path: normalizePath(path) });
}

export async function getFileIcon(path: string): Promise<string> {
  return invoke("get_file_icon", { path: normalizePath(path) });
}

export async function readFileBytes(path: string): Promise<string> {
  return invoke("read_file_bytes", { path: normalizePath(path) });
}

export interface ArchiveEntry {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
}

export async function listArchiveContents(
  path: string,
): Promise<ArchiveEntry[]> {
  return invoke("list_archive_contents", { path: normalizePath(path) });
}

export async function extractArchiveEntry(
  archivePath: string,
  entryPath: string,
): Promise<{ temp_path: string; original_name: string }> {
  return invoke("extract_archive_entry", {
    archivePath: normalizePath(archivePath),
    entryPath: normalizePath(entryPath),
  });
}

export async function printFile(path: string): Promise<void> {
  return invoke("print_file", { path: normalizePath(path) });
}

export async function copyFileAs(src: string, dest: string): Promise<void> {
  return invoke("copy_file_as", {
    src: normalizePath(src),
    dest: normalizePath(dest),
  });
}

export async function saveTextFile(
  path: string,
  content: string,
): Promise<void> {
  return invoke("save_text_file", {
    path: normalizePath(path),
    content,
  });
}
