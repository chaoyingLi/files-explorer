import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, DiskInfo, SpecialDirs, ClipboardInfo } from "@/types";

export async function listDirectory(path: string): Promise<FileEntry[]> {
  return invoke("list_directory", { path });
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

export async function openInTerminal(path: string): Promise<void> {
  return invoke("open_in_terminal", { path });
}

export async function searchFiles(
  directory: string,
  query: string,
): Promise<void> {
  return invoke("search_files", { directory, query });
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
