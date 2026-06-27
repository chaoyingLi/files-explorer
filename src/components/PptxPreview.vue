<template>
    <div ref="container" class="pptx-preview-wrap" />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { init } from "pptx-preview";

const props = defineProps<{ data: ArrayBuffer }>();

const container = ref<HTMLElement>();
let previewer: any = null;

async function renderPptx() {
    if (!container.value || !props.data) return;
    // Destroy previous instance
    if (previewer) {
        try {
            previewer.destroy();
        } catch {
            /* ignore */
        }
        previewer = null;
    }
    container.value.innerHTML = "";
    try {
        previewer = init(container.value, {
            width: container.value.clientWidth || 680,
            height: 480,
            mode: "slide",
        });
        await previewer.preview(props.data);
    } catch {
        /* render failed */
    }
}

onMounted(() => {
    renderPptx();
});

watch(
    () => props.data,
    () => {
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
.pptx-preview-wrap {
    width: 100%;
    overflow: auto;
    max-height: 520px;
}
.pptx-preview-wrap :deep(canvas) {
    max-width: 100%;
    height: auto;
}
.pptx-preview-wrap :deep(button) {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
    border-radius: 4px;
}
.pptx-preview-wrap :deep(button:hover) {
    background: var(--accent);
    color: #fff;
}
</style>
