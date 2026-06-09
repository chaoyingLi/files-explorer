<template>
    <div class="sidebar">
        <!-- Home / This PC -->
        <div class="sidebar-list">
            <div
                class="sidebar-item home-item"
                :class="{ active: !store.currentPath }"
                @click="navigateHome"
            >
                <svg class="sidebar-icon home-icon" viewBox="0 0 24 24">
                    <path
                        d="M4 10.5V20a1 1 0 001 1h5v-6h4v6h5a1 1 0 001-1v-9.5L12 3l-8 7.5z"
                        fill="#89B4FA"
                    />
                    <path d="M13 21v-5h-2v5" fill="#74C7EC" />
                </svg>
                <span class="sidebar-item-name">{{ t("sidebar.thisPc") }}</span>
            </div>
        </div>

        <div class="sidebar-header">
            <span class="sidebar-title">{{ t("sidebar.drives") }}</span>
        </div>
        <div class="sidebar-list">
            <div
                v-for="drive in store.drives"
                :key="drive.mount_point"
                class="sidebar-item"
                :class="{ active: isDriveActive(drive) }"
                @click="openDrive(drive)"
            >
                <svg class="sidebar-icon drive-icon" viewBox="0 0 24 24">
                    <rect
                        x="3"
                        y="5"
                        width="18"
                        height="14"
                        rx="2.5"
                        fill="#6C7086"
                    />
                    <rect
                        x="5"
                        y="7"
                        width="14"
                        height="10"
                        rx="1.5"
                        fill="#45475A"
                    />
                    <circle cx="9" cy="12" r="2.5" fill="#F5C542" />
                    <circle cx="9" cy="12" r="1.2" fill="#F9E2AF" />
                </svg>
                <div class="sidebar-item-info">
                    <span class="sidebar-item-name">{{ drive.name }}</span>
                    <span class="sidebar-item-meta">{{
                        drive.file_system
                    }}</span>
                </div>
            </div>
        </div>

        <div class="sidebar-header">
            <span class="sidebar-title">{{ t("sidebar.quickAccess") }}</span>
        </div>
        <div class="sidebar-list">
            <div
                v-for="item in quickAccess"
                :key="item.path"
                class="sidebar-item"
                :class="{ active: isPathActive(item.path) }"
                @click="handleQuickAccess(item.path)"
            >
                <svg
                    class="sidebar-icon"
                    viewBox="0 0 24 24"
                    v-html="item.iconSvg"
                ></svg>
                <span class="sidebar-item-name">{{ item.name }}</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import type { DiskInfo } from "@/types";

const { t } = useI18n();
const store = useFileStore();

const emit = defineEmits<{
    navigate: [path: string];
    navigateHome: [];
}>();

const quickAccess = computed(() => {
    const items: { name: string; path: string; iconSvg: string }[] = [];
    const dirs = store.specialDirs;
    if (!dirs) return items;

    // Home icon
    items.push({
        name: t("sidebar.home"),
        path: dirs.home,
        iconSvg:
            '<path d="M4 10.5V20a1 1 0 001 1h5v-6h4v6h5a1 1 0 001-1v-9.5L12 3l-8 7.5z" fill="#89B4FA"/><path d="M13 21v-5h-2v5" fill="#74C7EC"/>',
    });
    // Desktop
    items.push({
        name: t("sidebar.desktop"),
        path: dirs.desktop,
        iconSvg:
            '<rect x="2" y="3" width="20" height="14" rx="2" fill="#89B4FA"/><rect x="4" y="18" width="16" height="2" rx="1" fill="#585B70"/><rect x="10" y="20" width="4" height="2" rx="1" fill="#45475A"/>',
    });
    // Downloads
    items.push({
        name: t("sidebar.downloads"),
        path: dirs.downloads,
        iconSvg:
            '<rect x="4" y="2" width="16" height="14" rx="2" fill="#89B4FA"/><path d="M12 8v8m0 0l-3-3m3 3l3-3" stroke="#1E1E2E" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" opacity="0.35"/><path d="M4 18h16" stroke="#585B70" stroke-width="2.5" stroke-linecap="round"/>',
    });
    // Documents
    items.push({
        name: t("sidebar.documents"),
        path: dirs.documents,
        iconSvg:
            '<path d="M6 2h7.5l5.5 5.5V20a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z" fill="#89B4FA"/><path d="M13.5 2v5.5a.5.5 0 00.5.5H19.5" fill="#74C7EC"/><rect x="7" y="11" width="10" height="1.5" rx="0.75" fill="#1E1E2E" opacity="0.3"/><rect x="7" y="14" width="7" height="1.5" rx="0.75" fill="#1E1E2E" opacity="0.3"/>',
    });
    // Pictures
    items.push({
        name: t("sidebar.pictures"),
        path: dirs.pictures,
        iconSvg:
            '<rect x="2" y="4" width="20" height="16" rx="2.5" fill="#A6E3A1"/><circle cx="7.5" cy="9.5" r="2" fill="#1E1E2E" opacity="0.15"/><path d="M2 16l5-5 4 4 3-3 6 6" fill="#1E1E2E" opacity="0.12"/>',
    });
    // Music
    items.push({
        name: t("sidebar.music"),
        path: dirs.music,
        iconSvg:
            '<circle cx="7" cy="17" r="3.5" fill="#F38BA8"/><circle cx="17" cy="13" r="3.5" fill="#F9E2AF"/><path d="M10.5 17V6l6-2v9" stroke="#F38BA8" stroke-width="2.5" stroke-linecap="round"/><path d="M10.5 17V6l6-2v9" stroke="#F5C542" stroke-width="2.5" stroke-linecap="round" stroke-dasharray="0 13 16 0"/>',
    });
    // Videos
    items.push({
        name: t("sidebar.videos"),
        path: dirs.videos,
        iconSvg:
            '<rect x="2" y="4" width="15" height="16" rx="2.5" fill="#CBA6F7"/><path d="M17 10l5-3.5v11L17 14" fill="#CBA6F7"/><path d="M17 10l5-3.5v11L17 14" fill="#1E1E2E" opacity="0.12"/><path d="M9 13l3-2v4z" fill="#1E1E2E" opacity="0.25"/>',
    });

    return items;
});

function isDriveActive(drive: DiskInfo) {
    return store.currentPath && store.currentPath.startsWith(drive.name);
}

function isPathActive(path: string) {
    return store.currentPath === path;
}

function navigateHome() {
    emit("navigateHome");
}

function openDrive(drive: DiskInfo) {
    emit("navigate", drive.name);
}

async function handleQuickAccess(path: string) {
    emit("navigate", path);
}
</script>

<style scoped>
.sidebar {
    width: 220px;
    min-width: 180px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 8px 0;
}
.home-item {
    margin-bottom: 6px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
}
.sidebar-header {
    padding: 8px 16px 4px;
}
.sidebar-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.5px;
}
.sidebar-list {
    padding: 4px 8px;
}
.sidebar-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.1s;
}
.sidebar-item:hover {
    background: var(--bg-hover);
}
.sidebar-item.active {
    background: var(--bg-selected);
}
.sidebar-icon {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.15));
}
.drive-icon {
    color: var(--text-muted);
}
.sidebar-item-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
}
.sidebar-item-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.sidebar-item-meta {
    font-size: 11px;
    color: var(--text-muted);
}
</style>
