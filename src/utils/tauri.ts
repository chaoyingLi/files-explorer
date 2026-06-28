import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, DiskInfo, SpecialDirs, ClipboardInfo } from "@/types";

/**
 * Join path segments using the separator style detected from the base path.
 * On Windows, if base uses backslashes, result uses backslashes;
 * otherwise forward slashes.
 */
export function joinPath(base: string, name: string): string {
  const sep = base.includes("\\") ? "\\" : "/";
  if (base.endsWith("/") || base.endsWith("\\")) return base + name;
  return base + sep + name;
}

export async function listDirectory(path: string): Promise<FileEntry[]> {
  return invoke("list_directory", { path });
}

export async function listDirectoryStreamed(path: string): Promise<void> {
  return invoke("list_directory_streamed", { path });
}

export async function getDrives(): Promise<DiskInfo[]> {
  return invoke("get_drives");
}

export async function getParentDirectory(path: string): Promise<string> {
  return invoke("get_parent_directory", { path });
}

export async function createDirectory(path: string): Promise<void> {
  return invoke("create_directory", { path });
}

export async function createFile(path: string): Promise<void> {
  return invoke("create_file", { path });
}

export async function deleteItem(
  path: string,
  permanently: boolean = false,
): Promise<void> {
  return invoke("delete_item", { path, permanently });
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
      await deleteItem(p, permanently);
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
  return invoke("rename_item", { oldPath, newPath });
}

export async function copyClipboard(paths: string[]): Promise<void> {
  return invoke("copy_clipboard", { paths });
}

export async function cutClipboard(paths: string[]): Promise<void> {
  return invoke("cut_clipboard", { paths });
}

export async function pasteClipboard(destDir: string): Promise<void> {
  return invoke("paste_clipboard", { destDir });
}

export async function getClipboardInfo(): Promise<ClipboardInfo> {
  return invoke("get_clipboard_info");
}

export async function getFileInfo(path: string): Promise<FileEntry> {
  return invoke("get_file_info", { path });
}

export async function openFile(path: string): Promise<void> {
  return invoke("open_file", { path });
}

export async function showFileProperties(path: string): Promise<void> {
  return invoke("show_file_properties", { path });
}

export async function showInExplorer(path: string): Promise<void> {
  return invoke("show_in_explorer", { path });
}

export async function startNativeDrag(paths: string[]): Promise<string> {
  return invoke("start_native_drag_cmd", { paths });
}

export async function openInTerminal(path: string): Promise<void> {
  return invoke("open_in_terminal", { path });
}

export async function searchFiles(
  directory: string,
  query: string,
  content: string = "",
): Promise<void> {
  return invoke("search_files", { directory, query, content });
}

export async function cancelSearch(): Promise<void> {
  return invoke("cancel_search");
}

export async function getSpecialDirs(): Promise<SpecialDirs> {
  return invoke("get_special_dirs");
}

export async function pathExists(path: string): Promise<boolean> {
  return invoke("path_exists", { path });
}

export async function moveFiles(
  paths: string[],
  destDir: string,
  copy: boolean,
): Promise<void> {
  return invoke("move_files", { paths, destDir, copy });
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
  return invoke("get_file_base64", { path });
}

export async function compressFiles(
  paths: string[],
  dest: string,
): Promise<void> {
  return invoke("compress_files", { paths, dest });
}

export async function extractArchive(
  archive: string,
  destDir: string,
): Promise<void> {
  return invoke("extract_archive_cmd", { archive, destDir });
}

export async function getFilePreview(
  path: string,
): Promise<{ type: string; content?: string; data?: string; ext?: string }> {
  return invoke("get_file_preview", { path });
}

export async function getFileIcon(path: string): Promise<string> {
  return invoke("get_file_icon", { path });
}

export async function readFileBytes(path: string): Promise<string> {
  return invoke("read_file_bytes", { path });
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
  return invoke("list_archive_contents", { path });
}

export async function extractArchiveEntry(
  archivePath: string,
  entryPath: string,
): Promise<{ temp_path: string; original_name: string }> {
  return invoke("extract_archive_entry", {
    archivePath,
    entryPath,
  });
}

export async function printFile(path: string): Promise<void> {
  return invoke("print_file", { path });
}

export async function copyFileAs(src: string, dest: string): Promise<void> {
  return invoke("copy_file_as", { src, dest });
}
