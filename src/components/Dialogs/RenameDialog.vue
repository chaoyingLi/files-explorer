<template>
    <div class="dialog-overlay" @click.self="$emit('close')">
        <div class="dialog">
            <h3>{{ t("dialogs.renameTitle") }}</h3>
            <input
                ref="inputRef"
                v-model="newName"
                @keydown.enter="confirm"
                @keydown.escape="$emit('close')"
            />
            <div class="dialog-actions">
                <button @click="$emit('close')">
                    {{ t("dialogs.cancel") }}
                </button>
                <button
                    class="accent"
                    @click="confirm"
                    :disabled="!newName.trim() || newName === oldName"
                >
                    {{ t("dialogs.rename") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
    oldName: string;
}>();

const emit = defineEmits<{
    close: [];
    confirm: [newName: string];
}>();

const { t } = useI18n();
const newName = ref(props.oldName || "");
const inputRef = ref<HTMLInputElement | null>(null);

function confirm() {
    if (newName.value.trim() && newName.value !== props.oldName) {
        emit("confirm", newName.value.trim());
    }
}

onMounted(() => {
    inputRef.value?.focus();
    const dotIndex = props.oldName.lastIndexOf(".");
    if (dotIndex > 0 && inputRef.value) {
        inputRef.value.setSelectionRange(0, dotIndex);
    } else {
        inputRef.value?.select();
    }
});
</script>
