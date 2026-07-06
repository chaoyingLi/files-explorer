import { defineStore } from "pinia";
import { ref } from "vue";

export type ThemeMode =
  | "dark"
  | "light"
  | "nord"
  | "tokyo-night"
  | "one-dark-pro"
  | "dracula"
  | "solarized-light";
export type FontSize = "small" | "medium" | "large";
export type IconTheme = "fluent" | "material" | "material-full";

export interface Bookmark {
  path: string;
  label: string;
}

export interface AppSettings {
  theme: ThemeMode;
  locale: string;
  fontSize: FontSize;
  iconTheme: IconTheme;
  autoStart: boolean;
  showTray: boolean;
  quitOnClose: boolean;
  bookmarks: Bookmark[];
}

function loadBookmarks(): Bookmark[] {
  try {
    const raw = localStorage.getItem("app-bookmarks");
    if (raw) return JSON.parse(raw);
  } catch {}
  return [];
}

function saveBookmarks(bookmarks: Bookmark[]) {
  // Normalize all paths to forward slashes for storage
  const normalized = bookmarks.map((b) => ({
    ...b,
    path: b.path.replace(/\\/g, "/"),
  }));
  localStorage.setItem("app-bookmarks", JSON.stringify(normalized));
}

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem("app-settings");
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        theme: parsed.theme || "dark",
        locale: parsed.locale || localStorage.getItem("app-locale") || "zh",
        fontSize: parsed.fontSize || "medium",
        iconTheme: parsed.iconTheme || "fluent",
        autoStart: parsed.autoStart ?? true,
        showTray: parsed.showTray ?? true,
        quitOnClose: parsed.quitOnClose ?? false,
        bookmarks: loadBookmarks(),
      };
    }
  } catch {}
  return {
    theme: "dark",
    locale: localStorage.getItem("app-locale") || "zh",
    fontSize: "medium",
    iconTheme: "fluent",
    autoStart: true,
    showTray: true,
    quitOnClose: false,
    bookmarks: loadBookmarks(),
  };
}

function saveSettings(settings: AppSettings) {
  localStorage.setItem("app-settings", JSON.stringify(settings));
}

function applyTheme(theme: ThemeMode) {
  document.documentElement.setAttribute("data-theme", theme);
}

function applyFontSize(size: FontSize) {
  document.documentElement.setAttribute("data-font-size", size);
}

export const useSettingsStore = defineStore("settings", () => {
  const initial = loadSettings();
  const theme = ref<ThemeMode>(initial.theme);
  const locale = ref<string>(initial.locale);
  const fontSize = ref<FontSize>(initial.fontSize);
  const iconTheme = ref<IconTheme>(initial.iconTheme);
  const autoStart = ref<boolean>(initial.autoStart ?? true);
  const showTray = ref<boolean>(initial.showTray ?? true);
  const quitOnClose = ref<boolean>(initial.quitOnClose ?? false);

  // Apply on init
  applyTheme(theme.value);
  applyFontSize(fontSize.value);
  // Sync showTray / quitOnClose to Rust on init
  import("@/utils/tauri").then((m) => {
    m.setQuitOnClose(quitOnClose.value).catch(() => {});
  });
  import("@tauri-apps/api/core").then(({ invoke }) => {
    invoke("set_tray_visible", { visible: showTray.value }).catch(() => {});
  });

  function setTheme(t: ThemeMode) {
    theme.value = t;
    applyTheme(t);
    persist();
  }

  function setLocale(l: string) {
    locale.value = l;
    localStorage.setItem("app-locale", l);
    persist();
  }

  function setFontSize(s: FontSize) {
    fontSize.value = s;
    applyFontSize(s);
    persist();
  }

  function setIconTheme(t: IconTheme) {
    iconTheme.value = t;
    persist();
  }

  async function setAutoStart(v: boolean) {
    autoStart.value = v;
    persist();
    try {
      const { setAutoStart } = await import("@/utils/tauri");
      await setAutoStart(v);
    } catch {
      /* ignore */
    }
  }

  async function setShowTray(v: boolean) {
    showTray.value = v;
    persist();
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_tray_visible", { visible: v });
    } catch {
      /* ignore */
    }
  }

  async function setQuitOnClose(v: boolean) {
    quitOnClose.value = v;
    persist();
    try {
      const { setQuitOnClose } = await import("@/utils/tauri");
      await setQuitOnClose(v);
    } catch {
      /* ignore */
    }
  }

  const bookmarks = ref<Bookmark[]>(loadBookmarks());

  function addBookmark(path: string, label: string) {
    const norm = path.replace(/\\/g, "/");
    if (bookmarks.value.some((b) => b.path.replace(/\\/g, "/") === norm))
      return;
    bookmarks.value.push({ path: norm, label });
    saveBookmarks(bookmarks.value);
  }

  function removeBookmark(path: string) {
    const norm = path.replace(/\\/g, "/");
    bookmarks.value = bookmarks.value.filter(
      (b) => b.path.replace(/\\/g, "/") !== norm,
    );
    saveBookmarks(bookmarks.value);
  }

  function hasBookmark(path: string): boolean {
    const norm = path.replace(/\\/g, "/");
    return bookmarks.value.some((b) => b.path.replace(/\\/g, "/") === norm);
  }

  function renameBookmark(path: string, newLabel: string) {
    const norm = path.replace(/\\/g, "/");
    const bm = bookmarks.value.find((b) => b.path.replace(/\\/g, "/") === norm);
    if (bm) {
      bm.label = newLabel;
      saveBookmarks(bookmarks.value);
    }
  }

  function persist() {
    saveSettings({
      theme: theme.value,
      locale: locale.value,
      fontSize: fontSize.value,
      iconTheme: iconTheme.value as IconTheme,
      autoStart: autoStart.value,
      showTray: showTray.value,
      quitOnClose: quitOnClose.value,
      bookmarks: bookmarks.value,
    });
  }

  return {
    theme,
    locale,
    fontSize,
    iconTheme,
    autoStart,
    showTray,
    quitOnClose,
    bookmarks,
    addBookmark,
    removeBookmark,
    renameBookmark,
    hasBookmark,
    setTheme,
    setLocale,
    setFontSize,
    setIconTheme,
    setAutoStart,
    setShowTray,
    setQuitOnClose,
  };
});
