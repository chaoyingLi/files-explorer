<template>
    <div class="xls-root">
        <div class="xls-toolbar">
            <div class="xls-toolbar-group">
                <button
                    class="xls-tb-btn"
                    :title="$t('properties.zoomOut')"
                    @click.stop="zoom = Math.max(0.5, +(zoom - 0.2).toFixed(1))"
                >
                    −
                </button>
                <span class="xls-zoom-pct">{{ Math.round(zoom * 100) }}%</span>
                <button
                    class="xls-tb-btn"
                    :title="$t('properties.zoomIn')"
                    @click.stop="zoom = Math.min(3, +(zoom + 0.2).toFixed(1))"
                >
                    +
                </button>
                <button
                    class="xls-tb-btn"
                    :title="$t('properties.zoomReset')"
                    @click.stop="zoom = 1"
                >
                    ⊡
                </button>
            </div>
        </div>
        <div ref="scrollEl" class="xls-scroll" @wheel.prevent="onWheel">
            <div
                class="xls-table-wrap"
                :style="{
                    transform: 'scale(' + zoom + ')',
                    transformOrigin: 'top left',
                }"
                v-html="tableHtml"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import * as XLSX from "xlsx";

const props = defineProps<{ data: ArrayBuffer }>();

const tableHtml = ref("");
const zoom = ref(1);
const scrollEl = ref<HTMLElement>();

function onWheel(e: WheelEvent) {
    if (!e.ctrlKey && !e.metaKey) return;
    zoom.value = Math.max(
        0.5,
        Math.min(3, +(zoom.value + (e.deltaY > 0 ? -0.2 : 0.2)).toFixed(1)),
    );
}

function render() {
    if (!props.data) return;
    try {
        const wb = XLSX.read(new Uint8Array(props.data), { type: "array" });
        const first = wb.SheetNames[0];
        const html = XLSX.utils.sheet_to_html(wb.Sheets[first], {
            id: "xls-table",
        });
        tableHtml.value = html;
    } catch (e) {
        tableHtml.value = `<p style="padding:24px;color:var(--danger)">Preview failed</p>`;
    }
}

onMounted(() => render());
watch(
    () => props.data,
    () => render(),
);
</script>

<style scoped>
.xls-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}
.xls-toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
    flex-shrink: 0;
}
.xls-toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
}
.xls-tb-btn {
    background: var(--bg-hover);
    border: 1px solid transparent;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: var(--font-size-base);
    line-height: 1.4;
    transition: all 0.15s;
}
.xls-tb-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.xls-tb-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
}
.xls-zoom-pct {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    min-width: 32px;
    text-align: center;
    font-variant-numeric: tabular-nums;
}
.xls-scroll {
    flex: 1;
    overflow: auto;
    padding: 12px;
    background: var(--bg-primary);
}
.xls-table-wrap {
    transition: transform 0.1s;
}
.xls-table-wrap :deep(table) {
    border-collapse: collapse;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
}
.xls-table-wrap :deep(td),
.xls-table-wrap :deep(th) {
    border: 1px solid var(--border);
    padding: 4px 8px;
    white-space: nowrap;
    text-align: left;
}
.xls-table-wrap :deep(th) {
    background: var(--bg-secondary);
    font-weight: 600;
}
</style>
