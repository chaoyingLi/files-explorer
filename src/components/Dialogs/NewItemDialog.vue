<template>
    <div class="dialog-overlay" @click.self="$emit('close')">
        <div class="dialog">
            <h3>
                {{ isFolder ? t("dialogs.newFolder") : t("dialogs.newFile") }}
            </h3>
            <input
                ref="inputRef"
                v-model="name"
                :placeholder="
                    isFolder ? t('dialogs.folderName') : t('dialogs.fileName')
                "
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
                    :disabled="!name.trim()"
                >
                    {{ t("dialogs.create") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
    defineProps<{
        type?: string;
    }>(),
    { type: "folder" },
);

const emit = defineEmits<{
    close: [];
    confirm: [name: string, type: string];
}>();

const { t } = useI18n();
const isFolder = ref(props.type === "folder");
const name = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

function confirm() {
    if (name.value.trim()) {
        emit("confirm", name.value.trim(), isFolder.value ? "folder" : "file");
    }
}

onMounted(() => {
    inputRef.value?.focus();
});
</script>
