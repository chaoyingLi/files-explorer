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
                            <div class="theme-grid">
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
                                        class="theme-bars"
                                        :style="{ background: opt.preview.bg }"
                                    >
                                        <div
                                            class="tb-bar"
                                            :style="{
                                                background: opt.preview.sidebar,
                                            }"
                                        ></div>
                                        <div class="tb-spacer"></div>
                                        <div
                                            class="tb-dot"
                                            :style="{
                                                background: opt.preview.accent,
                                            }"
                                        ></div>
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

                        <div class="setting-group">
                            <div class="group-title">
                                {{ t("settings.iconTheme") }}
                            </div>
                            <div class="theme-grid">
                                <button
                                    class="theme-card"
                                    :class="{
                                        active: settings.iconTheme === 'fluent',
                                    }"
                                    @click="settings.setIconTheme('fluent')"
                                >
                                    <div class="icon-preview fluent-preview">
                                        <div class="ip-file ip-blue">JS</div>
                                        <div class="ip-file ip-green">VU</div>
                                        <div class="ip-folder"></div>
                                    </div>
                                    <span class="theme-label">{{
                                        t("settings.fluent")
                                    }}</span>
                                </button>
                                <button
                                    class="theme-card"
                                    :class="{
                                        active:
                                            settings.iconTheme === 'material',
                                    }"
                                    @click="settings.setIconTheme('material')"
                                >
                                    <div class="icon-preview material-preview">
                                        <div
                                            class="mp-file"
                                            style="background: #3178c6"
                                        >
                                            TS
                                        </div>
                                        <div
                                            class="mp-file"
                                            style="background: #41b883"
                                        >
                                            VU
                                        </div>
                                        <div
                                            class="mp-folder"
                                            style="background: #f6c23a"
                                        >
                                            FD
                                        </div>
                                    </div>
                                    <span class="theme-label">{{
                                        t("settings.material")
                                    }}</span>
                                    <span class="theme-label">{{
                                        t("settings.material")
                                    }}</span>
                                </button>
                                <button
                                    class="theme-card"
                                    :class="{
                                        active:
                                            settings.iconTheme ===
                                            'material-full',
                                    }"
                                    @click="
                                        settings.setIconTheme('material-full')
                                    "
                                >
                                    <div
                                        class="icon-preview material-full-preview"
                                    >
                                        <img
                                            src="/icons/typescript.svg"
                                            width="20"
                                            height="20"
                                        />
                                        <img
                                            src="/icons/vue.svg"
                                            width="20"
                                            height="20"
                                        />
                                        <img
                                            src="/icons/folder-src.svg"
                                            width="20"
                                            height="20"
                                        />
                                    </div>
                                    <span class="theme-label">{{
                                        t("settings.materialFull")
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
                        <button
                            class="about-clear-btn"
                            @click="$emit('clearCache')"
                        >
                            <svg
                                viewBox="0 0 14 14"
                                width="14"
                                height="14"
                                fill="none"
                            >
                                <circle
                                    cx="7"
                                    cy="7"
                                    r="5.5"
                                    stroke="currentColor"
                                    stroke-width="1.2"
                                />
                                <path
                                    d="M5 5l4 4M9 5l-4 4"
                                    stroke="currentColor"
                                    stroke-width="1.2"
                                    stroke-linecap="round"
                                />
                            </svg>
                            {{ $t("tray.clearCache") }}
                        </button>
                        <div class="about-pillars">
                            <span
                                class="pillar-chip"
                                :title="$t('settings.pillarLightweightTooltip')"
                            >
                                <svg
                                    viewBox="0 0 14 14"
                                    width="13"
                                    height="13"
                                    fill="none"
                                >
                                    <path
                                        d="M7 2L3 7l4 5 4-5-4-5z"
                                        stroke="currentColor"
                                        stroke-width="1.1"
                                        stroke-linejoin="round"
                                    />
                                    <path
                                        d="M4 7h6"
                                        stroke="currentColor"
                                        stroke-width="1.1"
                                        stroke-linecap="round"
                                    />
                                </svg>
                                {{ $t("settings.pillarLightweight") }}
                            </span>
                            <span
                                class="pillar-chip"
                                :title="$t('settings.pillarFastTooltip')"
                            >
                                <svg
                                    viewBox="0 0 14 14"
                                    width="13"
                                    height="13"
                                    fill="none"
                                >
                                    <path
                                        d="M6 3.5L11 7l-5 3.5v-7z"
                                        fill="currentColor"
                                        opacity="0.8"
                                    />
                                    <path
                                        d="M3 3.5l2.5 1.75v3.5L3 10.5v-7z"
                                        fill="currentColor"
                                        opacity="0.3"
                                    />
                                </svg>
                                {{ $t("settings.pillarFast") }}
                            </span>
                            <span
                                class="pillar-chip"
                                :title="$t('settings.pillarModernTooltip')"
                            >
                                <svg
                                    viewBox="0 0 14 14"
                                    width="13"
                                    height="13"
                                    fill="none"
                                >
                                    <circle
                                        cx="7"
                                        cy="7"
                                        r="4"
                                        stroke="currentColor"
                                        stroke-width="1.1"
                                    />
                                    <circle
                                        cx="7"
                                        cy="7"
                                        r="1.5"
                                        fill="currentColor"
                                        opacity="0.7"
                                    />
                                    <path
                                        d="M7 3v1.5M7 9.5v1.5M3 7h1.5M9.5 7H11"
                                        stroke="currentColor"
                                        stroke-width="1"
                                        stroke-linecap="round"
                                    />
                                </svg>
                                {{ $t("settings.pillarModern") }}
                            </span>
                            <span
                                class="pillar-chip"
                                :title="$t('settings.pillarSmartTooltip')"
                            >
                                <svg
                                    viewBox="0 0 14 14"
                                    width="13"
                                    height="13"
                                    fill="none"
                                >
                                    <path
                                        d="M7 2v3M4 5l3-3 3 3"
                                        stroke="currentColor"
                                        stroke-width="1.1"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    />
                                    <path
                                        d="M3 7h8l-1 5H4L3 7z"
                                        stroke="currentColor"
                                        stroke-width="1.1"
                                        stroke-linejoin="round"
                                    />
                                    <circle
                                        cx="7"
                                        cy="9"
                                        r=".7"
                                        fill="currentColor"
                                    />
                                </svg>
                                {{ $t("settings.pillarSmart") }}
                            </span>
                        </div>
                    </div>

                    <!-- General -->
                    <div v-if="activeTab === 'general'" class="tab-panel">
                        <div class="setting-group">
                            <div class="group-title">
                                {{ t("settings.general") }}
                            </div>
                            <label
                                class="switch-row"
                                @click="
                                    settings.setAutoStart(!settings.autoStart)
                                "
                            >
                                <div class="switch-text">
                                    <span>{{ t("settings.autoStart") }}</span>
                                    <span class="switch-desc">{{
                                        t("settings.autoStartDesc")
                                    }}</span>
                                </div>
                                <div
                                    class="switch-track"
                                    :class="{ active: settings.autoStart }"
                                >
                                    <div class="switch-thumb" />
                                </div>
                            </label>
                            <label
                                class="switch-row"
                                @click="
                                    settings.setShowTray(!settings.showTray)
                                "
                            >
                                <div class="switch-text">
                                    <span>{{ t("settings.showTray") }}</span>
                                    <span class="switch-desc">{{
                                        t("settings.showTrayDesc")
                                    }}</span>
                                </div>
                                <div
                                    class="switch-track"
                                    :class="{ active: settings.showTray }"
                                >
                                    <div class="switch-thumb" />
                                </div>
                            </label>
                            <label
                                class="switch-row"
                                @click="
                                    settings.setQuitOnClose(
                                        !settings.quitOnClose,
                                    )
                                "
                            >
                                <div class="switch-text">
                                    <span>{{ t("settings.quitOnClose") }}</span>
                                    <span class="switch-desc">{{
                                        t("settings.quitOnCloseDesc")
                                    }}</span>
                                </div>
                                <div
                                    class="switch-track"
                                    :class="{ active: settings.quitOnClose }"
                                >
                                    <div class="switch-thumb" />
                                </div>
                            </label>
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
const activeTab = ref("general");

defineEmits<{ close: []; clearCache: [] }>();

const tabs = [
    {
        id: "general",
        labelKey: "settings.tabGeneral",
        icon: `<svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="3" stroke="currentColor" stroke-width="1.3"/><path d="M8 1v3M8 12v3M1 8h3M12 8h3M3 3l2.5 2.5M10.5 10.5L13 13M3 13l2.5-2.5M10.5 5.5L13 3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>`,
    },
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

const themeOptions: {
    value: ThemeMode;
    labelKey: string;
    preview: { bg: string; sidebar: string; accent: string };
}[] = [
    {
        value: "dark",
        labelKey: "settings.catppuccinDark",
        preview: { bg: "#1E1E2E", sidebar: "#11111B", accent: "#89B4FA" },
    },
    {
        value: "light",
        labelKey: "settings.catppuccinLight",
        preview: { bg: "#EFF1F5", sidebar: "#CCD0DA", accent: "#1E66F5" },
    },
    {
        value: "nord",
        labelKey: "settings.nord",
        preview: { bg: "#2E3440", sidebar: "#434C5E", accent: "#88C0D0" },
    },
    {
        value: "tokyo-night",
        labelKey: "settings.tokyoNight",
        preview: { bg: "#1A1B26", sidebar: "#1F202A", accent: "#7AA2F7" },
    },
    {
        value: "one-dark-pro",
        labelKey: "settings.oneDarkPro",
        preview: { bg: "#282C34", sidebar: "#1B1E24", accent: "#61AFEF" },
    },
    {
        value: "dracula",
        labelKey: "settings.dracula",
        preview: { bg: "#282A36", sidebar: "#1B1D26", accent: "#BD93F9" },
    },
    {
        value: "solarized-light",
        labelKey: "settings.solarizedLight",
        preview: { bg: "#FDF6E3", sidebar: "#E0D8C3", accent: "#268BD2" },
    },
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
    height: 840px;
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

/* ── Header ── */
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

/* ── Layout ── */
.settings-layout {
    display: flex;
    flex: 1;
    min-height: 0;
}

/* ── Tabs ── */
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

/* ── Content ── */
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

/* ── Theme grid ── */
.theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
}

.theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 10px 8px;
    border: 2px solid var(--border);
    border-radius: 8px;
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

.theme-bars {
    width: 100%;
    height: 28px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    padding: 4px 6px;
    gap: 4px;
}

.tb-bar {
    width: 20%;
    height: 100%;
    border-radius: 2px;
    opacity: 0.7;
}

.tb-spacer {
    flex: 1;
}

.tb-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
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

.about-pillars {
    display: flex;
    justify-content: center;
    gap: 6px;
    flex-wrap: wrap;
}

.pillar-chip {
    cursor: default;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    padding: 4px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 16px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
}

.about-clear-btn {
    margin-top: 14px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    transition:
        color 0.15s,
        border-color 0.15s;
}
.about-clear-btn:hover {
    color: var(--danger);
    border-color: var(--danger);
}

.icon-preview {
    width: 100%;
    height: 48px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 6px;
}
.fluent-preview {
    background: var(--bg-primary);
    border: 1px solid var(--border);
}
.material-preview {
    background: var(--bg-primary);
    border: 1px solid var(--border);
}
.ip-file {
    width: 22px;
    height: 26px;
    border-radius: 3px 3px 0 0;
    border: 1.5px solid;
    border-bottom: 3px solid;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    font-size: 7px;
    font-weight: 700;
    padding-bottom: 1px;
}
.ip-blue {
    border-color: #4a6a9a;
    color: #4a6a9a;
}
.ip-green {
    border-color: #2d7d46;
    color: #2d7d46;
}
.ip-folder {
    width: 22px;
    height: 18px;
    border-radius: 3px;
    background: #f6c23a;
}
.mp-file,
.mp-folder {
    width: 24px;
    height: 24px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    font-size: 8px;
    font-weight: 700;
}

/* ── Checkbox ── */
.switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s;
    gap: 16px;
}
.switch-row:hover {
    background: var(--bg-hover);
}
.switch-text {
    flex: 1;
    min-width: 0;
}
.switch-text span {
    display: block;
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
}
.switch-desc {
    color: var(--text-muted);
    font-size: 11px;
    margin-top: 2px;
}
.switch-track {
    width: 40px;
    height: 22px;
    border-radius: 11px;
    background: var(--bg-hover);
    flex-shrink: 0;
    position: relative;
    transition: background 0.2s;
    border: 1px solid var(--border);
}
.switch-track.active {
    background: var(--accent);
    border-color: var(--accent);
}
.switch-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}
.switch-track.active .switch-thumb {
    transform: translateX(18px);
}
</style>
