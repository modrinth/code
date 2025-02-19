<template>
  <div class="flex h-[calc(100vh-12rem)] w-full flex-col items-center">
    <div
      ref="container"
      class="relative w-full flex-grow cursor-grab overflow-hidden rounded-b-2xl bg-black active:cursor-grabbing"
      @mousedown="startPan"
      @mousemove="handlePan"
      @mouseup="stopPan"
      @mouseleave="stopPan"
      @wheel.prevent="handleWheel"
    >
      <div v-if="state.isLoading" />
      <div
        v-if="state.hasError"
        class="flex h-full w-full flex-col items-center justify-center gap-8"
      >
        <UiServersIconsPanelErrorIcon />
        <p class="m-0">{{ state.errorMessage || "Invalid or empty image file." }}</p>
      </div>
      <img
        v-show="isReady"
        ref="imageRef"
        :src="imageObjectUrl"
        class="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transform"
        :style="imageStyle"
        alt="Viewed image"
        @load="handleImageLoad"
        @error="handleImageError"
      />
    </div>
    <div
      v-if="!state.hasError"
      class="absolute bottom-0 mb-2 flex w-fit justify-center gap-2 space-x-4 rounded-2xl bg-bg p-2"
    >
      <ButtonStyled type="transparent" @click="zoom(ZOOM_IN_FACTOR)">
        <button v-tooltip="'Zoom in'">
          <ZoomInIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled type="transparent" @click="zoom(ZOOM_OUT_FACTOR)">
        <button v-tooltip="'Zoom out'">
          <ZoomOutIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled type="transparent" @click="reset">
        <button>
          <span class="font-mono">{{ Math.round(state.scale * 100) }}%</span>
          <span class="ml-4 text-sm text-blue">Reset</span>
        </button>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { ZoomInIcon, ZoomOutIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const ZOOM_MIN = 0.1;
const ZOOM_MAX = 5;
const ZOOM_IN_FACTOR = 1.2;
const ZOOM_OUT_FACTOR = 0.8;
const INITIAL_SCALE = 0.5;
const MAX_IMAGE_DIMENSION = 4096;

const props = defineProps<{
  imageBlob: Blob;
}>();

const state = ref({
  scale: INITIAL_SCALE,
  translateX: 0,
  translateY: 0,
  isPanning: false,
  startX: 0,
  startY: 0,
  isLoading: false,
  hasError: false,
  errorMessage: "",
});

const imageRef = ref<HTMLImageElement | null>(null);
const container = ref<HTMLElement | null>(null);
const imageObjectUrl = ref("");
const rafId = ref(0);

const isReady = computed(() => !state.value.isLoading && !state.value.hasError);

const imageStyle = computed(() => ({
  transform: `translate(-50%, -50%) scale(${state.value.scale}) translate(${state.value.translateX}px, ${state.value.translateY}px)`,
  transition: state.value.isPanning ? "none" : "transform 0.3s ease-out",
}));

const validateImageDimensions = (img: HTMLImageElement): boolean => {
  if (img.naturalWidth > MAX_IMAGE_DIMENSION || img.naturalHeight > MAX_IMAGE_DIMENSION) {
    state.value.hasError = true;
    state.value.errorMessage = `Image too large to view (max ${MAX_IMAGE_DIMENSION}x${MAX_IMAGE_DIMENSION} pixels)`;
    return false;
  }
  return true;
};

const updateImageUrl = (blob: Blob) => {
  if (imageObjectUrl.value) URL.revokeObjectURL(imageObjectUrl.value);
  imageObjectUrl.value = URL.createObjectURL(blob);
};

const handleImageLoad = () => {
  if (!imageRef.value || !validateImageDimensions(imageRef.value)) {
    state.value.isLoading = false;
    return;
  }
  state.value.isLoading = false;
  reset();
};

const handleImageError = () => {
  state.value.isLoading = false;
  state.value.hasError = true;
  state.value.errorMessage = "Failed to load image";
};

const zoom = (factor: number) => {
  const newScale = state.value.scale * factor;
  state.value.scale = Math.max(ZOOM_MIN, Math.min(newScale, ZOOM_MAX));
};

const reset = () => {
  state.value.scale = INITIAL_SCALE;
  state.value.translateX = 0;
  state.value.translateY = 0;
};

const startPan = (e: MouseEvent) => {
  state.value.isPanning = true;
  state.value.startX = e.clientX - state.value.translateX;
  state.value.startY = e.clientY - state.value.translateY;
};

const handlePan = (e: MouseEvent) => {
  if (!state.value.isPanning) return;
  cancelAnimationFrame(rafId.value);
  rafId.value = requestAnimationFrame(() => {
    state.value.translateX = e.clientX - state.value.startX;
    state.value.translateY = e.clientY - state.value.startY;
  });
};

const stopPan = () => {
  state.value.isPanning = false;
};

const handleWheel = (e: WheelEvent) => {
  const delta = e.deltaY * -0.001;
  const factor = 1 + delta;
  zoom(factor);
};

watch(
  () => props.imageBlob,
  (newBlob) => {
    if (!newBlob) return;
    state.value.isLoading = true;
    state.value.hasError = false;
    updateImageUrl(newBlob);
  },
);

onMounted(() => {
  if (props.imageBlob) updateImageUrl(props.imageBlob);
});

onUnmounted(() => {
  if (imageObjectUrl.value) URL.revokeObjectURL(imageObjectUrl.value);
  cancelAnimationFrame(rafId.value);
});
</script>
