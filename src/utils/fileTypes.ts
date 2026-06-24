// Shared file type classification — single source of truth
// Used by FileItem.vue, FileList.vue (tree/grid views) + fileIcons.ts

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

// ── Extension → category map (canonical) ──
const CATEGORY_MAP: Record<string, FileCategory> = {
  // Code
  js: "code",
  ts: "code",
  jsx: "code",
  tsx: "code",
  vue: "code",
  svelte: "code",
  py: "code",
  rs: "code",
  go: "code",
  java: "code",
  scala: "code",
  kt: "code",
  c: "code",
  cpp: "code",
  cc: "code",
  cxx: "code",
  cs: "code",
  csx: "code",
  h: "code",
  hpp: "code",
  hxx: "code",
  rb: "code",
  swift: "code",
  dart: "code",
  php: "code",
  sh: "code",
  bash: "code",
  bat: "code",
  ps1: "code",
  sql: "code",
  lua: "code",
  perl: "code",
  pl: "code",
  r: "code",
  erl: "code",
  ex: "code",
  exs: "code",
  hs: "code",
  nim: "code",
  zig: "code",
  fs: "code",
  fsx: "code",
  vb: "code",
  // Image
  png: "image",
  jpg: "image",
  jpeg: "image",
  gif: "image",
  svg: "image",
  webp: "image",
  bmp: "image",
  ico: "image",
  tiff: "image",
  tif: "image",
  heic: "image",
  heif: "image",
  // Audio
  mp3: "audio",
  wav: "audio",
  flac: "audio",
  ogg: "audio",
  aac: "audio",
  m4a: "audio",
  wma: "audio",
  // Video
  mp4: "video",
  avi: "video",
  mkv: "video",
  mov: "video",
  wmv: "video",
  webm: "video",
  flv: "video",
  // Archive
  zip: "archive",
  rar: "archive",
  "7z": "archive",
  tar: "archive",
  gz: "archive",
  xz: "archive",
  bz2: "archive",
  lz: "archive",
  zst: "archive",
  dmg: "archive",
  pkg: "archive",
  iso: "archive",
  // PDF
  pdf: "pdf",
  // App / executable
  exe: "app",
  dll: "app",
  msi: "app",
  app: "app",
  apk: "app",
  deb: "app",
  rpm: "app",
  // Web
  html: "web",
  css: "web",
  scss: "web",
  less: "web",
  sass: "web",
  xml: "web",
  json: "web",
  yaml: "web",
  yml: "web",
  toml: "web",
  // Documents (use "default" category, specific icons via ICON_OVERRIDE)
  doc: "default",
  docx: "default",
  xls: "default",
  xlsx: "default",
  ppt: "default",
  pptx: "default",
  txt: "default",
  log: "default",
  md: "default",
  cfg: "default",
  ini: "default",
  // Windows shortcut
  lnk: "default",
};

// ── Icon key overrides — for extensions whose icon differs from their category name ──
export const ICON_OVERRIDE: Record<string, string> = {
  doc: "word",
  docx: "word",
  xls: "excel",
  xlsx: "excel",
  ppt: "ppt",
  pptx: "ppt",
  txt: "txt",
  log: "txt",
  md: "txt",
  cfg: "txt",
  ini: "txt",
  lnk: "shortcut",
};

// ── macOS bundle directories (is_dir=true but should display as file icons) ──
const BUNDLE_EXTS = new Set([
  "app",
  "bundle",
  "framework",
  "xcodeproj",
  "xcworkspace",
  "prefpane",
  "saver",
  "plugin",
  "pages",
  "numbers",
  "key",
]);

export function isBundleDirectory(extension: string, isDir: boolean): boolean {
  return isDir && BUNDLE_EXTS.has(extension.toLowerCase());
}

export function getFileCategory(
  extension: string,
  isDir: boolean,
): FileCategory {
  if (isDir) {
    return isBundleDirectory(extension, isDir) ? "app" : "folder";
  }
  return CATEGORY_MAP[extension.toLowerCase()] || "default";
}

// ── Category → icon key fallback ──
export function getIconKey(extension: string): string | null {
  const ext = extension.toLowerCase();
  if (ICON_OVERRIDE[ext]) return ICON_OVERRIDE[ext];
  const cat = CATEGORY_MAP[ext];
  // Only return the category as icon key if a corresponding icon exists
  if (cat && cat !== "default" && cat !== "folder") return cat;
  // "default" category extensions with no override get null → generic icon
  if (cat === "default") return null;
  return null;
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
