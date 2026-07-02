<template>
    <div class="preview-panel" v-if="visible" :style="{ width: width + 'px' }">
        <div class="preview-resize-handle" @mousedown.stop="onResizeStart" />
        <div class="preview-header">
            <span>{{ $t("properties.title") }}</span>
            <div class="preview-header-actions">
                <button
                    v-if="file"
                    class="preview-popout"
                    :title="$t('properties.popOut')"
                    @click="openPreviewWindow"
                >
                    ↗
                </button>
                <button class="preview-close" @click="$emit('close')">✕</button>
            </div>
        </div>

        <!-- Empty / multi-select state -->
        <div v-if="!file && multiCount <= 1" class="preview-empty">
            <span>{{ $t("properties.noFileSelected") }}</span>
        </div>
        <div v-else-if="!file && multiCount > 1" class="preview-multi">
            <span class="preview-multi-text">{{
                $t("properties.itemsSelected", { count: multiCount })
            }}</span>
            <span class="preview-meta"
                >{{ $t("properties.totalSize") }}:
                {{ formatSize(totalSize) }}</span
            >
        </div>

        <template v-else-if="file">
            <!-- ════ Preview area (top, fills available space) ════ -->
            <div
                class="preview-area"
                @contextmenu.stop.prevent="onPanelCtxMenu"
            >
                <!-- Loading -->
                <div v-if="previewLoading" class="preview-status">
                    <div class="preview-skeleton">
                        <div class="preview-skeleton-line"></div>
                        <div class="preview-skeleton-line"></div>
                        <div class="preview-skeleton-line"></div>
                        <div class="preview-skeleton-line"></div>
                        <div class="preview-skeleton-block"></div>
                    </div>
                </div>
                <!-- Error -->
                <div
                    v-else-if="previewError"
                    class="preview-status preview-status-error"
                >
                    <span>{{ previewError }}</span>
                    <button class="preview-btn" @click="openFileExternally">
                        {{ $t("properties.openExternally") }}
                    </button>
                </div>
                <!-- Image with zoom -->
                <div
                    v-else-if="previewType === 'image'"
                    class="preview-zoom-wrap"
                >
                    <div class="preview-zoom-bar">
                        <button
                            class="preview-zoom-btn"
                            :disabled="imageZoom <= 0.25"
                            :title="$t('properties.zoomOut')"
                            @click.stop="
                                imageZoom = Math.max(0.25, imageZoom - 0.1)
                            "
                        >
                            −
                        </button>
                        <span class="preview-zoom-pct"
                            >{{ Math.round(imageZoom * 100) }}%</span
                        >
                        <button
                            class="preview-zoom-btn"
                            :disabled="imageZoom >= 5"
                            :title="$t('properties.zoomIn')"
                            @click.stop="
                                imageZoom = Math.min(5, imageZoom + 0.1)
                            "
                        >
                            +
                        </button>
                        <button
                            class="preview-zoom-btn"
                            :title="$t('properties.zoomReset')"
                            @click.stop="
                                imageZoom = 1;
                                imageRotation = 0;
                            "
                        >
                            ⊡
                        </button>
                        <div class="preview-zoom-sep"></div>
                        <button
                            class="preview-zoom-btn"
                            :title="$t('properties.rotateLeft')"
                            @click.stop="
                                imageRotation = (imageRotation + 270) % 360
                            "
                        >
                            ↺
                        </button>
                        <button
                            class="preview-zoom-btn"
                            :title="$t('properties.rotateRight')"
                            @click.stop="
                                imageRotation = (imageRotation + 90) % 360
                            "
                        >
                            ↻
                        </button>
                    </div>
                    <div
                        class="preview-zoom-scroll"
                        @mousedown="
                            (e) =>
                                onDragStart(e, e.currentTarget as HTMLElement)
                        "
                        @mousemove="
                            (e) => onDragMove(e, e.currentTarget as HTMLElement)
                        "
                        @mouseup="
                            (e) => onDragEnd(e.currentTarget as HTMLElement)
                        "
                        @mouseleave="
                            (e) => onDragEnd(e.currentTarget as HTMLElement)
                        "
                    >
                        <img
                            class="preview-image"
                            :src="previewSrc"
                            alt=""
                            :style="{
                                transform:
                                    'scale(' +
                                    imageZoom +
                                    ') rotate(' +
                                    imageRotation +
                                    'deg)',
                            }"
                            @click.stop
                            @wheel.prevent="
                                imageZoom = Math.max(
                                    0.25,
                                    Math.min(
                                        5,
                                        imageZoom - $event.deltaY * 0.001,
                                    ),
                                )
                            "
                            draggable="false"
                        />
                    </div>
                </div>
                <!-- DOCX -->
                <div v-else-if="previewType === 'docx'" class="preview-office">
                    <VueOfficeDocx
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <!-- XLSX -->
                <div v-else-if="previewType === 'xlsx'" class="preview-office">
                    <VueOfficeExcel
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <!-- XLS (legacy with SheetJS) -->
                <div v-else-if="previewType === 'xls'" class="preview-office">
                    <XlsPreview
                        v-if="officeArrayBuffer"
                        :data="officeArrayBuffer"
                    />
                </div>
                <!-- Video -->
                <div v-else-if="previewType === 'video'" class="preview-office">
                    <VideoPreview
                        ref="videoRef"
                        :src="previewSrc"
                        :minimalist="true"
                    />
                </div>
                <!-- PDF with zoom -->
                <div
                    v-else-if="previewType === 'pdf'"
                    class="preview-zoom-wrap"
                >
                    <div class="preview-zoom-bar">
                        <button
                            class="preview-zoom-btn"
                            :disabled="pdfZoom <= 0.5"
                            :title="$t('properties.zoomOut')"
                            @click.stop="pdfZoom = Math.max(0.5, pdfZoom - 0.2)"
                        >
                            −
                        </button>
                        <span class="preview-zoom-pct"
                            >{{ Math.round(pdfZoom * 100) }}%</span
                        >
                        <button
                            class="preview-zoom-btn"
                            :disabled="pdfZoom >= 3"
                            :title="$t('properties.zoomIn')"
                            @click.stop="pdfZoom = Math.min(3, pdfZoom + 0.2)"
                        >
                            +
                        </button>
                        <button
                            class="preview-zoom-btn"
                            :title="$t('properties.zoomReset')"
                            @click.stop="pdfZoom = 1"
                        >
                            ⊡
                        </button>
                        <div class="preview-zoom-sep"></div>
                    </div>
                    <div
                        class="preview-zoom-scroll"
                        @mousedown="
                            (e) =>
                                onDragStart(e, e.currentTarget as HTMLElement)
                        "
                        @mousemove="
                            (e) => onDragMove(e, e.currentTarget as HTMLElement)
                        "
                        @mouseup="
                            (e) => onDragEnd(e.currentTarget as HTMLElement)
                        "
                        @mouseleave="
                            (e) => onDragEnd(e.currentTarget as HTMLElement)
                        "
                    >
                        <div
                            class="preview-office"
                            :style="{
                                transform: 'scale(' + pdfZoom + ')',
                                transformOrigin: 'top left',
                            }"
                        >
                            <VueOfficePdf
                                v-if="officeData"
                                :src="officeData"
                                style="height: 100%; width: 100%"
                            />
                        </div>
                    </div>
                </div>
                <!-- PPTX -->
                <div v-else-if="previewType === 'pptx'" class="preview-office">
                    <PptxPreview
                        v-if="officeArrayBuffer"
                        :data="officeArrayBuffer"
                    />
                </div>
                <!-- Archive listing -->
                <div
                    v-else-if="previewType === 'archive'"
                    class="preview-archive"
                >
                    <div class="preview-archive-header">
                        {{ archiveTotal }} entries
                    </div>
                    <div class="preview-archive-list">
                        <div
                            v-for="entry in archiveEntries"
                            :key="entry.path"
                            class="preview-archive-item"
                            :class="{
                                'preview-archive-item--dir': entry.is_dir,
                            }"
                            :style="{
                                paddingLeft:
                                    8 +
                                    entry.path.split('/').length * 14 +
                                    'px',
                            }"
                        >
                            <span class="preview-archive-icon">{{
                                entry.is_dir ? "📁" : "📄"
                            }}</span>
                            <span class="preview-archive-name">{{
                                entry.name
                            }}</span>
                            <span
                                v-if="!entry.is_dir"
                                class="preview-archive-size"
                                >{{ formatSize(entry.size) }}</span
                            >
                        </div>
                    </div>
                </div>

                <!-- Code / Text -->
                <div v-else-if="previewType === 'text'" class="preview-code">
                    <CodePreview :code="previewContent" :ext="previewExt" />
                </div>
                <!-- Markdown -->
                <div
                    v-else-if="previewType === 'markdown'"
                    class="preview-markdown"
                >
                    <MarkdownPreview
                        :content="previewContent"
                        :ext="previewExt"
                        :filePath="file?.path"
                    />
                </div>
                <!-- External only (no preview available) -->
                <div
                    v-else-if="previewType === 'externalOnly'"
                    class="preview-unsupported"
                >
                    <div class="preview-unsupported-icon">📄</div>
                    <span class="preview-unsupported-text">{{
                        $t("properties.noPreview")
                    }}</span>
                    <button class="preview-btn" @click="openFileExternally">
                        {{ $t("properties.openExternally") }}
                    </button>
                </div>
            </div>

            <!-- ════ File info (bottom, compact) ════ -->
            <div
                class="preview-info"
                @contextmenu.stop.prevent="onPanelCtxMenu"
            >
                <div class="preview-info-header">
                    <div class="preview-info-icon" v-html="fileIconSvg"></div>
                    <span class="preview-info-name">{{ file.name }}</span>
                </div>

                <table class="props-table">
                    <tr>
                        <td class="props-td-label">
                            {{ $t("fileList.type") }}
                        </td>
                        <td class="props-td-value">{{ fileType }}</td>
                    </tr>
                    <tr>
                        <td class="props-td-label">
                            {{ $t("fileList.size") }}
                        </td>
                        <td class="props-td-value">
                            {{ formatSize(file.size) }}
                        </td>
                    </tr>
                    <tr v-if="imageInfo">
                        <td class="props-td-label">
                            {{ $t("properties.dimensions") }}
                        </td>
                        <td class="props-td-value">
                            {{ imageInfo.width }} × {{ imageInfo.height }}
                        </td>
                    </tr>
                    <tr v-else-if="file.is_dir">
                        <td class="props-td-label">
                            {{ $t("properties.contents") }}
                        </td>
                        <td class="props-td-value">
                            {{
                                $t("properties.itemsCount", {
                                    count: dirItemCount,
                                })
                            }}
                        </td>
                    </tr>
                    <tr>
                        <td class="props-td-label">
                            {{ $t("fileList.dateModified") }}
                        </td>
                        <td class="props-td-value">
                            {{ formatDate(file.modified) }}
                        </td>
                    </tr>
                    <tr>
                        <td class="props-td-label">
                            {{ $t("fileList.dateCreated") }}
                        </td>
                        <td class="props-td-value">
                            {{ formatDate(file.created) }}
                        </td>
                    </tr>
                    <tr>
                        <td class="props-td-label">
                            {{ $t("properties.fullPath") }}
                        </td>
                        <td
                            class="props-td-value props-td-path"
                            :title="$t('contextMenu.showInFinder')"
                            @click.stop="showInExplorerClick"
                        >
                            {{ file.path }}
                        </td>
                    </tr>
                </table>

                <div class="preview-info-actions">
                    <button class="preview-btn" @click="openFileExternally">
                        {{ $t("properties.openExternally") }}
                    </button>
                </div>
            </div>
            <!-- Context menu -->
            <teleport to="body">
                <div
                    v-if="panelCtx.show"
                    class="panel-ctx-overlay"
                    @click="panelCtx.show = false"
                    @contextmenu.prevent="panelCtx.show = false"
                >
                    <div
                        class="panel-ctx-menu"
                        :style="{
                            left: panelCtx.x + 'px',
                            top: panelCtx.y + 'px',
                        }"
                    >
                        <button
                            class="panel-ctx-item"
                            @click="ctxAction('open')"
                        >
                            <span v-html="panelIcons.open"></span
                            >{{ $t("contextMenu.open") }}
                        </button>
                        <button
                            class="panel-ctx-item"
                            @click="ctxAction('showInExplorer')"
                        >
                            <span v-html="panelIcons.showInExplorer"></span
                            >{{ $t("contextMenu.showInFinder") }}
                        </button>
                        <button
                            class="panel-ctx-item"
                            @click="ctxAction('openInTerminal')"
                        >
                            <span v-html="panelIcons.terminal"></span
                            >{{ $t("contextMenu.openInTerminal") }}
                        </button>
                        <div class="panel-ctx-sep"></div>
                        <button
                            class="panel-ctx-item"
                            @click="ctxAction('copyPath')"
                        >
                            <span v-html="panelIcons.copy"></span
                            >{{ $t("contextMenu.copyPath") }}
                        </button>
                        <button
                            class="panel-ctx-item"
                            @click="ctxAction('previewWindow')"
                        >
                            <span v-html="panelIcons.popout"></span
                            >{{ $t("contextMenu.openInPreviewWindow") }}
                        </button>
                    </div>
                </div>
            </teleport>
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, reactive } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";

// Translate common Rust backend error messages to localized strings
function translatePreviewError(
    err: string,
    t: (key: string) => string,
): string {
    const known: Record<string, string> = {
        "Binary file": t("properties.binaryFile"),
        "File too large for preview": t("properties.fileTooLarge"),
        "Preview unavailable": t("properties.previewUnavailable"),
        "Extract failed": t("properties.extractFailed"),
    };
    // Direct match
    if (known[err]) return known[err];
    // Partial match for errors like "Failed to stat: ..."
    if (err.startsWith("Failed to stat")) return t("properties.failedToStat");
    if (err.startsWith("Read failed")) return t("properties.readFailed");
    if (err.startsWith("File too large")) return t("properties.fileTooLarge");
    return err;
}
import { useSelectionStore } from "@/stores/selectionStore";
import { useViewStore } from "@/stores/viewStore";
import { useTabStore } from "@/stores/tabStore";
import type { FileEntry } from "@/types";
import type { ArchiveEntry } from "@/utils/tauri";
import { formatFileSize } from "@/utils/fileTypes";
import {
    getFileIcon,
    listDirectory,
    getFilePreview,
    readFileBytes,
    listArchiveContents,
    openFile,
    showInExplorer,
    openInTerminal,
    getAdaptivePreviewSize,
} from "@/utils/tauri";
import { getFileIconSvg } from "@/utils/fileIcons";

import VueOfficeDocx from "@vue-office/docx";
import VueOfficeExcel from "@vue-office/excel";
import VueOfficePdf from "@vue-office/pdf";
import PptxPreview from "@/components/PptxPreview.vue";
import CodePreview from "@/components/CodePreview.vue";
import MarkdownPreview from "@/components/MarkdownPreview.vue";
import XlsPreview from "@/components/XlsPreview.vue";
import VideoPreview from "@/components/VideoPreview.vue";
import "@vue-office/docx/lib/index.css";
import "@vue-office/excel/lib/index.css";
import { convertFileSrc } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const IMG_EXTS = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"];
const VIDEO_EXTS = ["mp4", "webm", "ogg", "mov", "flv", "mkv", "avi", "wmv"];
const ARCHIVE_EXTS = [
    "zip",
    "7z",
    "rar",
    "tar",
    "gz",
    "tgz",
    "bz2",
    "tbz2",
    "xz",
    "txz",
];

const { t } = useI18n();
const store = useFileStore();
const sel = useSelectionStore();
const view = useViewStore();
const tabStore = useTabStore();

defineProps<{ visible: boolean; width: number }>();
const emit = defineEmits<{ close: []; resizeStart: [e: MouseEvent] }>();

const imageInfo = ref<{ width: number; height: number } | null>(null);
const imageZoom = ref(1);
const imageRotation = ref(0);
const pdfZoom = ref(1);
const videoRef = ref<any>(null);

// ── File icon (matches file list) ──
const FOLDER_ICON = `<svg viewBox="0 0 32 32" fill="none"><path d="M4 6.5A1.5 1.5 0 015.5 5h6.8l2.4 3H26.5A1.5 1.5 0 0128 9.5v16a1.5 1.5 0 01-1.5 1.5H5.5A1.5 1.5 0 014 25.5V6.5z" fill="#F6C23A"/><path d="M5.5 5h6.8l2.4 3" fill="#F9D56E"/></svg>`;
const fileIconSvg = computed(() => {
    const svg = getFileIconSvg(
        file.value?.extension || "",
        file.value?.is_dir || false,
    );
    if (svg) return svg;
    if (file.value?.is_dir) return FOLDER_ICON;
    return null;
});

// ── Panel context menu ──
const panelCtx = reactive({ show: false, x: 0, y: 0 });
const panelIcons = {
    open: `<svg viewBox="0 0 14 14" width="12" height="12"><path d="M2 4.5a1 1 0 011-1h2.5l1.2 1.5H11a1 1 0 011 1V11a1 1 0 01-1 1H3a1 1 0 01-1-1V4.5z" fill="none" stroke="currentColor" stroke-width="1"/></svg>`,
    showInExplorer: `<svg viewBox="0 0 14 14" width="12" height="12"><path d="M8 2h4v4M6 8l6-6M4 3H3a1 1 0 00-1 1v7a1 1 0 001 1h7a1 1 0 001-1v-1" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    terminal: `<svg viewBox="0 0 14 14" width="12" height="12"><rect x="1.5" y="2.5" width="11" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1"/><path d="M4 5l2 2-2 2M7 9h3" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    copy: `<svg viewBox="0 0 14 14" width="12" height="12"><rect x="3.5" y="1.5" width="7" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1"/><rect x="1.5" y="3.5" width="7" height="9" rx="1" fill="var(--bg-secondary)" stroke="currentColor" stroke-width="1"/></svg>`,
    popout: `<svg viewBox="0 0 14 14" width="12" height="12"><path d="M8 2h4v4M6 8l6-6M4 3H3a1 1 0 00-1 1v7a1 1 0 001 1h7a1 1 0 001-1v-1" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
};

function onPanelCtxMenu(e: MouseEvent) {
    e.stopPropagation();
    if (!file.value) return;
    panelCtx.x = e.clientX;
    panelCtx.y = e.clientY;
    panelCtx.show = true;
}

async function ctxAction(action: string) {
    const p = file.value?.path;
    if (!p) return;
    panelCtx.show = false;
    switch (action) {
        case "open":
            await openFile(p).catch(() => {});
            break;
        case "showInExplorer":
            await showInExplorer(p).catch(() => {});
            break;
        case "openInTerminal":
            await openInTerminal(p).catch(() => {});
            break;
        case "copyPath":
            await navigator.clipboard.writeText(p).catch(() => {});
            break;
        case "previewWindow":
            openPreviewWindow();
            break;
    }
}

function showInExplorerClick() {
    if (file.value) showInExplorer(file.value.path).catch(() => {});
}

// ── Drag-to-pan state (shared) ──
let _dragActive = false;
let _dragStartX = 0;
let _dragStartY = 0;
let _dragScrollLeft = 0;
let _dragScrollTop = 0;

function onDragStart(e: MouseEvent, el: HTMLElement | null) {
    if (!el) return;
    _dragActive = true;
    _dragStartX = e.clientX;
    _dragStartY = e.clientY;
    _dragScrollLeft = el.scrollLeft;
    _dragScrollTop = el.scrollTop;
    el.style.cursor = "grabbing";
    el.style.userSelect = "none";
}
function onDragMove(e: MouseEvent, el: HTMLElement | null) {
    if (!_dragActive || !el) return;
    el.scrollLeft = _dragScrollLeft - (e.clientX - _dragStartX);
    el.scrollTop = _dragScrollTop - (e.clientY - _dragStartY);
}
function onDragEnd(el: HTMLElement | null) {
    if (!_dragActive) return;
    _dragActive = false;
    if (el) {
        el.style.cursor = "";
        el.style.userSelect = "";
    }
}
const dirItemCount = ref(0);
const osIconSrc = ref("");
const previewType = ref<string>("");
const previewSrc = ref<string>("");
const previewContent = ref<string>("");
const previewExt = ref<string>("");

const previewLoading = ref(false);
const previewError = ref("");
const officeData = ref<ArrayBuffer | string | null>(null);
const officeArrayBuffer = computed(() =>
    officeData.value instanceof ArrayBuffer ? officeData.value : null,
);
const archiveEntries = ref<ArchiveEntry[]>([]);
const archiveTotal = ref(0);

const OFFICE_EXTS: Record<string, string> = {
    docx: "docx",
    xlsx: "xlsx",
    pptx: "pptx",
    // WPS formats (OOXML-compatible)
    wps: "docx",
    et: "xlsx",
    dps: "pptx",
    // Legacy formats
    doc: "externalOnly",
    xls: "xls",
    ppt: "externalOnly",
    // National standard e-invoice
    ofd: "externalOnly",
};

function findFileByPath(path: string): FileEntry | null {
    const f = store.files.find((x) => x.path === path);
    if (f) return f;
    if (view.viewMode === "column") {
        const tab = tabStore.getFocusedTab();
        if (tab?.columnStack) {
            for (const col of tab.columnStack) {
                const found = col.files.find((x) => x.path === path);
                if (found) return found;
            }
        }
    }
    if (view.viewMode === "tree") {
        for (const children of view.treeChildrenCache.values()) {
            const found = children.find((x: FileEntry) => x.path === path);
            if (found) return found;
        }
    }
    return null;
}

const file = computed<FileEntry | null>(() => {
    if (sel.selectedFiles.size !== 1) return null;
    const path = [...sel.selectedFiles][0];
    return findFileByPath(path);
});

const multiCount = computed(() => sel.selectedFiles.size);

const totalSize = computed(() => {
    let total = 0;
    for (const p of sel.selectedFiles) {
        const f = findFileByPath(p);
        if (f) total += f.size;
    }
    return total;
});

watch(file, async (f) => {
    imageInfo.value = null;
    dirItemCount.value = 0;
    osIconSrc.value = "";
    previewType.value = "";
    previewSrc.value = "";
    previewContent.value = "";
    previewExt.value = "";
    previewLoading.value = false;
    previewError.value = "";
    officeData.value = null;
    archiveEntries.value = [];
    archiveTotal.value = 0;
    if (!f) return;

    // ── Load OS file icon ──
    try {
        const b64 = await getFileIcon(f.path);
        osIconSrc.value = "data:image/png;base64," + b64;
    } catch {
        /* OS icon not available */
    }

    // ── Image: use native asset protocol (no base64, no size limit) ──
    if (IMG_EXTS.includes(f.extension.toLowerCase()) && !f.is_dir) {
        const assetUrl = convertFileSrc(f.path);
        // Get image dimensions from browser-native loading
        const img = new Image();
        img.onload = () => {
            imageInfo.value = {
                width: img.naturalWidth,
                height: img.naturalHeight,
            };
        };
        img.onerror = () => {
            imageInfo.value = null;
        };
        img.src = assetUrl;
    }

    // ── Directory item count ──
    if (f.is_dir) {
        try {
            const items = await listDirectory(f.path);
            dirItemCount.value = items.length;
        } catch {
            /* directory read failed */
        }
    }

    // ── Load file preview ──
    const ext = f.extension.toLowerCase();

    if (IMG_EXTS.includes(ext) && !f.is_dir) {
        // Image preview — use native asset protocol, no size limit
        previewType.value = "image";
        previewSrc.value = convertFileSrc(f.path);
    } else if (VIDEO_EXTS.includes(ext) && !f.is_dir) {
        const nativeExts = ["mp4", "webm", "ogg", "mov", "flv"];
        if (nativeExts.includes(ext)) {
            previewType.value = "video";
            previewSrc.value = convertFileSrc(f.path);
        } else {
            previewType.value = "externalOnly";
        }
    } else if (OFFICE_EXTS[ext] && !f.is_dir) {
        // Office documents: docx / xlsx / pptx — read raw bytes
        previewLoading.value = true;
        try {
            const buf = await loadFileAsArrayBuffer(f.path);
            officeData.value = buf;
            previewType.value = OFFICE_EXTS[ext];
            previewLoading.value = false;
        } catch (e: any) {
            previewError.value = translatePreviewError(
                String(e || "Preview unavailable"),
                t,
            );
            previewLoading.value = false;
        }
    } else if (ext === "pdf" && !f.is_dir) {
        // PDF — use @vue-office/pdf (read raw bytes)
        previewLoading.value = true;
        const pdfTimeout = setTimeout(() => {
            if (previewLoading.value) {
                previewError.value = t("properties.pdfLoadTimeout");
                previewLoading.value = false;
            }
        }, 15000);
        try {
            const b64 = await readFileBytes(f.path);
            clearTimeout(pdfTimeout);
            officeData.value = `data:application/pdf;base64,${b64}`;
            previewType.value = "pdf";
            previewLoading.value = false;
        } catch (e: any) {
            clearTimeout(pdfTimeout);
            previewError.value = translatePreviewError(
                String(e || "Preview unavailable"),
                t,
            );
            previewLoading.value = false;
        }
    } else if (!f.is_dir && ARCHIVE_EXTS.includes(ext)) {
        // Archive preview: list contents
        previewLoading.value = true;
        try {
            const entries = await listArchiveContents(f.path);
            archiveEntries.value = entries;
            archiveTotal.value = entries.length;
            previewType.value = "archive";
        } catch (e: any) {
            previewError.value = translatePreviewError(
                String(e || "Preview unavailable"),
                t,
            );
        } finally {
            previewLoading.value = false;
        }
    } else if (OFFICE_EXTS[ext] === "externalOnly" && !f.is_dir) {
        // Legacy / unsupported Office formats — show external-only placeholder
        previewType.value = "externalOnly";
        previewError.value = "";
    } else if (!f.is_dir) {
        // Text / Markdown preview via Rust
        previewLoading.value = true;
        try {
            const result = await getFilePreview(f.path);
            if (result.type === "markdown") {
                previewType.value = "markdown";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || f.extension;
            } else if (result.type === "text") {
                previewType.value = "text";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || f.extension;
            }
        } catch (e: any) {
            previewError.value = translatePreviewError(
                String(e || "Preview unavailable"),
                t,
            );
        } finally {
            previewLoading.value = false;
        }
    }
});

/**
 * Read a file as ArrayBuffer via Rust backend, with chunked base64 decode
 * to avoid blocking the main thread on large files.
 */
async function loadFileAsArrayBuffer(path: string): Promise<ArrayBuffer> {
    const b64 = await readFileBytes(path);
    // Decode base64 in chunks to yield to the main thread every 1MB
    const CHUNK = 1024 * 1024; // 1MB of base64 chars per chunk
    const binaryChunks: string[] = [];

    for (let i = 0; i < b64.length; i += CHUNK) {
        binaryChunks.push(atob(b64.slice(i, i + CHUNK)));
        // Yield to the main thread to keep UI responsive
        await new Promise((r) => setTimeout(r, 0));
    }

    const binary = binaryChunks.join("");
    const buf = new ArrayBuffer(binary.length);
    const view = new Uint8Array(buf);
    // Fill in chunks to avoid long synchronous loop
    const FILL_CHUNK = 256 * 1024; // 256KB per chunk
    for (let i = 0; i < binary.length; i += FILL_CHUNK) {
        const end = Math.min(i + FILL_CHUNK, binary.length);
        for (let j = i; j < end; j++) {
            view[j] = binary.charCodeAt(j);
        }
        if (i + FILL_CHUNK < binary.length) {
            await new Promise((r) => setTimeout(r, 0));
        }
    }
    return buf;
}

const defaultFileIcon = `<svg viewBox="0 0 24 24"><path d="M6.5 2.5h6.8l5.2 5.2V19.5a2 2 0 01-2 2H6.5a2 2 0 01-2-2V4.5a2 2 0 012-2z" fill="currentColor" opacity="0.3"/><path d="M13.3 2.5v4.2c0 .55.45 1 1 1H18" fill="currentColor" opacity="0.5"/></svg>`;

const fileType = computed(() => {
    if (!file.value) return "";
    if (file.value.is_dir) return t("fileTypes.folder");
    const ext = file.value.extension.toUpperCase();
    return ext ? t("properties.fileType", { ext }) : t("fileTypes.file");
});

function formatSize(bytes: number): string {
    return formatFileSize(bytes);
}

function formatDate(ts: number): string {
    if (!ts) return "";
    const d = new Date(ts * 1000);
    return (
        d.toLocaleDateString() +
        " " +
        d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    );
}

function onResizeStart(e: MouseEvent) {
    emit("resizeStart", e);
}

async function openFileExternally() {
    if (file.value) {
        try {
            await openFile(file.value.path);
        } catch {
            /* ignore */
        }
    }
}

async function openPreviewWindow() {
    if (!file.value) return;
    // Pause & destroy embedded video player before opening standalone window
    if (previewType.value === "video" && videoRef.value?.destroyPlayer) {
        videoRef.value.destroyPlayer();
    }
    const label = `preview-${file.value.path.replace(/[^a-zA-Z0-9]/g, "_")}`;
    try {
        const existing = await WebviewWindow.getByLabel(label);
        if (existing) {
            await existing.setFocus();
            return;
        }
    } catch {
        /* window doesn't exist yet, proceed to create */
    }
    try {
        const { width, height } = await getAdaptivePreviewSize();
        const win = new WebviewWindow(label, {
            url: `/?preview=${encodeURIComponent(file.value.path)}`,
            title: file.value.name,
            width,
            height,
            minWidth: 640,
            minHeight: 400,
            decorations: false,
            resizable: true,
            center: true,
            focus: true,
        });
        // 覆盖 window-state 插件恢复的状态，确保预览窗口始终自适应尺寸
        win.once("tauri://created", async () => {
            const { PhysicalSize } = await import("@tauri-apps/api/dpi");
            await win.setSize(new PhysicalSize(width, height));
            await win.center();
            setTimeout(() => win.setFocus().catch(() => {}), 80);
        });
    } catch (e) {
        console.error("Failed to open preview window:", e);
    }
}
</script>

<style scoped>
/* ── Panel container ── */
.preview-panel {
    position: relative;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-size: var(--font-size-base);
}
.preview-resize-handle {
    position: absolute;
    top: 0;
    left: -3px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 2;
    background: transparent;
    transition: background 0.1s;
}
.preview-resize-handle:hover,
.preview-resize-handle:active {
    background: var(--accent);
    opacity: 0.7;
}

/* ── Header ── */
.preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.preview-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
}
.preview-popout {
    background: none;
    border: 1px solid transparent;
    color: var(--text-muted);
    cursor: pointer;
    font-size: var(--font-size-xl);
    padding: 2px 6px;
    border-radius: 4px;
    line-height: 1;
    transition: all 0.15s;
}
.preview-popout:hover {
    background: var(--bg-hover);
    color: var(--accent);
    border-color: var(--border);
}
.preview-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: var(--font-size-lg);
    padding: 2px 6px;
    border-radius: 4px;
}
.preview-close:hover {
    background: var(--bg-hover);
}

/* ── Empty / multi-select ── */
.preview-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--font-size-base);
}
.preview-multi {
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.preview-multi-text {
    font-weight: 600;
    color: var(--text-primary);
    font-size: var(--font-size-base);
}
.preview-meta {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
}

/* ── Preview area (top, flex-fill) ── */
.preview-area {
    flex: 1;
    overflow: auto;
    min-height: 0;
    padding: 4px;
    background: var(--bg-primary);
}
.preview-area:empty::after {
    content: "";
    display: block;
    min-height: 120px;
}
.preview-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 48px 12px;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    min-height: 200px;
}
.preview-status-error {
    color: var(--danger);
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
}
.preview-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: preview-spin 0.6s linear infinite;
    flex-shrink: 0;
}
@keyframes preview-spin {
    to {
        transform: rotate(360deg);
    }
}

/* ── Preview types ── */
.preview-zoom-wrap {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.preview-zoom-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg-tertiary);
}
.preview-zoom-btn {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
    border-radius: 3px;
    width: 24px;
    height: 22px;
    font-size: var(--font-size-lg);
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
}
.preview-zoom-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.preview-zoom-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}
.preview-zoom-pct {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    min-width: 36px;
    text-align: center;
    font-variant-numeric: tabular-nums;
}
.preview-zoom-sep {
    width: 1px;
    height: 14px;
    background: var(--border);
    flex-shrink: 0;
}
.preview-zoom-scroll {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px;
    background: var(--bg-primary);
}
/* PDF container fills scroll area inside flex parent */
.preview-zoom-scroll .preview-office {
    align-self: stretch;
    width: 100%;
    height: 100%;
}
.preview-image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    transition: transform 0.1s;
    transform-origin: center center;
}
.preview-office {
    height: 100%;
    overflow: auto;
}
.preview-code {
    height: 100%;
    overflow: auto;
}
.preview-markdown {
    height: 100%;
    overflow: hidden;
}

/* ── Info section (bottom, compact) ── */
.preview-info {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    padding: 10px 12px;
    background: var(--bg-primary);
}
.preview-info-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}
.preview-info-icon {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
}
.preview-info-icon :deep(svg) {
    width: 22px;
    height: 22px;
}
.preview-info-name {
    font-weight: 600;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    word-break: break-all;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
/* ── Properties table ── */
.props-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--font-size-sm);
}
.props-table tr {
    border-bottom: 1px solid var(--border);
}
.props-table tr:last-child {
    border-bottom: none;
}
.props-td-label {
    padding: 5px 8px 5px 0;
    color: var(--text-muted);
    white-space: nowrap;
    vertical-align: top;
    width: 1%;
}
.props-td-value {
    padding: 5px 0;
    text-align: left;
    color: var(--text-secondary);
    user-select: text;
    word-break: break-all;
}
.props-td-path {
    cursor: pointer;
    color: var(--accent);
}
.props-td-path:hover {
    text-decoration: underline;
}
.preview-info-actions {
    margin-top: 8px;
    display: flex;
    gap: 6px;
}
.preview-btn {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 3px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition:
        background 0.15s,
        color 0.15s;
}
.preview-btn:hover {
    background: var(--accent);
    color: #fff;
}
/* ── Panel context menu ── */
.panel-ctx-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
}
.panel-ctx-menu {
    position: fixed;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 180px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    font-size: var(--font-size-sm);
}
.panel-ctx-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 5px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
}
.panel-ctx-item:hover {
    background: var(--bg-hover);
}
.panel-ctx-item :deep(svg) {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
}
.panel-ctx-sep {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
}
.preview-markdown {
    height: 100%;
    overflow: hidden;
}
/* ── Archive listing ── */
.preview-archive {
    height: 100%;
    display: flex;
    flex-direction: column;
}
.preview-archive-header {
    padding: 6px 8px;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.preview-archive-list {
    flex: 1;
    overflow: auto;
    padding: 2px 0;
}
.preview-archive-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: var(--font-size-sm);
    white-space: nowrap;
}
.preview-archive-icon {
    flex-shrink: 0;
    font-size: var(--font-size-sm);
}
.preview-archive-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
}
.preview-archive-size {
    margin-left: auto;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    flex-shrink: 0;
}

/* ── Unsupported format placeholder ── */
.preview-unsupported {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    color: var(--text-muted);
}
.preview-unsupported-icon {
    font-size: 32px;
    opacity: 0.5;
}
.preview-unsupported-text {
    font-size: var(--font-size-base);
    color: var(--text-muted);
}

/* ── Skeleton loading animation ── */
@keyframes preview-pulse {
    0%,
    100% {
        opacity: 0.4;
    }
    50% {
        opacity: 0.8;
    }
}
.preview-skeleton {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    animation: preview-pulse 1.5s ease-in-out infinite;
}
.preview-skeleton-line {
    height: 12px;
    background: var(--bg-hover);
    border-radius: 4px;
}
.preview-skeleton-line:nth-child(1) {
    width: 60%;
}
.preview-skeleton-line:nth-child(2) {
    width: 80%;
}
.preview-skeleton-line:nth-child(3) {
    width: 45%;
}
.preview-skeleton-line:nth-child(4) {
    width: 70%;
}
.preview-skeleton-line:nth-child(5) {
    width: 55%;
}
.preview-skeleton-block {
    height: 120px;
    background: var(--bg-hover);
    border-radius: 6px;
}
</style>
