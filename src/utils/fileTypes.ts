// Shared file type classification — single source of truth
// Used by FileItem.vue, FileList.vue (tree/grid views)

export type FileCategory =
  | "folder"
  | "code"
  | "image"
  | "audio"
  | "video"
  | "archive"
  | "pdf"
  | "app"
  | "web"
  | "default";

const CATEGORY_MAP: Record<FileCategory, string[]> = {
  folder: [],
  code: [
    "js",
    "ts",
    "jsx",
    "tsx",
    "vue",
    "py",
    "rs",
    "go",
    "java",
    "c",
    "cpp",
    "h",
    "rb",
    "swift",
    "kt",
    "scala",
    "php",
    "lua",
    "r",
    "sh",
    "bash",
  ],
  image: [
    "png",
    "jpg",
    "jpeg",
    "gif",
    "svg",
    "webp",
    "bmp",
    "ico",
    "tiff",
    "tif",
  ],
  audio: ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"],
  video: ["mp4", "avi", "mkv", "mov", "wmv", "webm", "flv"],
  archive: ["zip", "rar", "7z", "tar", "gz", "xz", "bz2", "lz", "zst"],
  pdf: ["pdf"],
  app: ["exe", "dll", "msi", "app", "apk", "deb", "rpm"],
  web: [
    "html",
    "css",
    "scss",
    "less",
    "sass",
    "xml",
    "json",
    "yaml",
    "yml",
    "toml",
  ],
  default: [],
};

export function getFileCategory(
  extension: string,
  isDir: boolean,
): FileCategory {
  if (isDir) return "folder";
  const ext = extension.toLowerCase();
  for (const [category, exts] of Object.entries(CATEGORY_MAP)) {
    if (category === "folder" || category === "default") continue;
    if (exts.includes(ext)) return category as FileCategory;
  }
  return "default";
}

export function colorClassForCategory(category: FileCategory): string {
  if (category === "folder") return "icon-folder";
  return `color-${category}`;
}

export function treeColorClassForCategory(category: FileCategory): string {
  if (category === "folder") return "";
  return `tree-color-${category}`;
}

export function gridColorClassForCategory(category: FileCategory): string {
  if (category === "folder") return "grid-folder-color";
  return `grid-color-${category}`;
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let size = bytes;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatFileDate(timestamp: number): string {
  if (timestamp === 0) return "";
  const date = new Date(timestamp * 1000);
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const h = String(date.getHours()).padStart(2, "0");
  const min = String(date.getMinutes()).padStart(2, "0");
  return `${y}/${m}/${d} ${h}:${min}`;
}
