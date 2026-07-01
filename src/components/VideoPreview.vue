<template>
    <div class="video-root" :class="{ 'video-minimal': minimalist }">
        <div v-if="loading && !loadError" class="video-status">
            <span class="video-spinner"></span>
        </div>
        <div v-if="loadError" class="video-status">
            <span class="video-error-text">{{ loadError }}</span>
        </div>
        <div ref="containerRef" class="video-container"></div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import DPlayer from "dplayer";

const { t } = useI18n();

const props = defineProps<{ src: string; minimalist?: boolean }>();

const containerRef = ref<HTMLElement | null>(null);
const loading = ref(true);
const loadError = ref("");

let _dp: any = null;
let _saveInterval: ReturnType<typeof setInterval> | null = null;
let _destroying = false;

const POS_KEY = computed(() => `dp-pos:${props.src}`);
const isFlv = computed(() => props.src.toLowerCase().endsWith(".flv"));

function initPlayer() {
    if (!containerRef.value) return;
    destroyPlayer();

    const videoType = isFlv.value ? "flv" : "auto";
    const accent =
        getComputedStyle(document.documentElement)
            .getPropertyValue("--accent")
            .trim() || "#20DDCC";

    // 生成视频封面 SVG（深色背景 + 播放图标）
    const posterSvg = `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360">
        <rect width="640" height="360" fill="#1a1a2e"/>
        <circle cx="320" cy="180" r="52" fill="none" stroke="${accent}" stroke-width="2.5" opacity="0.6"/>
        <polygon points="305,155 305,205 350,180" fill="${accent}" opacity="0.85"/>
    </svg>`;
    const posterUrl = `data:image/svg+xml,${encodeURIComponent(posterSvg)}`;

    _dp = new DPlayer({
        container: containerRef.value,
        video: {
            url: props.src,
            pic: posterUrl,
            type: videoType as any,
        },
        autoplay: false,
        theme: accent,
        hotkey: true,
        loop: false,
        playbackSpeed: [0.5, 0.75, 1, 1.25, 1.5, 2],
        screenshot: true,
        lang: "zh-cn",
    });

    // 恢复播放位置
    _dp.on("loadedmetadata", () => {
        loading.value = false;
        loadError.value = "";
        try {
            const v = localStorage.getItem(POS_KEY.value);
            if (v && _dp) _dp.seek(parseFloat(v));
        } catch {
            /* */
        }
    });

    _dp.on("error", () => {
        if (_destroying) return;
        loading.value = false;
        loadError.value = t("properties.videoError");
    });

    // 定时保存播放位置
    _saveInterval = setInterval(savePosition, 3000);
    _dp.on("pause", savePosition);
}

function savePosition() {
    if (!_dp?.video) return;
    try {
        const t = _dp.video.currentTime;
        if (t > 0) localStorage.setItem(POS_KEY.value, String(t));
    } catch {
        /* */
    }
}

function destroyPlayer() {
    _destroying = true;
    savePosition();
    if (_saveInterval) {
        clearInterval(_saveInterval);
        _saveInterval = null;
    }
    if (_dp) {
        try {
            _dp.pause();
            _dp.destroy();
        } catch {
            /* */
        }
        _dp = null;
    }
    _destroying = false;
}

watch(
    () => props.src,
    () => {
        loading.value = true;
        loadError.value = "";
        initPlayer();
    },
    { immediate: true },
);

onMounted(() => {
    // DPlayer 需要在 DOM 挂载后初始化，但 watch immediate 可能更早
    if (!_dp) initPlayer();
});

onUnmounted(() => {
    destroyPlayer();
});

defineExpose({ destroyPlayer });
</script>

<style>
/* ⚠️ 全局样式：DPlayer 在 scoped 之外 */
.video-root {
    width: 100%;
    height: 100%;
    min-height: 180px;
    background: #000;
    position: relative;
    display: flex;
    flex-direction: column;
}
.video-root.video-minimal {
    min-height: 160px;
    max-height: 360px;
}
.video-container {
    flex: 1;
    min-height: 0;
}
.video-container .dplayer {
    height: 100% !important;
}
/* Minimal 模式精简 */
.video-root.video-minimal .dplayer-controller .dplayer-setting,
.video-root.video-minimal .dplayer-controller .dplayer-full {
    display: none !important;
}
/* 主题色 */
.video-root .dplayer-bar-preview-inner,
.video-root .dplayer-played {
    background: var(--accent) !important;
}
.video-root .dplayer-volume-bar-inner {
    background: var(--accent) !important;
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
