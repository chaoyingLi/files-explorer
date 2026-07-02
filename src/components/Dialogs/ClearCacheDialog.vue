<template>
    <div class="dialog-overlay" @click.self="$emit('cancel')">
        <div class="dialog confirmation-dialog">
            <div class="confirm-icon">
                <svg viewBox="0 0 24 24" fill="none">
                    <circle
                        cx="12"
                        cy="12"
                        r="9"
                        stroke="currentColor"
                        stroke-width="1.8"
                    />
                    <path
                        d="M8 8l8 8M16 8l-8 8"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                    />
                </svg>
            </div>
            <h3>{{ $t("tray.clearCache") }}</h3>
            <p class="confirm-message">
                {{ $t("tray.clearCacheBrief") }}
            </p>
            <div class="clear-cache-list">
                <div class="clear-cache-row">
                    <span class="clear-cache-dot"></span>
                    {{ $t("tray.clearItemLayout") }}
                </div>
                <div class="clear-cache-row">
                    <span class="clear-cache-dot"></span>
                    {{ $t("tray.clearItemSearch") }}
                </div>
                <div class="clear-cache-row">
                    <span class="clear-cache-dot"></span>
                    {{ $t("tray.clearItemVideo") }}
                </div>
                <div class="clear-cache-row">
                    <span class="clear-cache-dot"></span>
                    {{ $t("tray.clearItemWindow") }}
                </div>
                <div class="clear-cache-row">
                    <span class="clear-cache-dot"></span>
                    {{ $t("tray.clearItemBookmarks") }}
                </div>
            </div>
            <p class="confirm-hint">
                {{ $t("tray.clearCacheHint") }}
            </p>
            <div class="dialog-actions">
                <button class="confirm-cancel" @click="$emit('cancel')">
                    {{ $t("dialogs.cancel") }}
                </button>
                <button class="confirm-btn danger" @click="$emit('confirm')">
                    {{ $t("tray.clearCache") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
defineEmits<{ confirm: []; cancel: [] }>();
</script>

<style scoped>
.confirmation-dialog {
    max-width: 420px;
    text-align: center;
}

.confirm-icon {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 16px;
    background: rgba(243, 139, 168, 0.12);
    color: var(--danger);
}
.confirm-icon svg {
    width: 24px;
    height: 24px;
}

h3 {
    margin-bottom: 8px;
    font-size: var(--font-size-xl);
    font-weight: 600;
}

.confirm-message {
    font-size: var(--font-size-base);
    color: var(--text-secondary);
    margin-bottom: 12px;
    line-height: 1.5;
}

.clear-cache-list {
    text-align: left;
    margin: 0 auto 12px;
    padding: 10px 14px;
    background: var(--bg-tertiary);
    border-radius: 8px;
    display: inline-flex;
    flex-direction: column;
    gap: 6px;
}
.clear-cache-row {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 8px;
}
.clear-cache-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
    opacity: 0.7;
}

.confirm-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: 16px;
    line-height: 1.4;
    padding: 8px 12px;
    background: var(--bg-hover);
    border-radius: 6px;
}

.dialog-actions {
    display: flex;
    justify-content: center;
    gap: 10px;
}
.dialog-actions button {
    padding: 8px 24px;
    border-radius: 8px;
    font-size: var(--font-size-base);
    font-weight: 500;
}
.confirm-cancel {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
}
.confirm-cancel:hover {
    background: var(--bg-secondary);
}
</style>
