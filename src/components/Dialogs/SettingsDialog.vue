<template>
    <div class="dialog-overlay" @click.self="$emit('close')">
        <div class="settings-dialog">
            <div class="settings-header">
                <h2>⚙ {{ t("settings.title") }}</h2>
                <button class="icon-btn close-btn" @click="$emit('close')">
                    <svg viewBox="0 0 20 20" fill="none">
                        <path
                            d="M5 5l10 10M15 5L5 15"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                        />
                    </svg>
                </button>
            </div>

            <div class="settings-layout">
                <!-- Tab sidebar -->
                <div class="settings-tabs">
                    <button
                        v-for="tab in tabs"
                        :key="tab.id"
                        class="settings-tab"
                        :class="{ active: activeTab === tab.id }"
                        @click="activeTab = tab.id"
                    >
                        <span class="tab-icon" v-html="tab.icon"></span>
                        <span class="tab-label">{{ t(tab.labelKey) }}</span>
                    </button>
                </div>

                <!-- Tab content -->
                <div class="settings-content">
                    <!-- Appearance -->
                    <div v-if="activeTab === 'appearance'" class="tab-panel">
                        <div class="setting-group">
                            <div class="group-title">
                                {{ t("settings.theme") }}
                            </div>
                            <div class="theme-options">
                                <button
                                    v-for="opt in themeOptions"
                                    :key="opt.value"
                                    class="theme-card"
                                    :class="{
                                        active: settings.theme === opt.value,
                                    }"
                                    @click="settings.setTheme(opt.value)"
                                >
                                    <div
                                        class="theme-preview"
                                        :class="opt.value"
                                    >
                                        <div class="preview-sidebar"></div>
                                        <div class="preview-main">
                                            <div class="preview-toolbar"></div>
                                            <div class="preview-content">
                                                <div
                                                    class="preview-folder"
                                                ></div>
                                                <div class="preview-file"></div>
                                                <div class="preview-file"></div>
                                            </div>
                                        </div>
                                    </div>
                                    <span class="theme-label">{{
                                        t(opt.labelKey)
                                    }}</span>
                                </button>
                            </div>
                        </div>

                        <div class="setting-group">
                            <div class="group-title">
                                {{ t("settings.fontSize") }}
                            </div>
                            <div class="font-options">
                                <button
                                    v-for="opt in fontSizeOptions"
                                    :key="opt.value"
                                    class="font-card"
                                    :class="{
                                        active: settings.fontSize === opt.value,
                                    }"
                                    @click="settings.setFontSize(opt.value)"
                                >
                                    <span
                                        class="font-sample"
                                        :style="{ fontSize: opt.sampleSize }"
                                        >Aa</span
                                    >
                                    <span class="font-label">{{
                                        t(opt.labelKey)
                                    }}</span>
                                </button>
                            </div>
                        </div>
                    </div>

                    <!-- Language -->
                    <div v-if="activeTab === 'language'" class="tab-panel">
                        <div class="setting-group">
                            <div class="group-title">
                                {{ t("settings.language") }}
                            </div>
                            <div class="lang-options">
                                <button
                                    v-for="opt in langOptions"
                                    :key="opt.value"
                                    class="lang-card"
                                    :class="{
                                        active: settings.locale === opt.value,
                                    }"
                                    @click="handleLocaleChange(opt.value)"
                                >
                                    <span class="lang-flag">{{
                                        opt.flag
                                    }}</span>
                                    <div class="lang-info">
                                        <span class="lang-name">{{
                                            opt.name
                                        }}</span>
                                        <span class="lang-native">{{
                                            opt.native
                                        }}</span>
                                    </div>
                                    <svg
                                        v-if="settings.locale === opt.value"
                                        class="check-icon"
                                        viewBox="0 0 20 20"
                                        fill="none"
                                    >
                                        <path
                                            d="M4 10l4 4 8-8"
                                            stroke="var(--accent)"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>

                    <!-- About -->
                    <div v-if="activeTab === 'about'" class="tab-panel">
                        <div class="about-content">
                            <img
                                class="about-icon"
                                src="/icon.png"
                                alt="Files Explorer"
                            />
                            <div class="about-info">
                                <div class="about-name">Files Explorer</div>
                                <div class="about-version">
                                    {{ t("settings.version") }}
                                    {{ APP_VERSION }}
                                </div>
                                <div class="about-desc">
                                    {{ t("settings.description") }}
                                </div>
                            </div>
                        </div>
                        <div class="about-meta">
                            <span class="meta-chip">Tauri 2.0</span>
                            <span class="meta-chip">Vue 3</span>
                            <span class="meta-chip">Rust</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settingsStore";
import { APP_VERSION } from "@/utils/version";
import type { ThemeMode, FontSize } from "@/stores/settingsStore";

const { t, locale } = useI18n();
const settings = useSettingsStore();
const activeTab = ref("appearance");

defineEmits<{ close: [] }>();

const tabs = [
    {
        id: "appearance",
        labelKey: "settings.tabAppearance",
        icon: `<svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="5" stroke="currentColor" stroke-width="1.3"/><path d="M8 3a5 5 0 000 10V3z" fill="currentColor" opacity="0.3"/></svg>`,
    },
    {
        id: "language",
        labelKey: "settings.tabLanguage",
        icon: `<svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.3"/><ellipse cx="8" cy="8" rx="3" ry="6" stroke="currentColor" stroke-width="1.1"/><path d="M2 8h12M8 2a9 9 0 010 12" stroke="currentColor" stroke-width="0.8"/></svg>`,
    },
    {
        id: "about",
        labelKey: "settings.tabAbout",
        icon: `<svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.3"/><path d="M8 7.5v4M8 5v.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>`,
    },
];

const themeOptions: { value: ThemeMode; labelKey: string }[] = [
    { value: "dark", labelKey: "settings.dark" },
    { value: "light", labelKey: "settings.light" },
];

const langOptions = [
    { value: "zh", name: "Chinese", native: "中文", flag: "🇨🇳" },
    { value: "en", name: "English", native: "English", flag: "🇺🇸" },
];

const fontSizeOptions: {
    value: FontSize;
    labelKey: string;
    sampleSize: string;
}[] = [
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
    width: 600px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 64px var(--shadow);
    animation: dialog-in 0.2s ease-out;
}

@keyframes dialog-in {
    from {
        opacity: 0;
        transform: scale(0.95) translateY(-10px);
    }
    to {
        opacity: 1;
        transform: scale(1) translateY(0);
    }
}

.settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}

.settings-header h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
}

.close-btn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
}

.settings-layout {
    display: flex;
    flex: 1;
    min-height: 0;
}

/* ── Tabs sidebar ── */
.settings-tabs {
    width: 130px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.settings-tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 6px;
    border: none;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
    text-align: left;
    transition: all 0.12s;
}

.settings-tab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.settings-tab.active {
    background: var(--accent);
    color: #fff;
}

.tab-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
}

.tab-icon :deep(svg) {
    width: 16px;
    height: 16px;
}

.tab-label {
    white-space: nowrap;
}

/* ── Content area ── */
.settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
}

.tab-panel {
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.setting-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.group-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.5px;
}

/* ── Theme ── */
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
    height: 64px;
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
}
.theme-preview.dark .preview-sidebar {
    background: #11111b;
}
.theme-preview.light .preview-sidebar {
    background: #ccd0da;
}

.preview-main {
    flex: 1;
    display: flex;
    flex-direction: column;
}

.preview-toolbar {
    height: 14px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
}

.theme-preview.dark .preview-toolbar {
    background: #181825;
}
.theme-preview.light .preview-toolbar {
    background: #e6e9ef;
}

.preview-content {
    flex: 1;
    padding: 5px;
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.preview-folder {
    height: 7px;
    width: 60%;
    border-radius: 2px;
    background: #f5c542;
    opacity: 0.6;
}

.preview-file {
    height: 5px;
    border-radius: 2px;
    background: var(--text-muted);
    opacity: 0.3;
}

.preview-file:last-child {
    width: 70%;
}

.theme-label {
    font-size: 13px;
    font-weight: 500;
}

/* ── Font size ── */
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

/* ── Language ── */
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

/* ── About ── */
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
    width: 48px;
    height: 48px;
    border-radius: 10px;
    flex-shrink: 0;
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

.about-meta {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
}

.meta-chip {
    font-size: 11px;
    color: var(--text-muted);
    padding: 3px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
}
</style>
