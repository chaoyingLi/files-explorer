<template>
  <div class="properties-panel" v-if="visible">
    <div class="props-header">
      <span>{{ $t("contextMenu.properties") }}</span>
      <button class="props-close" @click="$emit('close')">✕</button>
    </div>
    <div class="props-body" v-if="file">
      <!-- File icon & name -->
      <div class="props-icon-row">
        <div class="props-icon" v-html="fileIcon"></div>
        <div class="props-name">{{ file.name }}</div>
      </div>

      <div class="props-section">
        <div class="props-label">{{ $t("fileList.type") }}</div>
        <div class="props-value">{{ fileType }}</div>
      </div>
      <div class="props-section">
        <div class="props-label">{{ $t("fileList.size") }}</div>
        <div class="props-value">{{ formatSize(file.size) }}</div>
      </div>
      <div class="props-section" v-if="file.is_dir">
        <div class="props-label">Contents</div>
        <div class="props-value">{{ dirItemCount }} items</div>
      </div>
      <div class="props-section">
        <div class="props-label">{{ $t("fileList.dateModified") }}</div>
        <div class="props-value">{{ formatDate(file.modified) }}</div>
      </div>
      <div class="props-section">
        <div class="props-label">{{ $t("fileList.dateCreated") }}</div>
        <div class="props-value">{{ formatDate(file.created) }}</div>
      </div>
      <div class="props-section">
        <div class="props-label">Full Path</div>
        <div class="props-value props-path">{{ file.path }}</div>
      </div>

      <!-- Image dimensions -->
      <div class="props-section" v-if="imageInfo">
        <div class="props-label">Dimensions</div>
        <div class="props-value">{{ imageInfo.width }} × {{ imageInfo.height }}</div>
      </div>
    </div>

    <!-- Multi-selection summary -->
    <div class="props-body" v-else-if="multiCount > 1">
      <div class="props-icon-row">
        <div class="props-name">{{ multiCount }} items selected</div>
      </div>
      <div class="props-section">
        <div class="props-label">Total Size</div>
        <div class="props-value">{{ formatSize(totalSize) }}</div>
      </div>
    </div>

    <div class="props-body props-empty" v-else>
      <span>No file selected</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import type { FileEntry } from "@/types";
import { getFileIconSvg } from "@/utils/fileIcons";
import { formatFileSize } from "@/utils/fileTypes";

const { t } = useI18n();
const store = useFileStore();
const sel = useSelectionStore();

defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();

const imageInfo = ref<{ width: number; height: number } | null>(null);
const dirItemCount = ref(0);

const file = computed<FileEntry | null>(() => {
  if (sel.selectedFiles.size !== 1) return null;
  const path = [...sel.selectedFiles][0];
  return store.files.find(f => f.path === path) || null;
});

const multiCount = computed(() => sel.selectedFiles.size);

const totalSize = computed(() => {
  let total = 0;
  for (const p of sel.selectedFiles) {
    const f = store.files.find(x => x.path === p);
    if (f) total += f.size;
  }
  return total;
});

watch(file, async (f) => {
  imageInfo.value = null;
  dirItemCount.value = 0;
  if (!f) return;

  // Try to load image dimensions via base64
  const imgExts = ["png","jpg","jpeg","gif","webp","bmp","svg","ico"];
  if (imgExts.includes(f.extension.toLowerCase()) && !f.is_dir) {
    try {
      const { getFileBase64 } = await import("@/utils/tauri");
      const result = await getFileBase64(f.path);
      const img = new Image();
      img.onload = () => {
        imageInfo.value = { width: img.naturalWidth, height: img.naturalHeight };
      };
      img.src = `data:${result.mime};base64,${result.data}`;
    } catch {}
  }

  // Count dir items
  if (f.is_dir) {
    try {
      const { listDirectory } = await import("@/utils/tauri");
      const items = await listDirectory(f.path);
      dirItemCount.value = items.length;
    } catch {}
  }
});

const defaultFileIcon = `<svg viewBox="0 0 24 24"><path d="M6.5 2.5h6.8l5.2 5.2V19.5a2 2 0 01-2 2H6.5a2 2 0 01-2-2V4.5a2 2 0 012-2z" fill="currentColor" opacity="0.3"/><path d="M13.3 2.5v4.2c0 .55.45 1 1 1H18" fill="currentColor" opacity="0.5"/></svg>`;

const fileIcon = computed(() => {
  return file.value
    ? (getFileIconSvg(file.value.extension, file.value.is_dir) || defaultFileIcon)
    : defaultFileIcon;
});

const fileType = computed(() => {
  if (!file.value) return "";
  if (file.value.is_dir) return t("fileTypes.folder");
  const ext = file.value.extension.toUpperCase();
  return ext ? `${ext} File` : t("fileTypes.file");
});

function formatSize(bytes: number): string {
  return formatFileSize(bytes);
}

function formatDate(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString() + " " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}
</script>

<style scoped>
.properties-panel {
  width: 260px;
  flex-shrink: 0;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-size: var(--font-size-base);
}
.props-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-weight: 600;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border);
}
.props-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 4px;
}
.props-close:hover { background: var(--bg-hover); }
.props-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}
.props-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}
.props-icon-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}
.props-icon { width: 32px; height: 32px; display: flex; align-items: center; }
.props-icon :deep(svg) { width: 32px; height: 32px; }
.props-name { font-weight: 500; word-break: break-all; }
.props-section { margin-bottom: 12px; }
.props-label { font-size: var(--font-size-xs); color: var(--text-muted); margin-bottom: 2px; }
.props-value { color: var(--text-primary); }
.props-path { font-size: var(--font-size-xs); word-break: break-all; color: var(--text-secondary); }
</style>
