<template>
    <!-- 更新可用对话框 -->
    <div
        v-if="updateAvailable"
        class="updater-overlay"
        @click.self="dismissUpdate"
    >
        <div class="updater-dialog">
            <h3>
                {{ t("updater.available", { version: updateInfo?.version }) }}
            </h3>
            <div v-if="updateInfo?.body" class="updater-body">
                {{ updateInfo.body }}
            </div>
            <p>{{ t("updater.updateDialogDescription") }}</p>
            <div class="updater-actions">
                <button
                    class="btn btn-secondary"
                    :disabled="installing"
                    @click="dismissUpdate"
                >
                    {{ t("updater.updateLater") }}
                </button>
                <button
                    class="btn btn-primary"
                    :disabled="installing"
                    @click="handleInstall"
                >
                    {{
                        installing
                            ? t("updater.installing")
                            : t("updater.updateNow")
                    }}
                </button>
            </div>
        </div>
    </div>

    <!-- 重启对话框 -->
    <div
        v-if="showRestart"
        class="updater-overlay"
        @click.self="dismissRestart"
    >
        <div class="updater-dialog">
            <h3>{{ t("updater.restartPromptTitle") }}</h3>
            <p>{{ t("updater.restartPromptDescription") }}</p>
            <div class="updater-actions">
                <button class="btn btn-secondary" @click="dismissRestart">
                    {{ t("updater.restartLater") }}
                </button>
                <button class="btn btn-primary" @click="relaunchNow">
                    {{ t("updater.restartNow") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import {
    checkForUpdates,
    downloadSilently,
    installDownloadedUpdate,
    relaunchAfterUpdate,
    subscribeUpdateTask,
    enableMock,
    disableMock,
    type AvailableUpdate,
} from "@/utils/updater";

const { t } = useI18n();
const toast = useToast();

const emit = defineEmits<{
    noUpdate: [];
    error: [message: string];
}>();

const updateAvailable = ref(false);
const updateInfo = ref<AvailableUpdate | null>(null);
const showRestart = ref(false);
const installing = ref(false);

onMounted(async () => {
    try {
        const result = await checkForUpdates();
        if (result.state === "available" && result.update) {
            updateInfo.value = result.update;
            updateAvailable.value = true;
        }
    } catch (error) {
        emit("error", error instanceof Error ? error.message : String(error));
    }

    // 注册更新任务状态监听
    const unsub = subscribeUpdateTask((snapshot) => {
        if (snapshot.state === "ready_to_restart") {
            updateAvailable.value = false;
            showRestart.value = true;
        }
        if (snapshot.state === "error") {
            toast.show(snapshot.message || t("updater.updateFailed"), true);
            updateAvailable.value = false;
        }
    });
    onUnmounted(unsub);

    // 监听来自设置页的手动检查事件
    const onManualCheck = (e: Event) => {
        const detail = (e as CustomEvent).detail;
        if (detail?.version) {
            updateInfo.value = { version: detail.version, body: detail.body };
            updateAvailable.value = true;
        }
    };
    window.addEventListener("updater:show", onManualCheck);
    onUnmounted(() => window.removeEventListener("updater:show", onManualCheck));

    // Dev 模式挂载测试辅助
    if (import.meta.env.DEV) {
        const win = window as unknown as {
            __updaterTest?: {
                enableMock: typeof enableMock;
                disableMock: typeof disableMock;
                checkNow: () => Promise<void>;
                mockAvailable: () => void;
                mockNoUpdate: () => void;
                mockError: () => void;
            };
        };

        win.__updaterTest = {
            enableMock,
            disableMock,
            checkNow: async () => {
                console.log("[Updater Test] 手动触发检查...");
                const result = await checkForUpdates();
                if (result.state === "available" && result.update) {
                    updateInfo.value = result.update;
                    updateAvailable.value = true;
                }
            },
            mockAvailable: () => {
                enableMock("available");
                console.log("%c[Updater Test] 模拟：有可用更新（一次性）", "color: #4CAF50");
            },
            mockNoUpdate: () => {
                enableMock("no_update");
                console.log("%c[Updater Test] 模拟：无更新（一次性）", "color: #2196F3");
            },
            mockError: () => {
                enableMock("error");
                console.log("%c[Updater Test] 模拟：检查失败（一次性）", "color: #f44336");
            },
        };
        console.log("%c[Updater] 测试辅助已挂载到 window.__updaterTest", "color: #4CAF50");
    }
});

function dismissUpdate() {
    updateAvailable.value = false;
}

function dismissRestart() {
    showRestart.value = false;
}

async function handleInstall() {
    if (installing.value) return;
    installing.value = true;
    // Phase 1: 先下载
    const dl = downloadSilently();
    if (!dl.started) {
        toast.show(dl.reason === "ALREADY_DOWNLOADED"
            ? t("updater.installing")
            : t("updater.updateInProgress"), true);
        // 如果已下载，直接安装
        if (dl.reason === "ALREADY_DOWNLOADED") {
            const inst = installDownloadedUpdate();
            if (!inst.started) {
                toast.show(t("updater.updateFailed"), true);
                installing.value = false;
            }
        } else {
            installing.value = false;
        }
    }
    // 下载启动后，监听完成自动安装
    const unsubInstaller = subscribeUpdateTask((snapshot) => {
        if (snapshot.state === "ready_to_restart") {
            unsubInstaller();
            // 下载完成，执行安装
            const inst = installDownloadedUpdate();
            if (!inst.started) {
                toast.show(t("updater.updateFailed"), true);
                installing.value = false;
            }
        }
        if (snapshot.state === "error") {
            unsubInstaller();
            toast.show(snapshot.message || t("updater.updateFailed"), true);
            updateAvailable.value = false;
            installing.value = false;
        }
    });
}

async function relaunchNow() {
    await relaunchAfterUpdate();
}
</script>

<style scoped>
/* ── 下载提示 ── */
.updater-notice {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9998;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    box-shadow: 0 2px 12px rgba(0,0,0,0.15);
}

.updater-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
}
.updater-dialog {
    background: var(--bg-primary, #1e1e2e);
    color: var(--text-primary, #cdd6f4);
    border-radius: 8px;
    padding: 24px;
    max-width: 420px;
    width: 90%;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
.updater-dialog h3 {
    margin: 0 0 12px;
    font-size: 16px;
}
.updater-body {
    max-height: 160px;
    overflow-y: auto;
    background: var(--bg-secondary, #181825);
    border-radius: 4px;
    padding: 8px;
    margin-bottom: 12px;
    font-size: 12px;
    white-space: pre-wrap;
}
.updater-dialog p {
    margin: 0 0 16px;
    font-size: 13px;
    opacity: 0.8;
}
.updater-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
}
.updater-actions button {
    padding: 6px 16px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 13px;
}
.btn-secondary {
    background: var(--bg-secondary, #45475a);
    color: var(--text-primary, #cdd6f4);
}
.btn-primary {
    background: var(--accent, #89b4fa);
    color: #1e1e2e;
}
.btn-primary:disabled,
.btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}
.spinner {
    animation: spin 0.8s linear infinite;
}
</style>
