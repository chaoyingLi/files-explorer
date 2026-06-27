<template>
    <div class="status-bar">
        <div class="status-left">
            <span>{{ statusText }}</span>
        </div>
        <div class="status-right">
            <button
                class="status-btn"
                :title="$t('properties.title')"
                @click="$emit('toggleProperties')"
            >
                ⓘ
            </button>
            <span v-if="store.loading" class="loading-indicator">
                <div
                    class="loading-spinner"
                    style="width: 14px; height: 14px; border-width: 1.5px"
                ></div>
            </span>
            <span v-else>{{ itemCount }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";

defineEmits<{ toggleProperties: [] }>();

const { t } = useI18n();
const store = useFileStore();

const statusText = computed(() => {
    if (!store.currentPath) return t("statusBar.thisPc");
    return store.currentPath;
});

const itemCount = computed(() => {
    if (!store.currentPath) {
        return `${store.drives.length} ${t("statusBar.drives")}`;
    }
    return t("statusBar.items", { count: store.files.length });
});
</script>

<style scoped>
.status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-muted);
    height: 24px;
    flex-shrink: 0;
}

.status-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
}

.status-left span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.status-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
}

.loading-indicator {
    display: flex;
    align-items: center;
}

.status-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 3px;
    font-size: 14px;
    line-height: 1;
}

.status-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}
</style>
