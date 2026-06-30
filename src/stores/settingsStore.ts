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
  localStorage.setItem("app-bookmarks", JSON.stringify(bookmarks));
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
        bookmarks: loadBookmarks(),
      };
    }
  } catch {}
  return {
    theme: "dark",
    locale: localStorage.getItem("app-locale") || "zh",
    fontSize: "medium",
    iconTheme: "fluent",
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

  // Apply on init
  applyTheme(theme.value);
  applyFontSize(fontSize.value);

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

  const bookmarks = ref<Bookmark[]>(loadBookmarks());

  function addBookmark(path: string, label: string) {
    if (bookmarks.value.some((b) => b.path === path)) return;
    bookmarks.value.push({ path, label });
    saveBookmarks(bookmarks.value);
  }

  function removeBookmark(path: string) {
    bookmarks.value = bookmarks.value.filter((b) => b.path !== path);
    saveBookmarks(bookmarks.value);
  }

  function hasBookmark(path: string): boolean {
    return bookmarks.value.some((b) => b.path === path);
  }

  function persist() {
    saveSettings({
      theme: theme.value,
      locale: locale.value,
      fontSize: fontSize.value,
      iconTheme: iconTheme.value as IconTheme,
      bookmarks: bookmarks.value,
    });
  }

  return {
    theme,
    locale,
    fontSize,
    iconTheme,
    bookmarks,
    addBookmark,
    removeBookmark,
    hasBookmark,
    setTheme,
    setLocale,
    setFontSize,
    setIconTheme,
  };
});
