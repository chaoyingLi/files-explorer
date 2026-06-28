<template>
    <div class="this-pc">
        <div class="this-pc-header">
            {{ t("fileList.devicesAndDrives") }}
        </div>
        <div class="drives-grid">
            <div
                v-for="drive in store.drives"
                :key="drive.mount_point"
                class="drive-card"
                @dblclick="store.openDrive(drive)"
            >
                <div class="drive-card-top">
                    <svg class="drive-card-icon" viewBox="0 0 48 48">
                        <rect
                            x="6"
                            y="10"
                            width="36"
                            height="28"
                            rx="4"
                            fill="#6C7086"
                        />
                        <rect
                            x="10"
                            y="14"
                            width="28"
                            height="20"
                            rx="2"
                            fill="#45475A"
                        />
                        <circle cx="18" cy="24" r="4" fill="#F5C542" />
                        <circle cx="18" cy="24" r="2" fill="#F9E2AF" />
                    </svg>
                    <div class="drive-card-info">
                        <div class="drive-card-name">
                            <span v-if="drive.label" class="drive-label">{{
                                drive.label
                            }}</span>
                            <span class="drive-letter">{{ drive.name }}</span>
                        </div>
                        <div v-if="drive.total_space > 0" class="drive-space">
                            <div class="drive-progress">
                                <div
                                    class="drive-progress-bar"
                                    :style="{ width: usePercent(drive) + '%' }"
                                ></div>
                            </div>
                            <div class="drive-space-text">
                                {{ formatSize(drive.available_space) }} free of
                                {{ formatSize(drive.total_space) }}
                            </div>
                        </div>
                        <div v-else class="drive-space-text">
                            {{ drive.file_system }}
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div class="this-pc-header" style="margin-top: 24px">
            {{ t("fileList.folders") }}
        </div>
        <div class="drives-grid">
            <div
                v-for="item in quickAccessFolders"
                :key="item.path"
                class="drive-card"
                @dblclick="store.navigateTo(item.path)"
            >
                <svg class="drive-card-icon folder-icon" viewBox="0 0 48 48">
                    <path
                        d="M4 12a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v18a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                        fill="#DEB949"
                    />
                    <path
                        d="M4 15a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v16a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                        fill="#F5C542"
                    />
                </svg>
                <div class="drive-card-label">{{ item.name }}</div>
            </div>
        </div>
        <!-- Recent items -->
        <div v-if="recentItems.length > 0">
            <div class="this-pc-header" style="margin-top: 24px">
                {{ t("thisPc.recentFiles") }}
            </div>
            <div class="recent-table">
                <div class="recent-header">
                    <span class="recent-col-name">{{
                        t("fileList.name")
                    }}</span>
                    <span class="recent-col-type">{{
                        t("fileList.type")
                    }}</span>
                    <span class="recent-col-path">{{
                        t("properties.fullPath")
                    }}</span>
                    <span class="recent-col-time">{{
                        t("fileList.dateModified")
                    }}</span>
                </div>
                <div
                    v-for="item in recentItems"
                    :key="item.path"
                    class="recent-row"
                    @dblclick="store.navigateTo(item.path)"
                    @contextmenu.prevent="onRecentCtx(item, $event)"
                >
                    <svg
                        v-if="item.isDir"
                        class="recent-icon"
                        viewBox="0 0 18 18"
                    >
                        <path
                            d="M2 5.5c0-.83.67-1.5 1.5-1.5h2.5a1.5 1.5 0 011.1.5l.7.85a.5.5 0 00.38.18H14c.83 0 1.5.67 1.5 1.5v4.5a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 012 12V5.5z"
                            fill="var(--folder-back)"
                        />
                        <path
                            d="M2 6.5c0-.83.67-1.5 1.5-1.5h2.5a1.5 1.5 0 011.1.5l.7.85a.5.5 0 00.38.18H14c.83 0 1.5.67 1.5 1.5v4a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 012 12V6.5z"
                            fill="var(--file-icon-primary)"
                        />
                    </svg>
                    <svg v-else class="recent-icon" viewBox="0 0 18 18">
                        <path
                            d="M4.5 2h5.1l3.9 3.9V14a1.5 1.5 0 01-1.5 1.5h-7.5A1.5 1.5 0 013 14V3.5A1.5 1.5 0 014.5 2z"
                            fill="var(--file-icon-primary)"
                        />
                    </svg>
                    <span class="recent-col-name">{{ item.name }}</span>
                    <span class="recent-col-type">{{ typeLabel(item) }}</span>
                    <span class="recent-col-path" :title="dirPath(item.path)">{{
                        dirPath(item.path)
                    }}</span>
                    <span class="recent-col-time">{{
                        formatTime(item.time)
                    }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import type { DiskInfo } from "@/types";
import { formatFileSize } from "@/utils/fileTypes";

const { t } = useI18n();
const store = useFileStore();

const emit = defineEmits<{
    recentContextMenu: [path: string, e: MouseEvent];
}>();

const quickAccessFolders = computed(() => {
    const items: { name: string; path: string }[] = [];
    const dirs = store.specialDirs;
    if (!dirs) return items;
    items.push({ name: t("sidebar.desktop"), path: dirs.desktop });
    items.push({ name: t("sidebar.downloads"), path: dirs.downloads });
    items.push({ name: t("sidebar.documents"), path: dirs.documents });
    items.push({ name: t("sidebar.pictures"), path: dirs.pictures });
    items.push({ name: t("sidebar.music"), path: dirs.music });
    items.push({ name: t("sidebar.videos"), path: dirs.videos });
    return items;
});

const recentItems = computed(() => store.recentItems);

function dirPath(p: string): string {
    const parts = p.replace(/\\/g, "/").split("/");
    parts.pop();
    return parts.join("/") || "/";
}

function typeLabel(item: { isDir: boolean; ext: string }): string {
    if (item.isDir) return t("fileTypes.folder");
    const ext = item.ext.toUpperCase();
    return ext ? ext + " " + t("fileTypes.file") : t("fileTypes.file");
}

function formatTime(ts: number): string {
    const now = Date.now();
    const diff = now - ts;
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    if (mins < 1) return t("thisPc.justNow");
    if (mins < 60) return mins + " " + t("thisPc.minutesAgo");
    if (hours < 24) return hours + " " + t("thisPc.hoursAgo");
    if (days < 7) return days + " " + t("thisPc.daysAgo");
    return new Date(ts).toLocaleDateString();
}

function onRecentCtx(item: { path: string }, e: MouseEvent) {
    emit("recentContextMenu", item.path, e);
}

function formatSize(bytes: number): string {
    return formatFileSize(bytes);
}

function usePercent(drive: DiskInfo): number {
    if (drive.total_space === 0) return 0;
    return Math.round((drive.used_space / drive.total_space) * 100);
}
</script>

<style scoped>
.this-pc {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
}
.this-pc-header {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--text-primary);
}
.drives-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
}
.drive-card {
    display: flex;
    flex-direction: column;
    padding: 14px 16px;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.1s;
    border: 1px solid var(--border);
    width: 200px;
    flex-shrink: 0;
}
.drive-card:hover {
    background: var(--bg-hover);
}
.drive-card-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}
.drive-card-icon {
    width: 44px;
    height: 44px;
    flex-shrink: 0;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.15));
}
.drive-card-info {
    width: 100%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
}
.drive-card-name {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
}
.drive-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    word-break: break-word;
}
.drive-letter {
    font-size: 11px;
    color: var(--text-muted);
}
.drive-space {
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.drive-progress {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
}
.drive-progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
    min-width: 2px;
}
.drive-space-text {
    font-size: 10px;
    color: var(--text-muted);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.drive-card-label {
    font-size: 12px;
    color: var(--text-primary);
    text-align: center;
    margin-top: 4px;
}
/* ── Recent items table ── */
.recent-table {
    font-size: 12px;
}
.recent-header {
    display: flex;
    padding: 6px 8px;
    color: var(--text-muted);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1px solid var(--border);
}
.recent-row {
    display: flex;
    align-items: center;
    padding: 5px 8px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.08s;
}
.recent-row:hover {
    background: var(--bg-hover);
}
.recent-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-right: 6px;
}
.recent-col-name {
    flex: 2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
}
.recent-col-type {
    flex: 1;
    color: var(--text-muted);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.recent-col-path {
    flex: 2;
    color: var(--text-muted);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
}
.recent-col-time {
    flex: 1;
    color: var(--text-muted);
    font-size: 11px;
    text-align: right;
    white-space: nowrap;
}
</style>
