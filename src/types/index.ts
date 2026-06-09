export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number;
  extension: string;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  used_space: number;
  file_system: string;
  label: string;
}

export interface SpecialDirs {
  home: string;
  desktop: string;
  documents: string;
  downloads: string;
  pictures: string;
  music: string;
  videos: string;
}

export interface ContextMenuOption {
  label: string;
  action: string;
  shortcut?: string;
  icon?: string;
  separator?: boolean;
  disabled?: boolean;
  children?: ContextMenuOption[];
}

export interface ClipboardInfo {
  paths: string[];
  action: string; // "copy" | "cut"
}

export interface FileAction {
  kind: ActionKind;
  timestamp: number;
}

export interface ActionKind {
  type: "Delete" | "Rename" | "Create" | "Copy";
  // For Rename:
  old_path?: string;
  new_path?: string;
  // For Create:
  path?: string;
  is_dir?: boolean;
  // For Copy:
  src?: string;
  dest?: string;
}
