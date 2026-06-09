import { defineStore } from "pinia";
import { ref, watch } from "vue";

export type ThemeMode = "dark" | "light";
export type FontSize = "small" | "medium" | "large";

export interface AppSettings {
  theme: ThemeMode;
  locale: string;
  fontSize: FontSize;
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
      };
    }
  } catch {}
  return {
    theme: "dark",
    locale: localStorage.getItem("app-locale") || "zh",
    fontSize: "medium",
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

  function persist() {
    saveSettings({
      theme: theme.value,
      locale: locale.value,
      fontSize: fontSize.value,
    });
  }

  // Watch for external locale changes (from toolbar)
  watch(locale, () => persist());

  return {
    theme,
    locale,
    fontSize,
    setTheme,
    setLocale,
    setFontSize,
  };
});
