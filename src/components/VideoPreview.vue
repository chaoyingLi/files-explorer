<template>
    <div class="video-root" :class="{ 'video-minimal': minimalist }">
        <div v-if="loading && !loadError" class="video-status">
            <span class="video-spinner"></span>
        </div>
        <div v-if="loadError" class="video-status">
            <span class="video-error-text">{{ loadError }}</span>
        </div>
        <xgplayer
            v-show="!loadError"
            :key="playerKey"
            :config="playerConfig"
            @player="onPlayerReady"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
// @ts-ignore
import Xgplayer from "xgplayer-vue";
// @ts-ignore
import FlvPlugin from "xgplayer-flv";

const props = defineProps<{
    src: string;
    minimalist?: boolean;
}>();

const loading = ref(true);
const loadError = ref("");
const playerKey = ref(0);

const isFlv = computed(() => props.src.toLowerCase().endsWith(".flv"));

const playerConfig = computed(() => ({
    id: "mse-" + playerKey.value,
    url: props.src,
    controls: true,
    autoplay: false,
    playsinline: true,
    plugins: isFlv.value ? [FlvPlugin] : [],
}));

function onPlayerReady(player: any) {
    if (player) {
        loading.value = false;
        loadError.value = "";
    }
}

watch(
    () => props.src,
    () => {
        loading.value = true;
        loadError.value = "";
        playerKey.value++;
    },
);
</script>

<style>
.video-root {
    display: flex;
    flex-direction: column;
    background: #000;
    position: relative;
    height: 100%;
    min-height: 180px;
}
.video-root.video-minimal {
    height: 100%;
    min-height: 160px;
    max-height: 360px;
}
/* Override xgplayer inline styles so it fills the flex parent */
.video-root .xgplayer {
    flex: 1 1 auto !important;
    width: 100% !important;
    height: 100% !important;
    min-height: 0 !important;
    background: #000;
}
/* Ensure the inner <video> element is visible */
.video-root video,
.video-root .xgplayer video {
    display: block !important;
    width: 100% !important;
    height: 100% !important;
    object-fit: contain;
}
.video-root .xgplayer-skin-default .xgplayer-controls {
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
}
.video-root .xgplayer-progress-played {
    background: var(--accent) !important;
}
.video-root .xgplayer-volume-fill {
    background: var(--accent) !important;
}
.video-root.video-minimal .xgplayer-volume,
.video-root.video-minimal .xgplayer-fullscreen,
.video-root.video-minimal .xgplayer-playbackrate,
.video-root.video-minimal .xgplayer-pip {
    display: none !important;
}
</style>

<style scoped>
.video-status {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
}
.video-spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: video-spin 0.6s linear infinite;
}
@keyframes video-spin {
    to {
        transform: rotate(360deg);
    }
}
.video-error-text {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    text-align: center;
    padding: 16px;
}
</style>
