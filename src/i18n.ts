import { createI18n } from "vue-i18n";
import en from "./locales/en";
import zh from "./locales/zh";

function getInitialLocale(): string {
  try {
    const settings = localStorage.getItem("app-settings");
    if (settings) return JSON.parse(settings).locale || "zh";
  } catch {}
  return localStorage.getItem("app-locale") || "zh";
}

const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: "en",
  messages: {
    en,
    zh,
  },
});

export default i18n;
