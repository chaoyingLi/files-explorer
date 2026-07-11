<template>
    <div class="this-pc">
        <!-- ═══ 设备与驱动器 ═══ -->
        <div class="section-header" @click="expanded.drives = !expanded.drives">
            <span class="section-arrow">{{ expanded.drives ? "▼" : "▶" }}</span>
            {{ t("fileList.devicesAndDrives") }}
        </div>

        <!-- 骨架屏 -->
        <div v-if="store.loading && !store.drives.length" class="drives-grid">
            <div v-for="n in 4" :key="n" class="drive-card skeleton" />
        </div>

        <!-- 驱动器 -->
        <div v-show="expanded.drives && !store.loading" class="drives-grid">
            <div
                v-for="drive in store.drives"
                :key="drive.mount_point"
                class="drive-card"
                :class="{
                    'drive-card--selected': selectedDrive === drive.mount_point,
                    'drive-card--danger': usePercent(drive) > 90,
                }"
                @click="selectedDrive = drive.mount_point"
                @dblclick="store.openDrive(drive)"
                @contextmenu.prevent="emit('driveContextMenu', drive, $event)"
            >
                <svg class="drive-card-icon" viewBox="0 0 48 48">
                    <template v-if="drive.file_system === 'smb' || drive.file_system === 'nfs'">
                        <!-- 网络驱动器 -->
                        <rect x="6" y="10" width="36" height="28" rx="4" fill="#4A6FA5" />
                        <rect x="10" y="14" width="28" height="20" rx="2" fill="#3A5585" />
                        <circle cx="24" cy="24" r="3" fill="#8AB4F8" />
                        <line x1="18" y1="20" x2="30" y2="28" stroke="#fff" stroke-width="1.5" opacity="0.6" />
                    </template>
                    <template v-else-if="drive.name?.startsWith('/')">
                        <!-- macOS/Linux 挂载点 -->
                        <rect x="6" y="10" width="36" height="28" rx="4" fill="#6C7086" />
                        <rect x="10" y="14" width="28" height="20" rx="2" fill="#45475A" />
                        <circle cx="18" cy="24" r="4" fill="#F5C542" />
                        <circle cx="18" cy="24" r="2" fill="#F9E2AF" />
                    </template>
                    <template v-else>
                        <!-- Windows 本地磁盘 -->
                        <rect x="6" y="10" width="36" height="28" rx="4" fill="#6C7086" />
                        <rect x="10" y="14" width="28" height="20" rx="2" fill="#45475A" />
                        <text x="24" y="29" text-anchor="middle" fill="#F5C542" font-size="14" font-weight="bold">{{ drive.name?.charAt(0) || "?" }}</text>
                    </template>
                </svg>
                <div class="drive-card-info">
                    <div class="drive-card-name">
                        <span v-if="drive.label" class="drive-label">{{ drive.label }}</span>
                        <span class="drive-letter">{{ drive.name }}</span>
                    </div>
                    <div v-if="drive.total_space > 0" class="drive-space">
                        <div class="drive-progress">
                            <div
                                class="drive-progress-bar"
                                :style="{ width: usePercent(drive) + '%' }"
                            />
                        </div>
                        <div class="drive-space-text">
                            {{ formatSize(drive.available_space) }} / {{ formatSize(drive.total_space) }}
                        </div>
                    </div>
                    <div v-else class="drive-space-text">{{ drive.file_system }}</div>
                </div>
            </div>
        </div>

        <!-- ═══ 文件夹 ═══ -->
        <div v-if="!store.loading" class="section-header" @click="expanded.folders = !expanded.folders">
            <span class="section-arrow">{{ expanded.folders ? "▼" : "▶" }}</span>
            {{ t("fileList.folders") }}
        </div>
        <div v-show="expanded.folders" class="drives-grid">
            <div
                v-for="item in quickAccessFolders"
                :key="item.path"
                class="drive-card"
                @dblclick="store.navigateTo(item.path)"
            >
                <svg class="drive-card-icon" viewBox="0 0 48 48">
                    <path
                        d="M4 12a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v18a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                        :fill="item.color + '33'"
                    />
                    <path
                        d="M4 15a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v16a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                        :fill="item.color"
                    />
                </svg>
                <div class="drive-card-label">{{ item.name }}</div>
            </div>
        </div>

        <!-- ═══ 最近文件 ═══ -->
        <div v-if="recentItems.length > 0 || store.loading" class="section-header" @click="expanded.recent = !expanded.recent">
            <span class="section-arrow">{{ expanded.recent ? "▼" : "▶" }}</span>
            {{ t("thisPc.recentFiles") }}
        </div>

        <!-- 骨架屏 -->
        <div v-if="store.loading" v-show="expanded.recent" class="recent-table" style="border-color: transparent">
            <div v-for="n in 4" :key="n" class="recent-row skeleton-row">
                <div class="skeleton-block" style="width: 28px; height: 28px" />
                <div class="skeleton-block" style="flex:1; height:14px" />
                <div class="skeleton-block" style="width: 60px; height: 12px" />
            </div>
        </div>

        <!-- 最近文件表 -->
        <div v-show="expanded.recent && recentItems.length > 0 && !store.loading" class="recent-table">
            <div class="recent-header">
                <span class="recent-col-name">{{ t("fileList.name") }}</span>
                <span class="recent-col-type">{{ t("fileList.type") }}</span>
                <span class="recent-col-path">{{ t("properties.fullPath") }}</span>
                <span class="recent-col-time">{{ t("fileList.dateModified") }}</span>
            </div>
            <div
                v-for="(item, idx) in displayedRecentItems"
                :key="item.path"
                class="recent-row"
                :class="{ 'recent-row--last': idx === displayedRecentItems.length - 1 }"
                :style="{ '--stripe-color': stripeColor(item) }"
                @dblclick="store.navigateTo(item.path)"
                @contextmenu.prevent="onRecentCtx(item, $event)"
            >
                <svg
                    v-if="item.isDir"
                    class="recent-icon"
                    viewBox="0 0 24 24"
                >
                    <path
                        d="M2 7c0-1.1.9-2 2-2h4a2 2 0 011.5.6l1 1.2a.7.7 0 00.5.2H19c1.1 0 2 .9 2 2v7c0 1.1-.9 2-2 2H4a2 2 0 01-2-2V7z"
                        fill="var(--folder-back, #F5C542)"
                    />
                </svg>
                <svg v-else class="recent-icon" viewBox="0 0 24 24">
                    <path d="M6 2h7l5 5v13a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z" fill="var(--file-icon-primary, #89b4fa)" />
                    <text x="12" y="16" text-anchor="middle" font-size="6" fill="#1e1e2e" font-weight="bold">{{ item.ext.split('.').pop()?.slice(0, 3).toUpperCase() || "?" }}</text>
                </svg>
                <div class="recent-row-main">
                    <span class="recent-row-name">{{ item.name }}</span>
                    <span class="recent-row-meta">
                        <span class="recent-row-path" :title="dirPath(item.path)">{{ dirPath(item.path) }}</span>
                        <span class="recent-row-time">{{ formatTime(item.time) }}</span>
                    </span>
                </div>
            </div>
            <!-- 查看更多 -->
            <div v-if="recentItems.length > MAX_RECENT" class="recent-more" @click="showAllRecent = !showAllRecent">
                {{ showAllRecent ? t("thisPc.showLess") : t("thisPc.showMore", { n: recentItems.length - MAX_RECENT }) }}
            </div>
        </div>

        <!-- 空状态 -->
        <div v-if="!store.loading && recentItems.length === 0" class="empty-state">
            {{ t("thisPc.noRecentFiles") }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import type { DiskInfo } from "@/types";
import { formatFileSize } from "@/utils/fileTypes";

const { t } = useI18n();
const store = useFileStore();

const emit = defineEmits<{
    recentContextMenu: [path: string, e: MouseEvent];
    driveContextMenu: [drive: DiskInfo, e: MouseEvent];
}>();

const MAX_RECENT = 20;

const expanded = reactive({ drives: true, folders: true, recent: true });
const selectedDrive = ref<string | null>(null);
const showAllRecent = ref(false);

// 快速访问文件夹（带颜色）
const folderColors: Record<string, string> = {
    desktop: "#4FC3F7",
    downloads: "#81C784",
    documents: "#64B5F6",
    pictures: "#E57373",
    music: "#BA68C8",
    videos: "#FF8A65",
};

const quickAccessFolders = computed(() => {
    const dirs = store.specialDirs;
    if (!dirs) return [];
    const map: [string, string, string][] = [
        ["sidebar.desktop", dirs.desktop, "desktop"],
        ["sidebar.downloads", dirs.downloads, "downloads"],
        ["sidebar.documents", dirs.documents, "documents"],
        ["sidebar.pictures", dirs.pictures, "pictures"],
        ["sidebar.music", dirs.music, "music"],
        ["sidebar.videos", dirs.videos, "videos"],
    ];
    return map.map(([key, path, id]) => ({
        name: t(key),
        path,
        color: folderColors[id] || "#F5C542",
    }));
});

const recentItems = computed(() => store.recentItems);
const displayedRecentItems = computed(() =>
    showAllRecent.value ? recentItems.value : recentItems.value.slice(0, MAX_RECENT),
);

// 条纹颜色映射
function stripeColor(item: { isDir: boolean; ext: string }): string {
    if (item.isDir) return "#F5C542";
    const ext = item.ext?.toLowerCase() || "";
    if (/png|jpg|jpeg|gif|webp|svg|bmp|ico/.test(ext)) return "#4CAF50";
    if (/mp4|mkv|avi|mov|webm/.test(ext)) return "#E91E63";
    if (/mp3|wav|flac|aac|ogg/.test(ext)) return "#9C27B0";
    if (/doc|docx|pdf|xls|xlsx|ppt|pptx|txt|md/.test(ext)) return "#2196F3";
    if (/js|ts|vue|html|css|json|py|rs|go|java/.test(ext)) return "#FF9800";
    if (/zip|rar|7z|tar|gz|bz2|xz/.test(ext)) return "#795548";
    return "#6C7086";
}

function dirPath(p: string): string {
    const parts = p.replace(/\\/g, "/").split("/");
    parts.pop();
    return parts.join("/") || "/";
}

function formatTime(ts: number): string {
    const diff = Date.now() - ts;
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    if (mins < 1) return t("thisPc.justNow");
    if (mins < 60) return mins + t("thisPc.minutesAgo");
    if (hours < 24) return hours + t("thisPc.hoursAgo");
    if (days < 7) return days + t("thisPc.daysAgo");
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
/* ── 布局 ── */
.this-pc {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
}

/* ── 区域标题 ── */
.section-header {
    font-size: var(--font-size-lg);
    font-weight: 600;
    margin-bottom: 12px;
    margin-top: 24px;
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 6px;
}
.section-header:first-child { margin-top: 0; }
.section-arrow {
    font-size: 10px;
    width: 14px;
    text-align: center;
    color: var(--text-muted);
}

/* ── 驱动器/文件夹网格 ── */
.drives-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 10px;
}

.drive-card {
    display: flex;
    flex-direction: column;
    padding: 14px 16px;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.1s, border-color 0.15s, transform 0.1s;
    border: 1px solid var(--border);
}
.drive-card:hover { background: var(--bg-hover); transform: translateY(-1px); }
.drive-card--selected {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
}
.drive-card--danger .drive-progress-bar { background: var(--danger, #ef4444); }

.drive-card-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}

.drive-card-icon {
    width: 44px; height: 44px;
    flex-shrink: 0;
    filter: drop-shadow(0 1px 3px rgba(0,0,0,0.15));
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
    font-size: var(--font-size-base);
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    word-break: break-word;
}
.drive-letter {
    font-size: var(--font-size-sm);
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
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.drive-card-label {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    text-align: center;
    margin-top: 4px;
}

/* ── 最近文件表 ── */
.recent-table {
    font-size: var(--font-size-sm);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
}
.recent-header {
    display: flex;
    padding: 8px 12px;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary, #181825);
}
.recent-row {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    cursor: pointer;
    transition: background 0.08s, transform 0.1s;
    position: relative;
    gap: 10px;
}
.recent-row::before {
    content: "";
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 3px;
    background: var(--stripe-color);
    border-radius: 0 2px 2px 0;
}
.recent-row:hover {
    background: var(--bg-hover);
    transform: translateX(2px);
}
.recent-row:not(.recent-row--last) {
    border-bottom: 1px solid var(--border-subtle, rgba(128,128,128,0.12));
}
.recent-icon {
    width: 20px; height: 20px;
    flex-shrink: 0;
}
.recent-row-main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}
.recent-row-name {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
}
.recent-row-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}
.recent-row-path {
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
}
.recent-row-time {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
}

/* ── 隐藏旧列头（改用两行布局后不需要了） ── */
.recent-header { display: none; }

/* ── 查看更多 ── */
.recent-more {
    padding: 8px;
    text-align: center;
    font-size: var(--font-size-sm);
    color: var(--accent);
    cursor: pointer;
    border-top: 1px solid var(--border-subtle, rgba(128,128,128,0.12));
    transition: background 0.1s;
}
.recent-more:hover { background: var(--bg-hover); }

/* ── 骨架屏 ── */
@keyframes shimmer {
    0% { opacity: 0.4; }
    50% { opacity: 0.7; }
    100% { opacity: 0.4; }
}
.drive-card.skeleton {
    height: 120px;
    background: var(--bg-hover);
    animation: shimmer 1.5s infinite;
    cursor: default;
    border-color: transparent;
}
.drive-card.skeleton:hover { transform: none; }
.skeleton-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    cursor: default;
}
.skeleton-row:hover { transform: none; }
.skeleton-block {
    background: var(--bg-hover);
    border-radius: 4px;
    animation: shimmer 1.5s infinite;
}

/* ── 空状态 ── */
.empty-state {
    text-align: center;
    padding: 32px 16px;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
}
</style>
