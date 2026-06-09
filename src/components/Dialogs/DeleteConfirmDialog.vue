<template>
    <div class="dialog-overlay" @click.self="$emit('cancel')">
        <div class="dialog confirmation-dialog">
            <div class="confirm-icon" :class="permanently ? 'icon-danger' : 'icon-warning'">
                <svg viewBox="0 0 24 24" fill="none">
                    <path
                        d="M3 6h18M8 6V4a1 1 0 011-1h6a1 1 0 011 1v2"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                    />
                    <path
                        d="M5 6l1 13a2 2 0 002 2h8a2 2 0 002-2l1-13"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                    />
                </svg>
            </div>
            <h3>{{ permanently ? $t('dialogs.deletePermanentTitle') : $t('dialogs.deleteTitle') }}</h3>
            <p class="confirm-message">
                {{ $t('dialogs.deleteMessage', { count }) }}
            </p>
            <p v-if="!permanently" class="confirm-hint">
                {{ $t('dialogs.deleteHint') }}
            </p>
            <p v-else class="confirm-hint hint-danger">
                {{ $t('dialogs.deletePermanentWarning') }}
            </p>
            <div class="dialog-actions">
                <button class="confirm-cancel" @click="$emit('cancel')">
                    {{ $t('dialogs.cancel') }}
                </button>
                <button
                    :class="permanently ? 'danger' : 'accent'"
                    @click="$emit('confirm', permanently)"
                >
                    {{ permanently ? $t('dialogs.deletePermanent') : $t('dialogs.delete') }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
withDefaults(
    defineProps<{
        count: number;
        permanently: boolean;
    }>(),
    { count: 1, permanently: false },
);

defineEmits<{
    confirm: [permanently: boolean];
    cancel: [];
}>();
</script>

<style scoped>
.confirmation-dialog {
    max-width: 400px;
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
}

.confirm-icon svg {
    width: 26px;
    height: 26px;
}

.icon-warning {
    background: rgba(245, 197, 66, 0.15);
    color: var(--warning);
}

.icon-danger {
    background: rgba(243, 139, 168, 0.15);
    color: var(--danger);
}

h3 {
    margin-bottom: 8px;
    font-size: 16px;
    font-weight: 600;
}

.confirm-message {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 8px;
    line-height: 1.4;
}

.confirm-hint {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 16px;
    line-height: 1.4;
    padding: 8px 12px;
    background: var(--bg-hover);
    border-radius: 6px;
}

.hint-danger {
    color: var(--danger);
    background: rgba(243, 139, 168, 0.1);
    border: 1px solid rgba(243, 139, 168, 0.2);
}

.dialog-actions {
    display: flex;
    justify-content: center;
    gap: 10px;
}

.dialog-actions button {
    padding: 8px 24px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
}

.confirm-cancel {
    background: var(--bg-hover);
}
</style>
