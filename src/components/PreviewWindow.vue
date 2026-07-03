<template>
    <div class="pw-root">
        <div
            class="pw-top-bar"
            data-tauri-drag-region
            @mousedown="onTopBarDrag"
        >
            <TitleBar @open-settings="showSettings = true" />
            <div class="pw-header">
                <template v-if="renaming">
                    <input
                        ref="renameInput"
                        class="pw-rename-input"
                        v-model="renameValue"
                        @keydown.enter="confirmRename"
                        @keydown.escape="cancelRename"
                        @blur="confirmRename"
                    />
                </template>
                <template v-else>
                    <span class="pw-title">{{ fileName || "Preview" }}</span>
                </template>
            </div>
            <!-- Toolbar -->
            <div class="pw-toolbar" v-if="activePath && !viewingExtracted">
                <button
                    class="pw-tb-btn"
                    :title="$t('previewToolbar.open')"
                    @click="tbOpen"
                >
                    <span class="pw-tb-icon" v-html="ICONS.open"></span>
                    {{ $t("previewToolbar.open") }}
                </button>
                <button
                    class="pw-tb-btn"
                    :title="$t('previewToolbar.rename')"
                    @click="tbRename"
                >
                    <span class="pw-tb-icon" v-html="ICONS.rename"></span>
                    {{ $t("previewToolbar.rename") }}
                </button>
                <button
                    class="pw-tb-btn"
                    :title="$t('previewToolbar.saveAs')"
                    @click="tbSaveAs"
                >
                    <span class="pw-tb-icon" v-html="ICONS.saveAs"></span>
                    {{ $t("previewToolbar.saveAs") }}
                </button>
                <button
                    class="pw-tb-btn pw-tb-btn--danger"
                    :title="$t('previewToolbar.delete')"
                    @click="tbDelete"
                >
                    <span class="pw-tb-icon" v-html="ICONS.delete"></span>
                    {{ $t("previewToolbar.delete") }}
                </button>
                <div class="pw-tb-sep"></div>
                <button
                    class="pw-tb-btn"
                    :title="$t('previewToolbar.copyPath')"
                    @click="tbCopyPath"
                >
                    <span class="pw-tb-icon" v-html="ICONS.copy"></span>
                    {{
                        tbcopied
                            ? $t("previewToolbar.copied")
                            : $t("previewToolbar.copyPath")
                    }}
                </button>
                <button
                    class="pw-tb-btn"
                    :title="$t('previewToolbar.showInExplorer')"
                    @click="tbShowInExplorer"
                >
                    <span
                        class="pw-tb-icon"
                        v-html="ICONS.showInExplorer"
                    ></span>
                    {{ $t("previewToolbar.showInExplorer") }}
                </button>
            </div>
            <!-- Back to archive -->
            <div v-if="viewingExtracted" class="pw-back-bar">
                <button class="pw-back-btn" @click="backToArchive">
                    ← {{ archiveFileName }}
                </button>
            </div>
        </div>
        <div class="pw-body">
            <!-- Left: tree -->
            <div class="pw-tree" :style="{ width: treeWidth + 'px' }">
                <div class="pw-tree-nav">
                    <button
                        class="pw-nav-btn"
                        :disabled="!canGoUp"
                        @click="goUp"
                    >
                        ← {{ $t("toolbar.up") }}
                    </button>
                </div>
                <div
                    class="pw-tree-list"
                    tabindex="-1"
                    @keydown="onTreeKeydown"
                    @contextmenu.prevent="onTreeCtxMenu"
                >
                    <div v-if="flatTree.length === 0" class="pw-tree-empty">
                        {{ $t("properties.noPreview") }}
                    </div>
                    <template v-for="node in flatTree" :key="node.path">
                        <div
                            class="pw-tree-item"
                            :class="{
                                'pw-tree-item--active':
                                    node.path === activePath,
                                'pw-tree-item--dir': node.isDir,
                                'pw-tree-focused':
                                    flatTree.indexOf(node) === treeFocusIndex,
                            }"
                            :data-tree-idx="flatTree.indexOf(node)"
                            :style="{ paddingLeft: 8 + node.depth * 18 + 'px' }"
                            @click="onTreeClick(node)"
                            @dblclick="onTreeDblClick(node)"
                            @contextmenu.prevent.stop="
                                onItemCtxMenu(node, $event)
                            "
                        >
                            <!-- Chevron -->
                            <span
                                class="pw-tree-chevron"
                                :class="{
                                    'pw-tree-chevron--expanded':
                                        node.isDir && node.expanded,
                                    'pw-tree-chevron--hidden':
                                        !node.isDir || !node.hasChildren,
                                }"
                                @click.stop="toggleExpand(node)"
                            >
                                <svg viewBox="0 0 10 10" width="10" height="10">
                                    <path
                                        d="M3.5 2l3.5 3-3.5 3"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.3"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    />
                                </svg>
                            </span>
                            <!-- Icon -->
                            <span
                                class="pw-tree-icon-wrap"
                                :class="node.isDir ? '' : fileColorClass(node)"
                            >
                                <div
                                    v-if="!node.isDir || isBundle(node)"
                                    class="pw-tree-icon-svg"
                                    v-html="fileIcon(node)"
                                ></div>
                                <svg
                                    v-else
                                    class="pw-tree-icon-svg"
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
                            </span>
                            <span class="pw-tree-name">{{ node.name }}</span>
                            <span v-if="!node.isDir" class="pw-tree-size">{{
                                node.sizeFmt
                            }}</span>
                        </div>
                        <!-- Children (keep alive, hide when collapsed) -->
                        <template v-if="node.isDir && node.expanded">
                            <div
                                v-if="node.loading"
                                class="pw-tree-loading"
                                :style="{
                                    paddingLeft:
                                        8 + (node.depth + 1) * 18 + 'px',
                                }"
                            >
                                <span class="pw-spinner-sm"></span>
                            </div>
                        </template>
                    </template>
                </div>
            </div>

            <!-- Resize handle -->
            <div class="pw-split-handle" @mousedown.stop="onSplitStart" />

            <!-- Right: preview -->
            <div class="pw-preview" @contextmenu.prevent>
                <button
                    v-if="viewingExtracted"
                    class="pw-preview-back"
                    :title="
                        $t('properties.backToArchive', {
                            name: archiveFileName,
                        })
                    "
                    @click="backToArchive"
                >
                    ↰
                </button>
                <button
                    v-if="canCopy"
                    class="pw-preview-copy"
                    :title="
                        pwCopied
                            ? $t('previewToolbar.copied')
                            : $t('previewToolbar.copyContent')
                    "
                    @click="copyContent"
                >
                    <svg
                        v-if="pwCopied"
                        viewBox="0 0 14 14"
                        width="14"
                        height="14"
                    >
                        <path
                            d="M4.5 7l2 2 3.5-4"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                    <svg v-else viewBox="0 0 14 14" width="14" height="14">
                        <rect
                            x="3.5"
                            y="1.5"
                            width="7"
                            height="9"
                            rx="1"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1"
                        />
                        <rect
                            x="1.5"
                            y="3.5"
                            width="7"
                            height="9"
                            rx="1"
                            fill="var(--bg-secondary)"
                            stroke="currentColor"
                            stroke-width="1"
                        />
                    </svg>
                </button>
                <div v-if="previewLoading" class="pw-status">
                    <div class="pw-skeleton">
                        <div class="pw-skeleton-line"></div>
                        <div class="pw-skeleton-line"></div>
                        <div class="pw-skeleton-line"></div>
                        <div class="pw-skeleton-block"></div>
                    </div>
                </div>
                <div
                    v-else-if="!previewType && !previewError"
                    class="pw-status"
                >
                    <span class="pw-status-text">{{
                        $t("properties.noFileSelected")
                    }}</span>
                </div>
                <div
                    v-else-if="previewError"
                    class="pw-status pw-status--error"
                >
                    <span>{{ previewError }}</span>
                </div>
                <div v-else-if="previewType === 'image'" class="pw-zoom-wrap">
                    <div class="pw-zoom-bar">
                        <button
                            class="pw-zoom-btn"
                            :disabled="imageZoom <= 0.25"
                            :title="$t('properties.zoomOut')"
                            @click.stop="
                                imageZoom = Math.max(0.25, imageZoom - 0.1)
                            "
                        >
                            −
                        </button>
                        <span class="pw-zoom-pct"
                            >{{ Math.round(imageZoom * 100) }}%</span
                        >
                        <button
                            class="pw-zoom-btn"
                            :disabled="imageZoom >= 5"
                            :title="$t('properties.zoomIn')"
                            @click.stop="
                                imageZoom = Math.min(5, imageZoom + 0.1)
                            "
                        >
                            +
                        </button>
                        <button
                            class="pw-zoom-btn"
                            :title="$t('properties.zoomReset')"
                            @click.stop="
                                imageZoom = 1;
                                imageRotation = 0;
                            "
                        >
                            ⊡
                        </button>
                        <div class="pw-zoom-sep"></div>
                        <button
                            class="pw-zoom-btn"
                            :title="$t('properties.rotateLeft')"
                            @click.stop="
                                imageRotation = (imageRotation + 270) % 360
                            "
                        >
                            ↺
                        </button>
                        <button
                            class="pw-zoom-btn"
                            :title="$t('properties.rotateRight')"
                            @click.stop="
                                imageRotation = (imageRotation + 90) % 360
                            "
                        >
                            ↻
                        </button>
                    </div>
                    <div
                        class="pw-zoom-scroll"
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
                            class="pw-image"
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
                <div v-else-if="previewType === 'docx'" class="pw-office">
                    <VueOfficeDocx
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <div v-else-if="previewType === 'xlsx'" class="pw-office">
                    <VueOfficeExcel
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <div v-else-if="previewType === 'xls'" class="pw-office">
                    <XlsPreview
                        v-if="officeArrayBuffer"
                        :data="officeArrayBuffer"
                    />
                </div>
                <div v-else-if="previewType === 'video'" class="pw-office">
                    <VideoPreview :src="previewSrc" />
                </div>
                <div v-else-if="previewType === 'pdf'" class="pw-zoom-wrap">
                    <div class="pw-zoom-bar">
                        <button
                            class="pw-zoom-btn"
                            :disabled="pdfZoom <= 0.5"
                            :title="$t('properties.zoomOut')"
                            @click.stop="pdfZoom = Math.max(0.5, pdfZoom - 0.2)"
                        >
                            −
                        </button>
                        <span class="pw-zoom-pct"
                            >{{ Math.round(pdfZoom * 100) }}%</span
                        >
                        <button
                            class="pw-zoom-btn"
                            :disabled="pdfZoom >= 3"
                            :title="$t('properties.zoomIn')"
                            @click.stop="pdfZoom = Math.min(3, pdfZoom + 0.2)"
                        >
                            +
                        </button>
                        <button
                            class="pw-zoom-btn"
                            :title="$t('properties.zoomReset')"
                            @click.stop="pdfZoom = 1"
                        >
                            ⊡
                        </button>
                    </div>
                    <div
                        class="pw-zoom-scroll"
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
                            class="pw-office"
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
                <div v-else-if="previewType === 'pptx'" class="pw-office">
                    <PptxPreview
                        v-if="officeArrayBuffer"
                        :data="officeArrayBuffer"
                    />
                </div>
                <div v-else-if="previewType === 'text'" class="pw-code">
                    <CodePreview :code="previewContent" :ext="previewExt" />
                </div>
                <div v-else-if="previewType === 'markdown'" class="pw-markdown">
                    <MarkdownPreview
                        :content="previewContent"
                        :ext="previewExt"
                        :filePath="activePath"
                    />
                </div>
                <div
                    v-else-if="previewType === 'externalOnly'"
                    class="pw-status"
                >
                    <span>{{ $t("properties.noPreview") }}</span>
                </div>
                <div v-else-if="previewType === 'archive'" class="pw-archive">
                    <div class="pw-archive-hdr">
                        {{ archiveEntries.length }} entries
                    </div>
                    <div class="pw-archive-list">
                        <div
                            v-for="e in archiveEntries"
                            :key="e.path"
                            class="pw-archive-item"
                            :class="{
                                'pw-archive-item--active':
                                    !e.is_dir && selectedArchivePath === e.path,
                            }"
                            :style="{
                                paddingLeft:
                                    8 +
                                    (e.path.match(/\//g) || []).length * 14 +
                                    'px',
                            }"
                            @click="onArchiveEntryClick(e)"
                            @dblclick="onArchiveEntryDblClick(e)"
                            @contextmenu.prevent="onArchiveCtxMenu(e, $event)"
                        >
                            <span>{{ e.is_dir ? "📁" : "📄" }}</span>
                            <span class="pw-archive-name">{{ e.name }}</span>
                            <span v-if="!e.is_dir" class="pw-archive-size">{{
                                e.size
                            }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Context menu -->
        <div
            v-if="ctxMenu.show"
            class="pw-ctx-overlay"
            @click="ctxMenu.show = false"
        >
            <div
                class="pw-ctx-menu"
                :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
            >
                <button class="pw-ctx-item" @click="ctxOpenFile">
                    ↗ {{ $t("contextMenu.open") }}
                </button>
                <button class="pw-ctx-item" @click="ctxShowInExplorer">
                    📂
                    {{
                        isMac
                            ? $t("contextMenu.showInFinder")
                            : $t("contextMenu.showInExplorer")
                    }}
                </button>
                <button class="pw-ctx-item" @click="ctxOpenInTerminal">
                    ⌨ {{ $t("contextMenu.openInTerminal") }}
                </button>
                <div class="pw-ctx-sep"></div>
                <button class="pw-ctx-item" @click="ctxCopyPath">
                    📋 {{ $t("contextMenu.copyPath") }}
                </button>
            </div>
        </div>
        <SettingsDialog v-if="showSettings" @close="showSettings = false" />
    </div>
</template>

<script setup lang="ts">
import {
    ref,
    computed,
    reactive,
    onMounted,
    onUnmounted,
    watch,
    shallowRef,
} from "vue";
import { useI18n } from "vue-i18n";

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
    if (known[err]) return known[err];
    if (err.startsWith("Failed to stat")) return t("properties.failedToStat");
    if (err.startsWith("Read failed")) return t("properties.readFailed");
    if (err.startsWith("File too large")) return t("properties.fileTooLarge");
    return err;
}
import {
    listDirectory,
    getFilePreview,
    readFileBytes,
    listArchiveContents,
    extractArchiveEntry,
    copyFileAs,
    deleteItem,
    openFile,
    showInExplorer,
    openInTerminal,
    renameItem,
} from "@/utils/tauri";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";

function onTopBarDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, select, a, .traffic-btn")) return;
    getCurrentWebviewWindow().startDragging();
}
import { convertFileSrc } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { ask } from "@tauri-apps/plugin-dialog";

import VueOfficeDocx from "@vue-office/docx";
import VueOfficeExcel from "@vue-office/excel";
import VueOfficePdf from "@vue-office/pdf";
import PptxPreview from "@/components/PptxPreview.vue";
import CodePreview from "@/components/CodePreview.vue";
import MarkdownPreview from "@/components/MarkdownPreview.vue";
import XlsPreview from "@/components/XlsPreview.vue";
import VideoPreview from "@/components/VideoPreview.vue";
import SettingsDialog from "@/components/Dialogs/SettingsDialog.vue";
import TitleBar from "@/components/TitleBar.vue";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";
import {
    getFileCategory,
    treeColorClassForCategory,
    formatFileSize,
} from "@/utils/fileTypes";
import type { FileEntry } from "@/types";
import type { ArchiveEntry } from "@/utils/tauri";

// ── SVG icons (matching app icon system) ──
const ICONS = {
    open: `<svg viewBox="0 0 14 14"><path d="M2 4.5a1 1 0 011-1h2.5l1.2 1.5H11a1 1 0 011 1V11a1 1 0 01-1 1H3a1 1 0 01-1-1V4.5z" fill="none" stroke="currentColor" stroke-width="1"/></svg>`,
    showInExplorer: `<svg viewBox="0 0 14 14"><path d="M8 2h4v4M6 8l6-6M4 3H3a1 1 0 00-1 1v7a1 1 0 001 1h7a1 1 0 001-1v-1" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    copy: `<svg viewBox="0 0 14 14"><rect x="3.5" y="1.5" width="7" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1"/><rect x="1.5" y="3.5" width="7" height="9" rx="1" fill="var(--bg-secondary)" stroke="currentColor" stroke-width="1"/></svg>`,
    saveAs: `<svg viewBox="0 0 14 14"><path d="M2.5 10v1.5a1 1 0 001 1h7a1 1 0 001-1V10" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round"/><path d="M7 2v7M4.5 5.5L7 8l2.5-2.5" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    delete: `<svg viewBox="0 0 14 14"><path d="M3 3.5h8M5.5 3V2.5a.5.5 0 01.5-.5h2a.5.5 0 01.5.5V3" fill="none" stroke="currentColor" stroke-width="1"/><path d="M4 3.5v8a1 1 0 001 1h4a1 1 0 001-1v-8" fill="none" stroke="currentColor" stroke-width="1"/></svg>`,
    rename: `<svg viewBox="0 0 14 14"><path d="M3 11l3-7.5L7.5 7l-3 4H3zm4.5-8l1.5 1.5M10 2l2 2-4 4-2.5-.5L6 4.5 10 2z" fill="none" stroke="currentColor" stroke-width="1" stroke-linejoin="round"/></svg>`,
};

const { t } = useI18n();

const isMac =
    typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

// ── Start path ──
const params = new URLSearchParams(window.location.search);
const startPath = decodeURIComponent(params.get("preview") || "");

const activePath = ref(startPath);

// ── Tree state ──
interface TreeNode {
    path: string;
    name: string;
    isDir: boolean;
    size: number;
    sizeFmt: string;
    ext: string;
    depth: number;
    expanded: boolean;
    hasChildren: boolean;
    loading: boolean;
}
const treeNodes = shallowRef<Map<string, TreeNode>>(new Map());
const treeOrder = ref<string[]>([]);

const flatTree = computed<TreeNode[]>(() => {
    const result: TreeNode[] = [];
    const visited = new Set<string>();
    function walk(paths: string[], depth: number) {
        for (const p of paths) {
            if (visited.has(p)) continue;
            visited.add(p);
            const node = treeNodes.value.get(p);
            if (!node) continue;
            node.depth = depth;
            result.push(node);
            if (node.isDir && node.expanded) {
                const children = getChildPaths(p);
                walk(children, depth + 1);
            }
        }
    }
    walk(treeOrder.value, 0);
    return result;
});

function getChildPaths(parentPath: string): string[] {
    return treeOrder.value.filter((p) => {
        const dir = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
        return dir === parentPath.replace(/\\/g, "/") || dir === parentPath;
    });
}

// ── Tree width resize ──
const treeWidth = ref(loadTreeWidth());

function loadTreeWidth(): number {
    try {
        const v = localStorage.getItem("pw-tree-width");
        return v ? Math.max(160, parseInt(v)) : 220;
    } catch {
        return 220;
    }
}

let _splitStartX = 0;
let _splitStartW = 0;

function onSplitStart(e: MouseEvent) {
    _splitStartX = e.clientX;
    _splitStartW = treeWidth.value;
    addEventListener("mousemove", onSplitMove);
    addEventListener("mouseup", onSplitEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onSplitMove(e: MouseEvent) {
    treeWidth.value = Math.max(
        160,
        Math.min(500, _splitStartW + e.clientX - _splitStartX),
    );
}

function onSplitEnd() {
    removeEventListener("mousemove", onSplitMove);
    removeEventListener("mouseup", onSplitEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("pw-tree-width", String(treeWidth.value));
}

// ── Tree operations ──
async function loadDirTree(dir: string, parentPath?: string) {
    try {
        const items = await listDirectory(dir);
        const normDir = dir.replace(/\\/g, "/");
        const nodes = new Map(treeNodes.value);

        for (const f of items) {
            const p = f.path.replace(/\\/g, "/");
            if (nodes.has(p)) continue;
            const node: TreeNode = {
                path: f.path,
                name: f.name,
                isDir: f.is_dir,
                size: f.size,
                sizeFmt: f.is_dir ? "" : formatFileSize(f.size),
                ext: f.extension,
                depth: 0,
                expanded: false,
                hasChildren: f.is_dir,
                loading: false,
            };
            nodes.set(p, node);
        }

        // Update order: remove old children of this dir, insert new ones sorted
        const newOrder = items.map((f) => f.path.replace(/\\/g, "/"));
        let order = [...treeOrder.value];
        if (parentPath) {
            const pp = parentPath.replace(/\\/g, "/");
            order = order.filter((p) => {
                const d = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
                return d !== pp;
            });
        }
        // Insert after parent (if parent in list) or at end
        if (parentPath) {
            const pp = parentPath.replace(/\\/g, "/");
            const idx = order.indexOf(pp);
            if (idx >= 0) {
                order.splice(idx + 1, 0, ...newOrder);
            } else {
                order.push(...newOrder);
            }
        } else {
            order = [...newOrder];
        }

        treeNodes.value = nodes;
        treeOrder.value = order;
    } catch (e) {
        console.error("[PreviewWindow] loadDirTree failed:", dir, e);
    }
}

async function toggleExpand(node: TreeNode) {
    if (!node.isDir) return;
    const map = new Map(treeNodes.value);
    const n = map.get(node.path);
    if (!n) return;

    if (n.expanded) {
        n.expanded = false;
        treeNodes.value = map;
        return;
    }

    // Check if has children (load if first expand)
    n.loading = true;
    treeNodes.value = new Map(map);
    await loadDirTree(node.path, node.path);
    const map2 = new Map(treeNodes.value);
    const n2 = map2.get(node.path);
    if (n2) {
        n2.expanded = true;
        n2.loading = false;
        n2.hasChildren =
            treeOrder.value.filter((p) => {
                const d = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
                return d === node.path.replace(/\\/g, "/");
            }).length > 0;
        treeNodes.value = map2;
    }
}

function onTreeClick(node: TreeNode) {
    const idx = flatTree.value.indexOf(node);
    if (idx >= 0) treeFocusIndex.value = idx;
    if (!node.isDir) {
        activePath.value = node.path;
    }
}

// ── Tree keyboard navigation ──
const treeFocusIndex = ref(0);

function onTreeKeydown(e: KeyboardEvent) {
    const tree = flatTree.value;
    if (tree.length === 0) return;
    const shift = e.shiftKey;
    switch (e.key) {
        case "ArrowUp":
            e.preventDefault();
            treeFocusIndex.value = Math.max(0, treeFocusIndex.value - 1);
            syncTreePreview();
            break;
        case "ArrowDown":
            e.preventDefault();
            treeFocusIndex.value = Math.min(
                tree.length - 1,
                treeFocusIndex.value + 1,
            );
            syncTreePreview();
            break;
        case "ArrowRight":
            e.preventDefault();
            {
                const node = tree[treeFocusIndex.value];
                if (node && node.isDir && !node.expanded) {
                    toggleExpand(node);
                }
            }
            break;
        case "ArrowLeft":
            e.preventDefault();
            {
                const node = tree[treeFocusIndex.value];
                if (node && node.isDir && node.expanded) {
                    toggleExpand(node);
                } else {
                    goUp();
                }
            }
            break;
        case "Enter":
        case " ":
            e.preventDefault();
            {
                const node = tree[treeFocusIndex.value];
                if (node) {
                    if (node.isDir) toggleExpand(node);
                    else activePath.value = node.path;
                }
            }
            break;
        case "Home":
            e.preventDefault();
            treeFocusIndex.value = 0;
            syncTreePreview();
            break;
        case "End":
            e.preventDefault();
            treeFocusIndex.value = tree.length - 1;
            syncTreePreview();
            break;
    }
    // Scroll focused item into view
    const el = document.querySelector(
        `[data-tree-idx="${treeFocusIndex.value}"]`,
    );
    el?.scrollIntoView({ block: "nearest" });
}

function syncTreePreview() {
    const node = flatTree.value[treeFocusIndex.value];
    if (node && !node.isDir && node.path !== activePath.value) {
        activePath.value = node.path;
    }
}

function onTreeDblClick(node: TreeNode) {
    if (node.isDir) {
        toggleExpand(node);
    }
}

async function onArchiveEntryDblClick(entry: ArchiveEntry) {
    if (entry.is_dir) return;
    try {
        // Save archive state for return
        lastArchivePath.value = archivePath.value;
        lastArchiveEntries.value = [...archiveEntries.value];
        viewingExtracted.value = true;

        const result = await extractArchiveEntry(archivePath.value, entry.path);
        activePath.value = result.temp_path;
    } catch (e: any) {
        previewError.value = translatePreviewError(
            String(e || "Extract failed"),
            t,
        );
    }
}

function backToArchive() {
    archiveEntries.value = lastArchiveEntries.value;
    archivePath.value = lastArchivePath.value;
    previewType.value = "archive";
    activePath.value = "";
    viewingExtracted.value = false;
    previewError.value = "";
}

async function copyContent() {
    const text = previewContent.value || "";
    if (!text) return;
    try {
        await navigator.clipboard.writeText(text);
        pwCopied.value = true;
        setTimeout(() => (pwCopied.value = false), 1500);
    } catch {
        /* ignore */
    }
}

function onArchiveEntryClick(entry: ArchiveEntry) {
    if (!entry.is_dir) {
        selectedArchivePath.value = entry.path;
    }
}

function onArchiveCtxMenu(entry: ArchiveEntry, e: MouseEvent) {
    ctxMenu.show = true;
    ctxMenu.x = e.clientX;
    ctxMenu.y = e.clientY;
    ctxMenu.path = entry.path;
}

// ── Toolbar actions ──
function tbOpen() {
    const p = activePath.value || archivePath.value;
    if (p) openFile(p).catch(() => {});
}
function tbShowInExplorer() {
    const p = activePath.value || archivePath.value;
    if (p) showInExplorer(p).catch(() => {});
}
async function tbCopyPath() {
    const p = activePath.value || archivePath.value;
    if (!p) return;
    try {
        await navigator.clipboard.writeText(p);
        tbcopied.value = true;
        setTimeout(() => (tbcopied.value = false), 1500);
    } catch {
        /* ignore */
    }
}
async function tbSaveAs() {
    const src = activePath.value || archivePath.value;
    if (!src) return;
    try {
        const dest = await save({ defaultPath: fileName.value || "file" });
        if (dest) await copyFileAs(src, dest);
    } catch {
        /* ignore */
    }
}
async function tbRename() {
    const p = activePath.value;
    if (!p) return;
    renameValue.value = fileName.value;
    renaming.value = true;
    setTimeout(() => renameInput.value?.focus(), 50);
}

async function confirmRename() {
    const p = activePath.value;
    if (!p || !renameValue.value || renameValue.value === fileName.value) {
        renaming.value = false;
        return;
    }
    const sep = p.includes("/") ? "/" : "\\";
    const newPath =
        p.substring(0, p.lastIndexOf(sep)) + sep + renameValue.value;
    try {
        await renameItem(p, newPath);
        activePath.value = newPath;
        emit("file-changed", { path: parentDir.value });
        await loadDirTree(parentDir.value);
        await loadPreview(newPath);
    } catch (e: any) {
        previewError.value = String(e);
    }
    renaming.value = false;
}

function cancelRename() {
    renaming.value = false;
}

async function tbDelete() {
    const p = activePath.value || archivePath.value;
    if (!p) return;
    const confirmed = await ask(
        t("previewToolbar.confirmDelete", { name: fileName.value }) +
            "\n" +
            t("previewToolbar.deleteWarning"),
        { title: t("previewToolbar.delete"), kind: "warning" },
    );
    if (!confirmed) return;
    try {
        await deleteItem(p, false);
        emit("file-changed", { path: parentDir.value });
        getCurrentWebviewWindow().close();
    } catch {
        /* ignore */
    }
}

// ── Navigation ──
const canGoUp = computed(() => {
    const p = parentDir.value;
    if (!p) return false;
    if (/^[A-Za-z]:\\?$/.test(p) && p.length <= 3) return false;
    if (p === "/") return false;
    return true;
});

const parentDir = computed(() => {
    const p = activePath.value.replace(/\\/g, "/");
    const lastSep = p.lastIndexOf("/");
    if (lastSep <= 0) return p;
    return p.slice(0, lastSep);
});

const fileName = computed(() => {
    const parts = activePath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || "";
});

const archiveFileName = computed(() => {
    const parts = lastArchivePath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || "";
});

async function goUp() {
    const pp = parentDir.value;
    if (!pp) return;
    const grandParent =
        pp.replace(/\\/g, "/").replace(/\/[^/]+$/, "") ||
        pp.slice(0, pp.indexOf("/") + 1) ||
        "/";
    treeOrder.value = [];
    treeNodes.value = new Map();
    await loadDirTree(pp);
    // Expand the parent dir so its children are visible
    const map = new Map(treeNodes.value);
    const pn = map.get(pp);
    if (pn) {
        pn.expanded = true;
        treeNodes.value = map;
        await loadDirTree(pp, pp);
        const map2 = new Map(treeNodes.value);
        const pn2 = map2.get(pp);
        if (pn2) {
            pn2.expanded = true;
            pn2.loading = false;
            treeNodes.value = map2;
        }
    }
}

// ── Icons & colors ──
function fileIcon(node: TreeNode): string {
    if (node.isDir && !isBundleDirectory(node.ext, true)) return "";
    return getFileIconSvg(node.ext, false) || "";
}
function isBundle(node: TreeNode): boolean {
    return isBundleDirectory(node.ext, true);
}
function fileColorClass(node: TreeNode): string {
    return treeColorClassForCategory(getFileCategory(node.ext, false));
}

// ── Context menu ──
const ctxMenu = reactive({ show: false, x: 0, y: 0, path: "" });

function onTreeCtxMenu(e: MouseEvent) {
    ctxMenu.show = false;
}

function onItemCtxMenu(node: TreeNode, e: MouseEvent) {
    ctxMenu.show = true;
    ctxMenu.x = e.clientX;
    ctxMenu.y = e.clientY;
    ctxMenu.path = node.path;
}
function ctxOpenFile() {
    openFile(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
function ctxShowInExplorer() {
    showInExplorer(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
function ctxOpenInTerminal() {
    openInTerminal(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
async function ctxCopyPath() {
    try {
        await navigator.clipboard.writeText(ctxMenu.path);
    } catch {
        /* ignore */
    }
    ctxMenu.show = false;
}

// ── Preview ──
const previewType = ref("");
const previewSrc = ref("");
const previewContent = ref("");
const previewExt = ref("");
const previewLoading = ref(false);
const previewError = ref("");
const officeData = ref<ArrayBuffer | string | null>(null);
const officeArrayBuffer = computed(() =>
    officeData.value instanceof ArrayBuffer ? officeData.value : null,
);
const archiveEntries = ref<ArchiveEntry[]>([]);
const archivePath = ref("");
const selectedArchivePath = ref("");
const lastArchivePath = ref("");
const lastArchiveEntries = ref<ArchiveEntry[]>([]);
const viewingExtracted = ref(false);
const pwCopied = ref(false);
const renaming = ref(false);
const renameValue = ref("");
const renameInput = ref<HTMLInputElement>();
const showSettings = ref(false);

const canCopy = computed(() =>
    ["text", "markdown"].includes(previewType.value),
);
const tbcopied = ref(false);

const imageZoom = ref(1);
const imageRotation = ref(0);
const pdfZoom = ref(1);

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

async function loadPreview(path: string) {
    if (!path) return;
    previewType.value = "";
    previewError.value = "";
    officeData.value = null;
    const ext = (path.split(".").pop() || "").toLowerCase();
    if (IMG_EXTS.includes(ext)) {
        previewType.value = "image";
        previewSrc.value = convertFileSrc(path);
        previewLoading.value = false;
    } else if (VIDEO_EXTS.includes(ext)) {
        const nativeExts = ["mp4", "webm", "ogg", "mov", "flv"];
        if (nativeExts.includes(ext)) {
            previewType.value = "video";
            previewSrc.value = convertFileSrc(path);
        } else {
            previewType.value = "externalOnly";
        }
        previewLoading.value = false;
    } else if (OFFICE_EXTS[ext] && OFFICE_EXTS[ext] !== "externalOnly") {
        previewLoading.value = true;
        try {
            const buf = await loadAsArrayBuffer(path);
            officeData.value = buf;
            previewType.value = OFFICE_EXTS[ext];
        } catch (e: any) {
            previewError.value = translatePreviewError(String(e), t);
        }
        previewLoading.value = false;
    } else if (OFFICE_EXTS[ext] === "externalOnly") {
        previewType.value = "externalOnly";
    } else if (ext === "pdf") {
        previewLoading.value = true;
        const pdfTimeout = setTimeout(() => {
            if (previewLoading.value) {
                previewError.value = t("properties.pdfLoadTimeout");
                previewLoading.value = false;
            }
        }, 15000);
        try {
            const b64 = await readFileBytes(path);
            clearTimeout(pdfTimeout);
            officeData.value = `data:application/pdf;base64,${b64}`;
            previewType.value = "pdf";
        } catch (e: any) {
            clearTimeout(pdfTimeout);
            previewError.value = translatePreviewError(String(e), t);
        }
        previewLoading.value = false;
    } else if (ARCHIVE_EXTS.includes(ext)) {
        previewLoading.value = true;
        try {
            const entries = await listArchiveContents(path);
            archiveEntries.value = entries;
            archivePath.value = path;
            previewType.value = "archive";
        } catch (e: any) {
            previewError.value = translatePreviewError(String(e), t);
        }
        previewLoading.value = false;
    } else {
        previewLoading.value = true;
        try {
            const result = await getFilePreview(path);
            if (result.type === "markdown") {
                previewType.value = "markdown";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || ext;
            } else {
                previewType.value = "text";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || ext;
            }
        } catch (e: any) {
            previewError.value = translatePreviewError(String(e), t);
        }
        previewLoading.value = false;
    }
}

async function loadAsArrayBuffer(path: string): Promise<ArrayBuffer> {
    const b64 = await readFileBytes(path);
    const chunks: string[] = [];
    for (let i = 0; i < b64.length; i += 1024 * 1024) {
        chunks.push(atob(b64.slice(i, i + 1024 * 1024)));
        await new Promise((r) => setTimeout(r, 0));
    }
    const bin = chunks.join("");
    const buf = new ArrayBuffer(bin.length);
    const v = new Uint8Array(buf);
    for (let i = 0; i < bin.length; i += 256 * 1024) {
        const end = Math.min(i + 256 * 1024, bin.length);
        for (let j = i; j < end; j++) v[j] = bin.charCodeAt(j);
        if (i + 256 * 1024 < bin.length)
            await new Promise((r) => setTimeout(r, 0));
    }
    return buf;
}

watch(activePath, (p) => {
    if (p) loadPreview(p);
});

onMounted(async () => {
    if (startPath) {
        await loadDirTree(parentDir.value);
        // Auto-expand the directory containing the active file
        const map = new Map(treeNodes.value);
        const pn = map.get(parentDir.value);
        if (pn) {
            pn.expanded = true;
            pn.loading = false;
            treeNodes.value = map;
            await loadDirTree(parentDir.value, parentDir.value);
            const map2 = new Map(treeNodes.value);
            const pn2 = map2.get(parentDir.value);
            if (pn2) {
                pn2.expanded = true;
                pn2.loading = false;
                treeNodes.value = map2;
            }
        }
        await loadPreview(startPath);
    }

    // ── Keyboard shortcuts ──
    const onKey = (e: KeyboardEvent) => {
        if (
            e.target instanceof HTMLInputElement ||
            e.target instanceof HTMLTextAreaElement
        )
            return;
        if (renaming.value) return;
        if (e.key === "Escape") {
            getCurrentWebviewWindow().close();
        } else if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            tbSaveAs();
        } else if (e.key === "Delete" && activePath.value) {
            e.preventDefault();
            tbDelete();
        } else if (e.key === "F2" && activePath.value) {
            e.preventDefault();
            tbRename();
        }
    };
    window.addEventListener("keydown", onKey);
    onUnmounted(() => window.removeEventListener("keydown", onKey));

    // 预览加载完成后强制获取焦点，避免被主窗口遮挡
    const forceFocus = () => {
        getCurrentWebviewWindow()
            .setFocus()
            .catch(() => {});
    };
    requestAnimationFrame(forceFocus);
    setTimeout(forceFocus, 200);
    setTimeout(forceFocus, 500);
    setTimeout(forceFocus, 1000);
});
</script>

<style scoped>
.pw-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
}
/* ── Top bar: unified drag region ── */
.pw-top-bar {
    -webkit-app-region: drag;
    flex-shrink: 0;
}
.pw-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
}
.pw-title {
    font-size: var(--font-size-base);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.pw-rename-input {
    -webkit-app-region: no-drag;
    background: var(--input-bg);
    border: 1px solid var(--accent);
    color: var(--text-primary);
    font-size: var(--font-size-base);
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    outline: none;
    width: 100%;
}
.pw-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: var(--font-size-xl);
    padding: 2px 6px;
    border-radius: 4px;
}
.pw-close:hover {
    background: var(--bg-hover);
    color: var(--danger);
}
/* ── Back bar ── */
.pw-back-bar {
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
}
.pw-back-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--accent);
    padding: 2px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: background 0.15s;
    -webkit-app-region: no-drag;
}
.pw-back-btn:hover {
    background: var(--bg-hover);
}
/* ── Toolbar ── */
.pw-toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
}
.pw-tb-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: 1px solid transparent;
    color: var(--text-secondary);
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
    white-space: nowrap;
    transition: all 0.1s;
    -webkit-app-region: no-drag;
}
.pw-tb-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border);
    color: var(--text-primary);
}
.pw-tb-btn--danger:hover {
    background: var(--danger);
    border-color: var(--danger);
    color: #fff;
}
.pw-tb-icon {
    width: 14px;
    height: 14px;
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
}
.pw-tb-icon :deep(svg) {
    width: 14px;
    height: 14px;
}
.pw-tb-sep {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 4px;
}
.pw-body {
    flex: 1;
    display: flex;
    overflow: hidden;
}

/* ── Tree ── */
.pw-tree {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    overflow: hidden;
}
.pw-tree-nav {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
}
.pw-nav-btn {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
    -webkit-app-region: no-drag;
}
.pw-nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
}
.pw-nav-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.pw-tree-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2px 0;
}
.pw-tree-item {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 2px 4px;
    margin: 0 4px;
    cursor: pointer;
    font-size: var(--font-size-sm);
    min-height: 26px;
    border-radius: 4px;
    white-space: nowrap;
    transition: background 0.05s;
}
.pw-tree-item:hover {
    background: var(--bg-hover);
}
.pw-tree-item--active {
    background: var(--bg-selected) !important;
}
.pw-tree-item--active:hover {
    background: var(--bg-selected) !important;
}
.pw-tree-focused:not(.pw-tree-item--active) {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
}
.pw-tree-chevron {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: transform 0.15s;
}
.pw-tree-chevron--expanded {
    transform: rotate(90deg);
}
.pw-tree-chevron--hidden {
    visibility: hidden;
}
.pw-tree-icon-wrap {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
}
.pw-tree-icon-svg {
    width: 16px;
    height: 16px;
}
.pw-tree-icon-svg :deep(svg) {
    width: 16px;
    height: 16px;
}
.pw-tree-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
    font-size: var(--font-size-sm);
}
.pw-tree-size {
    margin-left: auto;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    flex-shrink: 0;
}
.pw-tree-loading {
    padding: 3px 12px;
}
.pw-spinner-sm {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: pw-spin 0.6s linear infinite;
}
.pw-tree-empty {
    padding: 16px 12px;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    text-align: center;
}

/* ── Split handle ── */
.pw-split-handle {
    width: 4px;
    flex-shrink: 0;
    cursor: col-resize;
    background: transparent;
    transition: background 0.1s;
    z-index: 1;
}
.pw-split-handle:hover,
.pw-split-handle:active {
    background: var(--accent);
    opacity: 0.7;
}

/* ── Preview ── */
.pw-preview {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    padding: 8px;
    position: relative;
    background: var(--bg-primary);
}
.pw-preview:empty::after {
    content: "";
    display: block;
    min-height: 120px;
}
.pw-preview-back {
    position: absolute;
    top: 4px;
    right: 8px;
    z-index: 5;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--accent);
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    font-size: var(--font-size-xl);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition:
        opacity 0.15s,
        background 0.15s;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}
.pw-preview-back:hover {
    opacity: 1;
    background: var(--bg-hover);
}
.pw-preview-copy {
    position: absolute;
    top: 4px;
    right: 44px;
    z-index: 5;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    font-size: var(--font-size-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition:
        opacity 0.15s,
        background 0.15s;
}
.pw-preview-copy:hover {
    opacity: 1;
    background: var(--bg-hover);
    color: var(--accent);
}
.pw-status {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-muted);
    font-size: var(--font-size-base);
    min-height: 200px;
}
.pw-status--error {
    color: var(--danger);
}
.pw-status-text {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
}
.pw-spinner {
    width: 28px;
    height: 28px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: pw-spin 0.6s linear infinite;
    flex-shrink: 0;
}
@keyframes pw-spin {
    to {
        transform: rotate(360deg);
    }
}

/* ── Skeleton loading ── */
@keyframes pw-pulse {
    0%,
    100% {
        opacity: 0.4;
    }
    50% {
        opacity: 0.8;
    }
}
.pw-skeleton {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    width: 80%;
    max-width: 400px;
    animation: pw-pulse 1.5s ease-in-out infinite;
}
.pw-skeleton-line {
    height: 12px;
    background: var(--bg-hover);
    border-radius: 4px;
}
.pw-skeleton-line:nth-child(1) {
    width: 60%;
}
.pw-skeleton-line:nth-child(2) {
    width: 80%;
}
.pw-skeleton-line:nth-child(3) {
    width: 45%;
}
.pw-skeleton-block {
    height: 160px;
    background: var(--bg-hover);
    border-radius: 6px;
}
.pw-zoom-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.pw-zoom-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg-tertiary);
}
.pw-zoom-btn {
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
.pw-zoom-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.pw-zoom-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}
.pw-zoom-pct {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    min-width: 36px;
    text-align: center;
    font-variant-numeric: tabular-nums;
}
.pw-zoom-sep {
    width: 1px;
    height: 14px;
    background: var(--border);
    flex-shrink: 0;
}
.pw-zoom-scroll {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    background: var(--bg-primary);
}
.pw-zoom-scroll .pw-office {
    align-self: stretch;
    width: 100%;
    height: 100%;
}
.pw-image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    transition: transform 0.1s;
    transform-origin: center center;
}
.pw-office {
    flex: 1;
    overflow: auto;
}
.pw-code {
    flex: 1;
    overflow: auto;
}
.pw-markdown {
    padding: 12px 16px;
    font-size: var(--font-size-base);
    line-height: 1.6;
    overflow: auto;
    flex: 1;
    color: var(--text-primary);
}

/* ── Archive ── */
.pw-archive {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.pw-archive-hdr {
    padding: 6px 8px;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.pw-archive-list {
    flex: 1;
    overflow: auto;
    padding: 2px 0;
}
.pw-archive-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: var(--font-size-sm);
    white-space: nowrap;
    cursor: pointer;
    border-radius: 3px;
    margin: 0 2px;
}
.pw-archive-item:hover {
    background: var(--bg-hover);
}
.pw-archive-item--active {
    background: var(--bg-selected);
}
.pw-archive-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
}
.pw-archive-size {
    margin-left: auto;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    flex-shrink: 0;
}

/* ── Context menu ── */
.pw-ctx-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
}
.pw-ctx-menu {
    position: fixed;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 180px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    font-size: var(--font-size-sm);
}
.pw-ctx-item {
    display: block;
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
.pw-ctx-item:hover {
    background: var(--bg-hover);
}
.pw-ctx-sep {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
}
</style>
