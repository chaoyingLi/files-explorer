<template>
    <div
        ref="menuRef"
        class="context-menu"
        :style="{ left: adjX + 'px', top: adjY + 'px' }"
        @click.stop
    >
        <template v-for="(item, i) in items" :key="i">
            <div v-if="item.separator" class="context-menu-separator"></div>
            <div
                v-else-if="item.children"
                class="context-menu-item has-sub"
                @mouseenter="subId = i"
                @mouseleave="onSubLeave"
            >
                <span
                    v-if="item.icon"
                    class="menu-icon"
                    v-html="item.icon"
                ></span>
                <span>{{ item.label }}</span>
                <span class="arrow">▶</span>
                <div v-if="subId === i" class="submenu" @mouseenter="subId = i">
                    <div
                        v-for="child in item.children"
                        :key="child.action"
                        class="context-menu-item"
                        @click.stop="onChildClick(child.action)"
                    >
                        <span
                            v-if="child.icon"
                            class="menu-icon"
                            v-html="child.icon"
                        ></span>
                        <span>{{ child.label }}</span>
                    </div>
                </div>
            </div>
            <div
                v-else
                class="context-menu-item"
                :class="{ disabled: item.disabled }"
                @click="!item.disabled && onItemClick(item.action)"
            >
                <span
                    v-if="item.icon"
                    class="menu-icon"
                    v-html="item.icon"
                ></span>
                <span>{{ item.label }}</span>
                <span v-if="item.shortcut" class="shortcut">{{
                    item.shortcut
                }}</span>
            </div>
        </template>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { ContextMenuOption } from "@/types";

const props = defineProps<{
    x: number;
    y: number;
    items: ContextMenuOption[];
}>();
const emit = defineEmits<{ close: []; action: [action: string] }>();
const menuRef = ref<HTMLElement | null>(null);
const subId = ref<number | null>(null);

const adjX = computed(() => Math.min(props.x, window.innerWidth - 240));
const adjY = computed(() =>
    Math.min(props.y, window.innerHeight - (props.items.length * 32 + 20)),
);

function onItemClick(action: string) {
    emit("action", action);
}
function onChildClick(action: string) {
    subId.value = null;
    emit("action", action);
}
function onSubLeave() {
    setTimeout(() => {
        subId.value = null;
    }, 100);
}
</script>

<style scoped>
.context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    min-width: 200px;
    box-shadow: 0 8px 32px var(--shadow);
}
.context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--font-size-base);
    position: relative;
}
.menu-icon {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
}
.menu-icon :deep(svg) {
    width: 14px;
    height: 14px;
}
.context-menu-item:hover {
    background: var(--bg-hover);
}
.context-menu-item.disabled {
    color: var(--text-muted);
    cursor: default;
}
.context-menu-item.disabled:hover {
    background: transparent;
}
.context-menu-item .shortcut {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
}
.arrow {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin-left: 8px;
}
.context-menu-separator {
    height: 1px;
    background: var(--border);
    margin: 4px 8px;
}
.submenu {
    position: absolute;
    left: 100%;
    top: -4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    min-width: 140px;
    box-shadow: 0 4px 16px var(--shadow);
    z-index: 1001;
}
</style>
