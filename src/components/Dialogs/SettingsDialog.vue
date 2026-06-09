<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="settings-dialog">
      <div class="settings-header">
        <h2>{{ t('settings.title') }}</h2>
        <button class="icon-btn close-btn" @click="$emit('close')">
          <svg viewBox="0 0 20 20" fill="none"><path d="M5 5l10 10M15 5L5 15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        </button>
      </div>

      <div class="settings-body">
        <!-- Theme -->
        <div class="setting-section">
          <div class="section-title">{{ t('settings.theme') }}</div>
          <div class="theme-options">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              class="theme-card"
              :class="{ active: settings.theme === opt.value }"
              @click="settings.setTheme(opt.value)"
            >
              <div class="theme-preview" :class="opt.value">
                <div class="preview-sidebar"></div>
                <div class="preview-main">
                  <div class="preview-toolbar"></div>
                  <div class="preview-content">
                    <div class="preview-folder"></div>
                    <div class="preview-file"></div>
                    <div class="preview-file"></div>
                  </div>
                </div>
              </div>
              <span class="theme-label">{{ t(opt.labelKey) }}</span>
            </button>
          </div>
        </div>

        <!-- Language -->
        <div class="setting-section">
          <div class="section-title">{{ t('settings.language') }}</div>
          <div class="lang-options">
            <button
              v-for="opt in langOptions"
              :key="opt.value"
              class="lang-card"
              :class="{ active: settings.locale === opt.value }"
              @click="handleLocaleChange(opt.value)"
            >
              <span class="lang-flag">{{ opt.flag }}</span>
              <div class="lang-info">
                <span class="lang-name">{{ opt.name }}</span>
                <span class="lang-native">{{ opt.native }}</span>
              </div>
              <svg v-if="settings.locale === opt.value" class="check-icon" viewBox="0 0 20 20" fill="none">
                <path d="M4 10l4 4 8-8" stroke="var(--accent)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Font Size -->
        <div class="setting-section">
          <div class="section-title">{{ t('settings.fontSize') }}</div>
          <div class="font-options">
            <button
              v-for="opt in fontSizeOptions"
              :key="opt.value"
              class="font-card"
              :class="{ active: settings.fontSize === opt.value }"
              @click="settings.setFontSize(opt.value)"
            >
              <span class="font-sample" :style="{ fontSize: opt.sampleSize }">Aa</span>
              <span class="font-label">{{ t(opt.labelKey) }}</span>
            </button>
          </div>
        </div>

        <!-- About -->
        <div class="setting-section">
          <div class="section-title">{{ t('settings.about') }}</div>
          <div class="about-content">
            <div class="about-icon">
              <svg viewBox="0 0 48 48">
                <path d="M4 12a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v18a3 3 0 01-3 3H7a3 3 0 01-3-3V12z" fill="#DEB949"/>
                <path d="M4 15a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v16a3 3 0 01-3 3H7a3 3 0 01-3-3V12z" fill="#F5C542"/>
              </svg>
            </div>
            <div class="about-info">
              <div class="about-name">Files Explorer</div>
              <div class="about-version">{{ t('settings.version') }} 0.1.0</div>
              <div class="about-desc">{{ t('settings.description') }}</div>
              <div class="about-tech">Tauri 2.0 + Vue 3 + Vite</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settingsStore";
import type { ThemeMode, FontSize } from "@/stores/settingsStore";

const { t, locale } = useI18n();
const settings = useSettingsStore();

defineEmits<{ close: [] }>();

const themeOptions: { value: ThemeMode; labelKey: string }[] = [
  { value: "dark", labelKey: "settings.dark" },
  { value: "light", labelKey: "settings.light" },
];

const langOptions = [
  { value: "zh", name: "Chinese", native: "\u4E2D\u6587", flag: "\uD83C\uDDE8\uD83C\uDDF3" },
  { value: "en", name: "English", native: "English", flag: "\uD83C\uDDFA\uD83C\uDDF8" },
];

const fontSizeOptions: { value: FontSize; labelKey: string; sampleSize: string }[] = [
  { value: "small", labelKey: "settings.small", sampleSize: "14px" },
  { value: "medium", labelKey: "settings.medium", sampleSize: "18px" },
  { value: "large", labelKey: "settings.large", sampleSize: "24px" },
];

function handleLocaleChange(l: string) {
  settings.setLocale(l);
  locale.value = l;
}
</script>

<style scoped>
.settings-dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 14px;
  width: 540px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px var(--shadow);
  animation: dialog-in 0.2s ease-out;
}

@keyframes dialog-in {
  from { opacity: 0; transform: scale(0.95) translateY(-10px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.settings-header h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.setting-section {
  margin-bottom: 24px;
}

.setting-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted);
  letter-spacing: 0.5px;
  margin-bottom: 12px;
}

/* Theme cards */
.theme-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.theme-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border: 2px solid var(--border);
  border-radius: 10px;
  background: var(--bg-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.theme-card:hover {
  border-color: var(--text-muted);
}

.theme-card.active {
  border-color: var(--accent);
  background: var(--bg-hover);
}

.theme-preview {
  width: 100%;
  height: 70px;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  border: 1px solid var(--border);
}

.theme-preview.dark {
  background: #1e1e2e;
}

.theme-preview.light {
  background: #eff1f5;
}

.preview-sidebar {
  width: 25%;
  background: var(--bg-tertiary);
}

.theme-preview.dark .preview-sidebar { background: #11111b; }
.theme-preview.light .preview-sidebar { background: #ccd0da; }

.preview-main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.preview-toolbar {
  height: 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.theme-preview.dark .preview-toolbar { background: #181825; }
.theme-preview.light .preview-toolbar { background: #e6e9ef; }

.preview-content {
  flex: 1;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preview-folder {
  height: 8px;
  width: 60%;
  border-radius: 2px;
  background: #F5C542;
  opacity: 0.6;
}

.preview-file {
  height: 6px;
  border-radius: 2px;
  background: var(--text-muted);
  opacity: 0.3;
}

.preview-file:last-child { width: 70%; }

.theme-label {
  font-size: 13px;
  font-weight: 500;
}

/* Language options */
.lang-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.lang-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  background: var(--bg-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.lang-card:hover {
  border-color: var(--text-muted);
}

.lang-card.active {
  border-color: var(--accent);
  background: var(--bg-hover);
}

.lang-flag {
  font-size: 24px;
}

.lang-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.lang-name {
  font-size: 14px;
  font-weight: 500;
}

.lang-native {
  font-size: 12px;
  color: var(--text-muted);
}

.check-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

/* Font size options */
.font-options {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 10px;
}

.font-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 14px 12px;
  border: 2px solid var(--border);
  border-radius: 10px;
  background: var(--bg-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.font-card:hover {
  border-color: var(--text-muted);
}

.font-card.active {
  border-color: var(--accent);
  background: var(--bg-hover);
}

.font-sample {
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.font-label {
  font-size: 12px;
  color: var(--text-muted);
}

/* About */
.about-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--bg-primary);
  border-radius: 10px;
  border: 1px solid var(--border);
}

.about-icon {
  width: 56px;
  height: 56px;
  flex-shrink: 0;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.15));
}

.about-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.about-name {
  font-size: 16px;
  font-weight: 600;
}

.about-version {
  font-size: 12px;
  color: var(--text-muted);
}

.about-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.about-tech {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  padding: 2px 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
  display: inline-block;
  width: fit-content;
}
</style>
