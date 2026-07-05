// src/utils/platform.ts
// Cross-platform frontend utilities.
// Primary: Tauri @tauri-apps/api/os::platform()
// Fallback: navigator.platform / userAgent
// --
// Exports: isMac / isWindows / isLinux / META_KEY / MOD_KEY
//          shortcut helpers, theme listeners, DPI, dialogs, path join,
//          backend directory lookup, general OS detection utilities.

// ═══════════════════════════════════════════════════════════════
// 1. Platform detection — navigator (reliable in Tauri WebView)
// ═══════════════════════════════════════════════════════════════

type PlatformKind = "windows" | "macos" | "linux" | "unknown";

let _platform: PlatformKind = "unknown";
let _platformReady = false;

/** One-shot init: resolve platform from navigator, then cache. */
export function initPlatform(): PlatformKind {
  if (_platformReady) return _platform;
  _platform = _detectFromNavigator();
  _platformReady = true;
  return _platform;
}

function _detectFromNavigator(): PlatformKind {
  if (typeof navigator === "undefined") return "unknown";
  const p = (navigator.platform || "").toUpperCase();
  const u = navigator.userAgent || "";
  if (p.includes("MAC") || u.includes("Macintosh")) return "macos";
  if (p.includes("WIN") || u.includes("Windows")) return "windows";
  if (p.includes("LINUX")) return "linux";
  return "unknown";
}

/** Synchronous platform string — call `initPlatform()` first for accuracy. */
export function getPlatform(): PlatformKind {
  return _platform === "unknown" ? _detectFromNavigator() : _platform;
}

/** Human-readable platform name. */
export function platformName(): string {
  switch (getPlatform()) {
    case "windows":
      return "Windows";
    case "macos":
      return "macOS";
    case "linux":
      return "Linux";
    default:
      return "Unknown";
  }
}

export function isTauri(): boolean {
  try {
    return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  } catch {
    return false;
  }
}

// ═══════════════════════════════════════════════════════════════
// 2. OS boolean constants (synchronous — safe after initPlatform)
// ═══════════════════════════════════════════════════════════════

/** macOS only. */
export const isMac: boolean = _detectFromNavigator() === "macos";

/** Windows only. */
export const isWindows: boolean = _detectFromNavigator() === "windows";

/** Linux only (excludes macOS). */
export const isLinux: boolean = !isMac && !isWindows;

// ═══════════════════════════════════════════════════════════════
// 3. Keyboard modifier constants
// ═══════════════════════════════════════════════════════════════

/** Human-readable modifier key label. */
export const META_KEY: string = isMac ? "⌘" : "Ctrl";

/**
 * `KeyboardEvent` property name for the primary modifier.
 * Use `e[MOD_KEY]` to read the modifier state in event handlers.
 */
export const MOD_KEY: "metaKey" | "ctrlKey" = isMac ? "metaKey" : "ctrlKey";

export const ALT_KEY: "altKey" = "altKey";
export const SHIFT_KEY: "shiftKey" = "shiftKey";

// ═══════════════════════════════════════════════════════════════
// 4. Shortcut helpers
// ═══════════════════════════════════════════════════════════════

/**
 * Convert a shortcut descriptor like "mod+s" to a human-readable label.
 *   "mod+s"   →  macOS "⌘S"    Windows "Ctrl+S"
 *   "mod+shift+n" → macOS "⌘⇧N"  Windows "Ctrl+Shift+N"
 */
export function shortcutLabel(shortcut: string): string {
  return shortcut
    .replace(/mod/i, isMac ? "⌘" : "Ctrl")
    .replace(/shift/i, isMac ? "⇧" : "Shift")
    .replace(/alt/i, isMac ? "⌥" : "Alt")
    .split("+")
    .join(isMac ? "" : "+");
}

/** Check whether a `KeyboardEvent` matches a shortcut descriptor. */
export function matchesShortcut(e: KeyboardEvent, shortcut: string): boolean {
  const parts = shortcut.toLowerCase().split("+");
  const mod = parts.includes("mod") ? e[MOD_KEY] : true;
  const shift = parts.includes("shift")
    ? e.shiftKey
    : !e.shiftKey || parts.includes("shift");
  const alt = parts.includes("alt")
    ? e.altKey
    : !e.altKey || parts.includes("alt");
  const key = parts.find((p) => !["mod", "shift", "alt"].includes(p));
  return mod && shift && alt && e.key.toLowerCase() === key;
}

// ═══════════════════════════════════════════════════════════════
// 5. DPI / scale factor
// ═══════════════════════════════════════════════════════════════

/** Raw device pixel ratio. */
export function devicePixelRatio(): number {
  return (typeof window !== "undefined" && window.devicePixelRatio) || 1;
}

/** Scale a CSS-pixel size to physical pixels (e.g. for icon rendering). */
export function scaleSize(cssPx: number): number {
  return Math.round(cssPx * devicePixelRatio());
}

// ═══════════════════════════════════════════════════════════════
// 6. System theme — CSS + Tauri native
// ═══════════════════════════════════════════════════════════════

export function isSystemDarkMode(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** CSS-only listener. Prefer `onNativeThemeChange`. */
export function onSystemThemeChange(
  callback: (dark: boolean) => void,
): () => void {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = (e: MediaQueryListEvent) => callback(e.matches);
  mq.addEventListener("change", handler);
  return () => mq.removeEventListener("change", handler);
}

/**
 * Tauri-native + CSS theme listener (syncs with OS instantly).
 * Fires immediately with the current value, then on every change.
 * Returns an unsubscribe function.
 */
export async function onNativeThemeChange(
  callback: (dark: boolean) => void,
): Promise<() => void> {
  // 1. Immediate check
  callback(isSystemDarkMode());

  // 2. Tauri window theme-changed event
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    const unlisten = await win.onThemeChanged(({ payload }) => {
      callback(payload === "dark");
    });
    return () => {
      unlisten();
    };
  } catch {
    // Fallback to CSS media-query
    return onSystemThemeChange(callback);
  }
}

// ═══════════════════════════════════════════════════════════════
// 7. File dialogs (cross-platform, via @tauri-apps/plugin-dialog)
// ═══════════════════════════════════════════════════════════════

import type { DialogFilter } from "@tauri-apps/plugin-dialog";

export async function openFileDialog(options?: {
  title?: string;
  filters?: { name: string; extensions: string[] }[];
  multiple?: boolean;
  directory?: boolean;
}): Promise<string | string[] | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({
    title: options?.title || "Select File",
    filters: options?.filters as DialogFilter[] | undefined,
    multiple: options?.multiple ?? false,
    directory: options?.directory ?? false,
  });
}

export async function saveFileDialog(options?: {
  title?: string;
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | null> {
  const { save } = await import("@tauri-apps/plugin-dialog");
  return save({
    title: options?.title || "Save File",
    defaultPath: options?.defaultPath,
    filters: options?.filters as DialogFilter[] | undefined,
  });
}

// ═══════════════════════════════════════════════════════════════
// 8. Path join (cross-platform separator)
// ═══════════════════════════════════════════════════════════════

export function joinPath(base: string, ...segments: string[]): string {
  const sep = isWindows ? "\\" : "/";
  let result = base.replace(/[/\\]$/, "");
  for (const seg of segments) {
    result += sep + seg.replace(/^[/\\]/, "");
  }
  return result;
}

// ═══════════════════════════════════════════════════════════════
// 9. Backend directory lookup (invoke → get_special_dirs)
// ═══════════════════════════════════════════════════════════════

export interface AppDirs {
  home: string;
  desktop: string;
  documents: string;
  downloads: string;
  pictures: string;
  music: string;
  videos: string;
}

let _cachedDirs: AppDirs | null = null;

export async function getAppDirs(): Promise<AppDirs> {
  if (_cachedDirs) return _cachedDirs;
  const { invoke } = await import("@tauri-apps/api/core");
  _cachedDirs = await invoke("get_special_dirs");
  return _cachedDirs!;
}

/** Invalidate the cached app dirs (e.g. after the user changes home folder). */
export function clearAppDirsCache(): void {
  _cachedDirs = null;
}

// ═══════════════════════════════════════════════════════════════
// 10. General OS utility functions
// ═══════════════════════════════════════════════════════════════

/** Check if the current platform is in a set. */
export function platformIs(...kinds: PlatformKind[]): boolean {
  return kinds.includes(getPlatform());
}

/** Run different logic per platform. */
export function platformSwitch<T>(
  handlers: Partial<Record<PlatformKind, () => T>> & { default: () => T },
): T {
  const h = handlers[getPlatform()];
  return h ? h() : handlers.default();
}

/**
 * Get a platform-specific value.
 *   const sep = platformValue({ windows: "\\", macos: "/", linux: "/", default: "/" });
 */
export function platformValue<T>(
  map: Partial<Record<PlatformKind, T>> & { default: T },
): T {
  return map[getPlatform()] ?? map.default;
}
