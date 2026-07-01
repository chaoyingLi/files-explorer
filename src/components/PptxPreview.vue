<template>
    <div class="pptx-root">
        <div class="pptx-toolbar">
            <div class="pptx-toolbar-group">
                <button
                    class="pptx-tb-btn"
                    :class="{ active: mode === 'slide' }"
                    @click.stop="switchMode('slide')"
                    title="Slide view"
                >
                    📄
                </button>
                <button
                    class="pptx-tb-btn"
                    :class="{ active: mode === 'list' }"
                    @click.stop="switchMode('list')"
                    title="Vertical list"
                >
                    📋
                </button>
            </div>
            <div class="pptx-toolbar-sep"></div>
            <div class="pptx-toolbar-group">
                <button
                    class="pptx-tb-btn"
                    :disabled="zoom <= 0.25"
                    @click.stop="
                        zoom = Math.max(0.25, +(zoom - 0.1).toFixed(1))
                    "
                    title="Zoom out"
                >
                    −
                </button>
                <span class="pptx-zoom-pct">{{ Math.round(zoom * 100) }}%</span>
                <button
                    class="pptx-tb-btn"
                    :disabled="zoom >= 3"
                    @click.stop="zoom = Math.min(3, +(zoom + 0.1).toFixed(1))"
                    title="Zoom in"
                >
                    +
                </button>
                <button
                    class="pptx-tb-btn"
                    @click.stop="zoom = 1"
                    title="Reset zoom"
                >
                    ⊡
                </button>
            </div>
        </div>
        <div ref="container" class="pptx-preview-wrap" @wheel="onWheel">
            <div
                class="pptx-zoom-layer"
                :style="{
                    transform: 'scale(' + zoom + ')',
                    transformOrigin: 'top center',
                }"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from "vue";
import { init } from "pptx-preview";

const props = defineProps<{ data: ArrayBuffer }>();

const container = ref<HTMLElement>();
const mode = ref<"slide" | "list">("slide");
const zoom = ref(1);
let previewer: any = null;

function switchMode(newMode: "slide" | "list") {
    if (mode.value === newMode) return;
    mode.value = newMode;
    zoom.value = 1;
    renderPptx();
}

function onWheel(e: WheelEvent) {
    // Only zoom when Ctrl is held — allows normal scroll for slide navigation
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    zoom.value = Math.max(0.25, Math.min(3, +(zoom.value + delta).toFixed(1)));
}

async function renderPptx() {
    if (!container.value || !props.data) return;
    if (previewer) {
        try {
            previewer.destroy();
        } catch {
            /* ignore */
        }
        previewer = null;
    }
    // Clear only the zoom-layer, not the container itself
    const layer = container.value.querySelector(
        ".pptx-zoom-layer",
    ) as HTMLElement;
    if (!layer) return;
    layer.innerHTML = "";
    try {
        const opts: any = {
            width: container.value.clientWidth - 4 || 680,
            mode: mode.value,
        };
        if (mode.value === "slide") {
            opts.height = 480;
        }
        previewer = init(layer, opts);
        const uint8 = new Uint8Array(props.data);
        await previewer.preview(uint8);
    } catch (e) {
        console.error("PPTX preview failed:", e);
        if (layer) {
            layer.innerHTML = '<div class="pptx-fallback">Preview failed</div>';
        }
    }
}

onMounted(() => {
    renderPptx();
});

watch(
    () => props.data,
    () => {
        zoom.value = 1;
        renderPptx();
    },
);

onUnmounted(() => {
    if (previewer) {
        try {
            previewer.destroy();
        } catch {
            /* ignore */
        }
        previewer = null;
    }
});
</script>

<style scoped>
.pptx-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}
.pptx-toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
    flex-shrink: 0;
}
.pptx-toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
}
.pptx-toolbar-sep {
    width: 1px;
    height: 16px;
    background: var(--border);
    flex-shrink: 0;
}
.pptx-tb-btn {
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
.pptx-tb-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.pptx-tb-btn.active {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
}
.pptx-tb-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
}
.pptx-zoom-pct {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    min-width: 32px;
    text-align: center;
    font-variant-numeric: tabular-nums;
}
.pptx-preview-wrap {
    flex: 1;
    overflow: auto;
    display: flex;
    justify-content: center;
    padding: 12px 0;
    background: var(--bg-primary);
    position: relative;
}
.pptx-zoom-layer {
    transition: transform 0.1s;
    transform-origin: top center;
}
.pptx-zoom-layer :deep(canvas) {
    max-width: 100%;
    height: auto;
    display: block;
}
.pptx-zoom-layer :deep(button) {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
    border-radius: 4px;
}
.pptx-zoom-layer :deep(button:hover) {
    background: var(--accent);
    color: #fff;
}
</style>
