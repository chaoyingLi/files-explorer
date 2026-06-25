<template>
    <div class="grid-items">
        <div
            v-for="file in files"
            :key="file.path"
            class="grid-card"
            :class="{ selected: isSelected(file.path) }"
            @click="$emit('fileClick', file, $event)"
            @dblclick="$emit('fileDblClick', file, $event)"
            @contextmenu.prevent="$emit('fileContextMenu', file, $event)"
        >
            <div class="grid-icon" :class="gridColorClass(file)">
                <!-- Image thumbnail (show when loaded, hide icon for failed images too) -->
                <img
                    v-if="
                        isImage(file) &&
                        thumbSrc(file) &&
                        thumbSrc(file) !== '__error__'
                    "
                    :src="thumbSrc(file)"
                    class="grid-thumb"
                    :alt="file.name"
                    loading="lazy"
                />
                <!-- Per-extension rich SVG icon (also for bundle dirs, or image while loading) -->
                <div
                    v-else-if="richIcon(file)"
                    class="grid-rich-icon"
                    v-html="richIcon(file)"
                ></div>
                <!-- Folders -->
                <svg
                    v-else-if="file.is_dir && !isBundle(file)"
                    viewBox="0 0 48 48"
                    class="grid-folder"
                >
                    <defs>
                        <linearGradient
                            id="gf-grad"
                            x1="0"
                            y1="0"
                            x2="0"
                            y2="1"
                        >
                            <stop
                                offset="0%"
                                stop-color="var(--folder-main)"
                                stop-opacity="0.9"
                            />
                            <stop
                                offset="100%"
                                stop-color="var(--folder-shade)"
                                stop-opacity="0.95"
                            />
                        </linearGradient>
                        <linearGradient id="gf-tab" x1="0" y1="0" x2="0" y2="1">
                            <stop
                                offset="0%"
                                stop-color="var(--folder-shade)"
                                stop-opacity="0.6"
                            />
                            <stop
                                offset="100%"
                                stop-color="var(--folder-shade)"
                                stop-opacity="0.4"
                            />
                        </linearGradient>
                    </defs>
                    <path
                        d="M5 15a3.5 3.5 0 013.5-3.5h8.08c.97 0 1.88.47 2.44 1.26l2.34 3.24H39.5A3.5 3.5 0 0143 19.5v14a3.5 3.5 0 01-3.5 3.5H8.5A3.5 3.5 0 015 33.5V15z"
                        fill="url(#gf-tab)"
                    />
                    <path
                        d="M5 17.5A3.5 3.5 0 018.5 14h8.08c.97 0 1.88.47 2.44 1.26l2.34 3.24H39.5A3.5 3.5 0 0143 22v11.5a3.5 3.5 0 01-3.5 3.5H8.5A3.5 3.5 0 015 33.5V17.5z"
                        fill="url(#gf-grad)"
                    />
                </svg>
                <!-- Fallback document icon -->
                <svg v-else viewBox="0 0 48 48" class="grid-doc">
                    <defs>
                        <linearGradient
                            id="gd-grad"
                            x1="0"
                            y1="0"
                            x2="0"
                            y2="1"
                        >
                            <stop
                                offset="0%"
                                stop-color="var(--doc-main)"
                                stop-opacity="0.9"
                            />
                            <stop
                                offset="100%"
                                stop-color="var(--doc-shade)"
                                stop-opacity="0.85"
                            />
                        </linearGradient>
                    </defs>
                    <path
                        d="M12 6h18.38l9.62 9.62V38a4 4 0 01-4 4H12a4 4 0 01-4-4V10a4 4 0 014-4z"
                        fill="url(#gd-grad)"
                    />
                    <path
                        d="M30.38 6v6.62c0 .55.45 1 1 1H38"
                        fill="var(--doc-highlight)"
                        opacity="0.5"
                    />
                </svg>
            </div>
            <div class="grid-name">{{ file.name }}</div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import type { FileEntry } from "@/types";
import { getFileCategory, gridColorClassForCategory } from "@/utils/fileTypes";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";
import * as tauri from "@/utils/tauri";

defineEmits<{
    fileClick: [file: FileEntry, e: MouseEvent];
    fileDblClick: [file: FileEntry, e: MouseEvent];
    fileContextMenu: [file: FileEntry, e: MouseEvent];
}>();

const props = defineProps<{
    files: FileEntry[];
}>();

const store = useFileStore();
const sel = useSelectionStore();

// ── Thumbnail cache ──
const thumbCache = ref<Record<string, string>>({});
let pending = new Set<string>();
let timer: ReturnType<typeof setTimeout> | null = null;

function isImage(file: FileEntry): boolean {
    if (file.is_dir) return false;
    return getFileCategory(file.extension, false) === "image";
}

function thumbSrc(file: FileEntry): string {
    return thumbCache.value[file.path] || "";
}

function loadThumb(path: string) {
    if (pending.has(path) || thumbCache.value[path]) return;
    pending.add(path);
    if (timer) clearTimeout(timer);
    timer = setTimeout(async () => {
        const batch = [...pending];
        pending.clear();
        for (const p of batch) {
            try {
                const res = await tauri.getFileBase64(p);
                thumbCache.value = {
                    ...thumbCache.value,
                    [p]: `data:${res.mime};base64,${res.data}`,
                };
            } catch {
                thumbCache.value = { ...thumbCache.value, [p]: "__error__" };
            }
        }
    }, 50);
}

// Load thumbnails when files change
watch(
    () => props.files,
    (newFiles) => {
        for (const f of newFiles) {
            if (isImage(f)) loadThumb(f.path);
        }
    },
    { immediate: true },
);

function isSelected(path: string): boolean {
    return sel.selectedFiles.has(path);
}

function gridColorClass(file: FileEntry): string {
    return gridColorClassForCategory(
        getFileCategory(file.extension, file.is_dir),
    );
}

function richIcon(file: FileEntry): string | null {
    if (file.is_dir && !isBundleDirectory(file.extension, file.is_dir))
        return null;
    return getFileIconSvg(file.extension, false);
}

function isBundle(file: FileEntry): boolean {
    return isBundleDirectory(file.extension, file.is_dir);
}
</script>

<style scoped>
.grid-items {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 4px;
    padding: 8px;
    align-content: start;
    content-visibility: auto;
    contain-intrinsic-size: auto 500px;
}
.grid-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 6px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s;
    text-align: center;
    min-height: 0;
    content-visibility: auto;
    contain-intrinsic-size: auto 100px;
}
.grid-card:hover {
    background: var(--bg-hover);
}
.grid-card.selected {
    background: var(--bg-selected);
}
.grid-icon {
    width: 64px;
    height: 64px;
    margin-bottom: 6px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
}
.grid-icon svg {
    width: 100%;
    height: 100%;
    filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.15));
}
.grid-thumb {
    width: 56px;
    height: 56px;
    object-fit: cover;
    border-radius: 4px;
}
.grid-rich-icon {
    width: 48px;
    height: 48px;
}
.grid-rich-icon svg {
    width: 100%;
    height: 100%;
    display: block;
}
.grid-name {
    font-size: 12px;
    line-height: 1.3;
    word-break: break-word;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    max-width: 100%;
}

/* ── Grid view icon colors (Win11 style, theme-aware) ── */
.grid-icon {
    --folder-main: #f5c542;
    --folder-shade: #e8b825;
    --doc-main: #7890b0;
    --doc-shade: #5a7295;
    --doc-highlight: #9ab0cc;
}

[data-theme="dark"] .grid-color-code {
    --doc-main: #a6e3a1;
    --doc-shade: #7bc47a;
    --doc-highlight: #c8f0c4;
}
[data-theme="dark"] .grid-color-image {
    --doc-main: #cba6f7;
    --doc-shade: #b485e8;
    --doc-highlight: #dec0fa;
}
[data-theme="dark"] .grid-color-audio {
    --doc-main: #f9e2af;
    --doc-shade: #e8c77a;
    --doc-highlight: #fcf0d0;
}
[data-theme="dark"] .grid-color-video {
    --doc-main: #f38ba8;
    --doc-shade: #e46d8e;
    --doc-highlight: #f8b0c4;
}
[data-theme="dark"] .grid-color-archive {
    --doc-main: #f5c542;
    --doc-shade: #dba42e;
    --doc-highlight: #f9e2af;
}
[data-theme="dark"] .grid-color-pdf {
    --doc-main: #f38ba8;
    --doc-shade: #e46d8e;
    --doc-highlight: #f8b0c4;
}
[data-theme="dark"] .grid-color-app {
    --doc-main: #89b4fa;
    --doc-shade: #5f9cf0;
    --doc-highlight: #b8d4fc;
}
[data-theme="dark"] .grid-color-web {
    --doc-main: #fab387;
    --doc-shade: #e8955e;
    --doc-highlight: #fccca8;
}
[data-theme="dark"] .grid-color-default {
    --doc-main: #7890b0;
    --doc-shade: #5a7295;
    --doc-highlight: #9ab0cc;
}

[data-theme="light"] .grid-icon {
    --folder-main: #df8e1d;
    --folder-shade: #c47a15;
    --doc-main: #8c8fa0;
    --doc-shade: #70748c;
    --doc-highlight: #a8acc0;
}
[data-theme="light"] .grid-color-code {
    --doc-main: #40a02b;
    --doc-shade: #2e801e;
    --doc-highlight: #6cc05a;
}
[data-theme="light"] .grid-color-image {
    --doc-main: #8839ef;
    --doc-shade: #7020d5;
    --doc-highlight: #a868f8;
}
[data-theme="light"] .grid-color-audio {
    --doc-main: #df8e1d;
    --doc-shade: #c47a15;
    --doc-highlight: #f0b050;
}
[data-theme="light"] .grid-color-video {
    --doc-main: #d20f39;
    --doc-shade: #b0082a;
    --doc-highlight: #e84560;
}
[data-theme="light"] .grid-color-archive {
    --doc-main: #df8e1d;
    --doc-shade: #c47a15;
    --doc-highlight: #f0b050;
}
[data-theme="light"] .grid-color-pdf {
    --doc-main: #d20f39;
    --doc-shade: #b0082a;
    --doc-highlight: #e84560;
}
[data-theme="light"] .grid-color-app {
    --doc-main: #1e66f5;
    --doc-shade: #0d4fd8;
    --doc-highlight: #5090f8;
}
[data-theme="light"] .grid-color-web {
    --doc-main: #fe640b;
    --doc-shade: #d95208;
    --doc-highlight: #fe8a48;
}
[data-theme="light"] .grid-color-default {
    --doc-main: #8c8fa0;
    --doc-shade: #70748c;
    --doc-highlight: #a8acc0;
}
/* ── macOS folder blue ── */
[data-platform="macos"] .grid-icon {
    --folder-main: #5ea8f5;
    --folder-shade: #3b8de0;
}
[data-platform="macos"][data-theme="light"] .grid-icon {
    --folder-main: #3994f5;
    --folder-shade: #2578d8;
}
</style>
