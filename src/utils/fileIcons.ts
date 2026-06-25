// Fluent UI System Icons (Microsoft) — file-type-specific SVGs
// Icon key derivation now handled by fileTypes.ts (single source of truth)

const icons = {
  word: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#185ABD"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#4A8FE0"/><path d="M10.5 14h11v1.5h-11V14zm0 4h11v1.5h-11V18zm0 4h7v1.5h-7V22z" fill="#fff"/></svg>`,
  excel: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#1E7145"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#3DA56A"/><path d="M11 14h4v5.5h-4V14zm6 0h4v5.5h-4V14zm-6 7h4V26h-4v-5zm6 0h4V26h-4v-5z" fill="#fff"/></svg>`,
  ppt: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#C43E1C"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#E66A3E"/><path d="M11 14h10v1.5H11V14zm0 3.5h10v1.5H11v-1.5zm0 3.5h6V26h-6v-5z" fill="#fff"/></svg>`,
  pdf: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#D13438"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#E85153"/><text x="10" y="23" fill="#fff" font-size="7" font-weight="700">PDF</text></svg>`,
  txt: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#4A6A9A"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#6B8FC0"/><path d="M10.5 14h11v1.5h-11V14zm0 4h11v1.5h-11V18zm0 4h7v1.5h-7V22z" fill="#fff"/></svg>`,
  image: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#6A3E9A"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#A072D0"/><circle cx="13" cy="16" r="2" fill="#fff"/><path d="M10 25l4-5 3 3.5 2-2.5 3 4H10z" fill="#fff" opacity="0.8"/></svg>`,
  video: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#6A1E7A"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#9A3EB0"/><path d="M13 14l5 3.5-5 3.5V14z" fill="#fff"/></svg>`,
  audio: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#8A6A1E"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#C0A040"/><path d="M19 14a3 3 0 00-6 0v4a3 3 0 006 0v-4zm-3 9a5 5 0 004.5-2.8V23H22v2a5 5 0 01-10 0v-2h1.5v2.2A5 5 0 0016 23z" fill="#fff"/></svg>`,
  archive: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#6E5A3A"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#A08A5A"/><rect x="13" y="14" width="6" height="2" rx="1" fill="#fff"/><rect x="11" y="18" width="10" height="1.5" rx="0.75" fill="#fff" opacity="0.7"/><rect x="11" y="21" width="8" height="1.5" rx="0.75" fill="#fff" opacity="0.5"/></svg>`,
  code: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#2D7D46"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#4DA868"/><path d="M13 15l-2.5 2.5L13 20m6-5l2.5 2.5L19 20" stroke="#fff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  exe: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#4A5A6A"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#7A8A9A"/><path d="M12 14l3 3-3 3m5-6v6" stroke="#fff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  shortcut: `<svg viewBox="0 0 32 32" fill="none"><path d="M6 2.5A1.5 1.5 0 004.5 4v24A1.5 1.5 0 006 29.5h20a1.5 1.5 0 001.5-1.5V10.8a1.5 1.5 0 00-.44-1.06l-6.3-6.3A1.5 1.5 0 0019.7 3H6z" fill="#6C7086"/><path d="M19.5 3v5.5a1.5 1.5 0 001.5 1.5h5.5" fill="#9CA0B0"/><path d="M19 14l-4 4V21h3l4-4v-3h-3z" fill="#89B4FA"/></svg>`,
} as const;

// Re-export bundle helper from single source
export { isBundleDirectory } from "./fileTypes";

import { getIconKey, getFileCategory } from "./fileTypes";
import type { FileCategory } from "./fileTypes";

const iconCache = new Map<string, string | null>();

export function getFileIconSvg(
  extOrFile: string | { extension: string; is_dir: boolean },
  isDir?: boolean,
): string | null {
  let extension: string;
  let isDirFlag: boolean;
  if (typeof extOrFile === "string") {
    extension = extOrFile;
    isDirFlag = isDir ?? false;
  } else {
    extension = extOrFile.extension;
    isDirFlag = extOrFile.is_dir;
  }

  // Cache key: "ext|isDir"
  const cacheKey = `${extension.toLowerCase()}|${isDirFlag}`;
  const cached = iconCache.get(cacheKey);
  if (cached !== undefined) return cached;

  let result: string | null = null;

  // Directory icon
  if (isDirFlag) {
    const cat = getFileCategory(extension, true);
    if (cat === "app") {
      // Bundle directory (e.g. .app)
      result = icons["exe"] || icons["shortcut"] || null;
    } else {
      // Regular folder — return null so caller uses default folder icon
      result = null;
    }
  } else {
    const key = getIconKey(extension);
    result = key ? icons[key as keyof typeof icons] : null;
  }

  iconCache.set(cacheKey, result);
  return result;
}

/** Return file type description string for display */
export function getFileTypeDescription(
  extOrFile: string | { extension: string; is_dir: boolean },
): string {
  let extension: string;
  let isDir: boolean;
  if (typeof extOrFile === "string") {
    extension = extOrFile;
    isDir = false;
  } else {
    extension = extOrFile.extension;
    isDir = extOrFile.is_dir;
  }
  const cat = getFileCategory(extension, isDir);
  const labels: Record<FileCategory, string> = {
    folder: "Folder",
    code: "Code",
    image: "Image",
    audio: "Audio",
    video: "Video",
    archive: "Archive",
    pdf: "PDF",
    app: "Application",
    web: "Web",
    default: extension ? extension.toUpperCase() + " File" : "File",
  };
  return labels[cat] || labels.default;
}
