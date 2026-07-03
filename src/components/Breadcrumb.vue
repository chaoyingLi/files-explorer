<template>
    <div class="breadcrumb">
        <div class="breadcrumb-items">
            <span
                v-if="!path"
                class="breadcrumb-item"
                @click="$emit('navigate', '')"
            >
                <span
                    class="breadcrumb-label"
                    style="color: var(--text-primary); font-weight: 500"
                >
                    {{ t("sidebar.thisPc") }}
                </span>
            </span>
            <span
                v-for="(seg, i) in segments"
                :key="i"
                class="breadcrumb-item"
                @click="$emit('navigate', seg.path)"
            >
                <span v-if="i > 0" class="breadcrumb-sep">
                    <svg viewBox="0 0 16 16" width="12" height="12">
                        <path fill="currentColor" d="M6 3l5 5-5 5" />
                    </svg>
                </span>
                <span
                    class="breadcrumb-label"
                    :class="{ 'last-segment': i === segments.length - 1 }"
                    >{{ seg.name }}</span
                >
            </span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps<{
    path: string;
}>();

defineEmits<{
    navigate: [path: string];
}>();

const segments = computed(() => {
    if (!props.path) return [];
    const parts = props.path.replace(/\\/g, "/").split("/").filter(Boolean);
    const result: { name: string; path: string }[] = [];
    let accumulated = "";

    if (props.path.match(/^[A-Za-z]:/)) {
        accumulated = parts[0] + "/";
        result.push({ name: parts[0], path: accumulated });
        parts.shift();
    } else {
        result.push({ name: "/", path: "/" });
    }

    for (const part of parts) {
        if (accumulated && !accumulated.endsWith("/")) {
            accumulated += "/";
        }
        accumulated += part;
        result.push({ name: part, path: accumulated });
    }

    return result;
});
</script>

<style scoped>
.breadcrumb {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border);
    min-height: 32px;
    overflow-x: auto;
}

.breadcrumb-items {
    display: flex;
    align-items: center;
    gap: 2px;
    white-space: nowrap;
}

.breadcrumb-item {
    display: flex;
    align-items: center;
    gap: 2px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: var(--font-size-base);
}

.breadcrumb-item:hover {
    background: var(--bg-hover);
}

.breadcrumb-sep {
    color: var(--text-muted);
    display: flex;
    align-items: center;
}

.breadcrumb-label {
    color: var(--text-secondary);
}

.breadcrumb-item:last-child .breadcrumb-label,
.breadcrumb-label.last-segment {
    color: var(--text-primary);
    font-weight: 500;
}
</style>
