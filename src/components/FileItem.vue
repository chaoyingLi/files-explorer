<template>
    <div
        class="file-item"
        :class="{ selected, cut: isCut, focused }"
        @contextmenu.prevent="$emit('contextmenu', $event)"
        @click="$emit('click', $event)"
        @dblclick="$emit('dblclick', $event)"
        @mousedown="startNativeFileDrag"
    >
        <div class="col-name">
            <div
                class="file-icon-wrap"
                :class="file.is_dir ? 'icon-folder' : ''"
            >
                <!-- Fluent UI file-type icon (also for bundle dirs like .app) -->
                <div
                    v-if="fileIconSvg && (!file.is_dir || isBundle)"
                    class="file-icon"
                    v-html="fileIconSvg"
                ></div>
                <!-- Folder icon (skip for bundles) -->
                <svg
                    v-else-if="file.is_dir && !isBundle"
                    class="file-icon"
                    viewBox="0 0 24 24"
                >
                    <path
                        d="M3 7.3c0-.94.76-1.7 1.7-1.7h4a1.7 1.7 0 011.36.64l1.28 1.6a.5.5 0 00.38.18h8.58c.94 0 1.7.76 1.7 1.7v6.7a2 2 0 01-2 2H5a2 2 0 01-2-2V7.3z"
                        fill="var(--folder-back)"
                    />
                    <path
                        d="M3 8.8c0-.94.76-1.7 1.7-1.7h4a1.7 1.7 0 011.36.64l1.28 1.6a.5.5 0 00.38.18h8.58c.94 0 1.7.76 1.7 1.7v6.2a2 2 0 01-2 2H5a2 2 0 01-2-2V8.8z"
                        fill="var(--file-icon-primary)"
                    />
                </svg>
                <!-- File icon (Win11 style, solid colors) -->
                <svg v-else class="file-icon" viewBox="0 0 24 24">
                    <path
                        d="M6.5 2.5h6.8l5.2 5.2V19.5a2 2 0 01-2 2H6.5a2 2 0 01-2-2V4.5a2 2 0 012-2z"
                        fill="var(--file-icon-primary)"
                    />
                    <path
                        d="M13.3 2.5v4.2c0 .55.45 1 1 1H18"
                        fill="var(--file-icon-secondary)"
                    />
                    <rect
                        x="7.5"
                        y="11"
                        width="9"
                        height="1.5"
                        rx="0.75"
                        fill="var(--file-icon-lines)"
                        opacity="0.4"
                    />
                    <rect
                        x="7.5"
                        y="14"
                        width="7"
                        height="1.5"
                        rx="0.75"
                        fill="var(--file-icon-lines)"
                        opacity="0.4"
                    />
                    <rect
                        x="7.5"
                        y="17"
                        width="5"
                        height="1.5"
                        rx="0.75"
                        fill="var(--file-icon-lines)"
                        opacity="0.4"
                    />
                </svg>
            </div>
            <span class="file-name">
                {{ file.name }}
                <span v-if="isCut" class="cut-badge">✂</span>
            </span>
            <span v-if="showPath && compact" class="file-path-sub">{{
                file.path
            }}</span>
        </div>
        <div v-if="showPath && !compact" class="col-path" :title="file.path">
            {{ file.path }}
        </div>
        <div class="col-date" v-if="!compact">
            {{ formatDate(file.modified) }}
        </div>
        <div class="col-created" v-if="!compact">
            {{ formatDate(file.created) }}
        </div>
        <div class="col-type">{{ fileType }}</div>
        <div class="col-size">{{ formatSize(file.size) }}</div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useTabStore } from "@/stores/tabStore";
import type { FileEntry } from "@/types";
import {
    getFileCategory,
    colorClassForCategory,
    formatFileSize,
    formatFileDate,
} from "@/utils/fileTypes";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";
import * as tauri from "@/utils/tauri";

const props = defineProps<{
    file: FileEntry;
    selected: boolean;
    focused?: boolean;
    compact?: boolean;
    isCut?: boolean;
    showPath?: boolean;
}>();

defineEmits<{
    click: [e: MouseEvent];
    dblclick: [e: MouseEvent];
    contextmenu: [e: MouseEvent];
}>();

const { t } = useI18n();
const sel = useSelectionStore();

const category = computed(() =>
    getFileCategory(props.file.extension, props.file.is_dir),
);

const colorClass = computed(() => colorClassForCategory(category.value));

const isBundle = computed(() =>
    isBundleDirectory(props.file.extension, props.file.is_dir),
);

const fileIconSvg = computed(() =>
    getFileIconSvg(props.file.extension, props.file.is_dir),
);

const fileType = computed(() => {
    if (props.file.is_dir) return t("fileTypes.folder");
    const ext = props.file.extension.toLowerCase();
    const m: Record<string, string> = {
        txt: t("fileTypes.textDocument"),
        md: t("fileTypes.markdown"),
        js: t("fileTypes.javascript"),
        ts: t("fileTypes.typescript"),
        vue: t("fileTypes.vueComponent"),
        json: t("fileTypes.jsonFile"),
        html: t("fileTypes.htmlFile"),
        css: t("fileTypes.cssFile"),
        scss: t("fileTypes.scssFile"),
        py: t("fileTypes.pythonFile"),
        rs: t("fileTypes.rustSource"),
        go: t("fileTypes.goSource"),
        java: t("fileTypes.javaSource"),
        cpp: t("fileTypes.cppSource"),
        c: t("fileTypes.cSource"),
        h: t("fileTypes.cHeader"),
        exe: t("fileTypes.application"),
        dll: t("fileTypes.library"),
        png: t("fileTypes.pngImage"),
        jpg: t("fileTypes.jpegImage"),
        jpeg: t("fileTypes.jpegImage"),
        gif: t("fileTypes.gifImage"),
        svg: t("fileTypes.svgImage"),
        webp: t("fileTypes.webpImage"),
        mp3: t("fileTypes.mp3Audio"),
        wav: t("fileTypes.wavAudio"),
        mp4: t("fileTypes.mp4Video"),
        pdf: t("fileTypes.pdfDocument"),
        zip: t("fileTypes.zipArchive"),
        rar: t("fileTypes.rarArchive"),
        "7z": t("fileTypes.sevenZArchive"),
        tar: t("fileTypes.tarArchive"),
        gz: t("fileTypes.gzArchive"),
        toml: t("fileTypes.tomlFile"),
        yaml: t("fileTypes.yamlFile"),
        yml: t("fileTypes.yamlFile"),
        xml: t("fileTypes.xmlFile"),
        lock: t("fileTypes.lockFile"),
        gitignore: t("fileTypes.gitIgnore"),
        env: t("fileTypes.envFile"),
    };
    return (
        m[ext] ||
        (ext
            ? `${ext.toUpperCase()} ${t("fileTypes.file")}`
            : t("fileTypes.file"))
    );
});

// ── Native File Drag (COM DoDragDrop via Rust) ──
function startNativeFileDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    console.log("drag:start", props.file.name);
    const store = useFileStore();
    const selected = sel.selectedFiles;
    const paths = selected.has(props.file.path)
        ? [...selected]
        : [props.file.path];
    const sx = e.clientX,
        sy = e.clientY;
    let started = false;

    function onMove(ev: MouseEvent) {
        if (started) return;
        const dx = ev.clientX - sx,
            dy = ev.clientY - sy;
        if (dx * dx + dy * dy <= 9) return; // 3px threshold
        started = true;
        console.log("drag:launch", paths.length, "files");
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
        tauri
            .startNativeDrag(paths)
            .catch((err: any) => console.error("drag:err", err));
    }
    function onUp() {
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
}

function formatSize(bytes: number): string {
    return formatFileSize(bytes);
}
function formatDate(ts: number): string {
    return formatFileDate(ts);
}
</script>

<style scoped>
.file-item {
    display: flex;
    align-items: center;
    padding: 3px 12px;
    cursor: pointer;
    font-size: var(--font-size-base);
    transition: background 0.05s;
    min-height: 34px;
    border-radius: 4px;
    margin: 0 4px;
}

.file-item:hover {
    background: var(--bg-hover);
}
.file-item.selected {
    background: var(--bg-selected);
}
.file-item.focused:not(.selected) {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
}

.col-name {
    width: var(--col-name, 280px);
    min-width: 120px;
    display: flex;
    align-items: center;
    gap: 10px;
}

.col-date {
    width: var(--col-date, 140px);
    min-width: 70px;
    flex-shrink: 0;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
}
.col-created {
    width: var(--col-created, 140px);
    min-width: 70px;
    flex-shrink: 0;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
}
.col-type {
    width: var(--col-type, 100px);
    min-width: 60px;
    flex-shrink: 0;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
}
.col-size {
    width: var(--col-size, 90px);
    min-width: 60px;
    flex-shrink: 0;
    text-align: right;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
}

.file-icon-wrap {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
}

.file-icon {
    width: 22px;
    height: 22px;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.15));
    transition: transform 0.1s;
}
.file-icon :deep(svg) {
    width: 100%;
    height: 100%;
}

.file-item:hover .file-icon {
    transform: scale(1.05);
}

.file-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 4px;
}

.file-path-sub {
    font-size: var(--font-size-sm, 11px);
    color: var(--text-muted);
    margin-left: 6px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 1;
    min-width: 0;
}

.col-path {
    width: 260px;
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: var(--font-size-sm, 11px);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 8px;
}

.cut-badge {
    font-size: var(--font-size-sm);
    opacity: 0.7;
}

/* ── Cut visual state ── */
.file-item.cut {
    opacity: 0.45;
    pointer-events: none;
}
</style>

<style>
/* ── Root icon color variables (folder = warm Win11 yellow) ── */
:root,
[data-theme="dark"] {
    --file-icon-primary: #f5c542;
    --file-icon-secondary: #e8b825;
    --folder-back: #e8b825;
    --file-icon-lines: #585b70;
}

[data-theme="light"] {
    --file-icon-primary: #df8e1d;
    --file-icon-secondary: #c47a15;
    --folder-back: #c47a15;
    --file-icon-lines: #9ca0b0;
}

/* ── macOS folder blue (SF Symbols style) ── */
[data-platform="macos"][data-theme="dark"] {
    --file-icon-primary: #5ea8f5;
    --file-icon-secondary: #3b8de0;
    --folder-back: #3b8de0;
}
[data-platform="macos"][data-theme="light"] {
    --file-icon-primary: #3994f5;
    --file-icon-secondary: #2578d8;
    --folder-back: #2578d8;
}
</style>

<style scoped>
/* ── Category colors for dark theme ── */
[data-theme="dark"] .color-code {
    --file-icon-primary: #a6e3a1;
    --file-icon-secondary: #7bc47a;
}
[data-theme="dark"] .color-image {
    --file-icon-primary: #cba6f7;
    --file-icon-secondary: #b485e8;
}
[data-theme="dark"] .color-audio {
    --file-icon-primary: #f9e2af;
    --file-icon-secondary: #e8c77a;
}
[data-theme="dark"] .color-video {
    --file-icon-primary: #f38ba8;
    --file-icon-secondary: #e46d8e;
}
[data-theme="dark"] .color-archive {
    --file-icon-primary: #f5c542;
    --file-icon-secondary: #dba42e;
}
[data-theme="dark"] .color-pdf {
    --file-icon-primary: #f38ba8;
    --file-icon-secondary: #e46d8e;
}
[data-theme="dark"] .color-app {
    --file-icon-primary: #89b4fa;
    --file-icon-secondary: #5f9cf0;
}
[data-theme="dark"] .color-web {
    --file-icon-primary: #fab387;
    --file-icon-secondary: #e8955e;
}
[data-theme="dark"] .color-default {
    --file-icon-primary: #7890b0;
    --file-icon-secondary: #5a7295;
}

/* ── Category colors for light theme ── */
[data-theme="light"] .color-code {
    --file-icon-primary: #40a02b;
    --file-icon-secondary: #2e801e;
}
[data-theme="light"] .color-image {
    --file-icon-primary: #8839ef;
    --file-icon-secondary: #7020d5;
}
[data-theme="light"] .color-audio {
    --file-icon-primary: #df8e1d;
    --file-icon-secondary: #c47a15;
}
[data-theme="light"] .color-video {
    --file-icon-primary: #d20f39;
    --file-icon-secondary: #b0082a;
}
[data-theme="light"] .color-archive {
    --file-icon-primary: #df8e1d;
    --file-icon-secondary: #c47a15;
}
[data-theme="light"] .color-pdf {
    --file-icon-primary: #d20f39;
    --file-icon-secondary: #b0082a;
}
[data-theme="light"] .color-app {
    --file-icon-primary: #1e66f5;
    --file-icon-secondary: #0d4fd8;
}
[data-theme="light"] .color-web {
    --file-icon-primary: #fe640b;
    --file-icon-secondary: #d95208;
}
[data-theme="light"] .color-default {
    --file-icon-primary: #8c8fa0;
    --file-icon-secondary: #70748c;
}
</style>
