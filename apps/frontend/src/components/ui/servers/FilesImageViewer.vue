<template>
  <div class="flex h-[calc(100vh-12rem)] w-full flex-col items-center bg-bg-raised">
    <div
      ref="container"
      class="relative w-full flex-grow overflow-hidden bg-bg-raised"
      @mousedown="startPan"
      @mousemove="pan"
      @mouseup="endPan"
      @mouseleave="endPan"
      @wheel.prevent="handleWheel"
    >
      <UiServersPyroLoading v-if="loading" />
      <img
        v-show="!loading"
        ref="image"
        :src="imageUrl"
        class="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transform"
        :style="{
          transform: `translate(-50%, -50%) scale(${scale}) translate(${translateX}px, ${translateY}px)`,
          transition: isPanning ? 'none' : 'transform 0.3s ease-out',
        }"
        alt="Viewed image"
        @load="onImageLoad"
      />
    </div>
    <div class="absolute bottom-0 mb-2 flex w-fit justify-center space-x-4 rounded-xl bg-bg p-2">
      <Button icon-only transparent @click="zoomIn">
        <ZoomInIcon />
      </Button>
      <Button icon-only transparent @click="resetZoom">
        <HomeIcon />
      </Button>
      <Button icon-only transparent @click="zoomOut">
        <ZoomOutIcon />
      </Button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from "vue";
import { HomeIcon, ZoomInIcon, ZoomOutIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";

const props = defineProps({
  imageBlob: {
    type: Blob,
    required: true,
  },
});

const container = ref(null);
const image = ref(null);
const scale = ref(1);
const translateX = ref(0);
const translateY = ref(0);
const isPanning = ref(false);
const startX = ref(0);
const startY = ref(0);
const imageUrl = ref("");
const loading = ref(true);

const createImageUrl = (blob) => {
  if (imageUrl.value) {
    URL.revokeObjectURL(imageUrl.value);
  }
  imageUrl.value = URL.createObjectURL(blob);
};

watch(
  () => props.imageBlob,
  (newBlob) => {
    if (newBlob) {
      loading.value = true;
      createImageUrl(newBlob);
    }
  },
);

onMounted(() => {
  if (props.imageBlob) {
    createImageUrl(props.imageBlob);
  }
});

const onImageLoad = () => {
  loading.value = false;
  resetZoom();
};

const zoomIn = () => {
  scale.value = Math.min(scale.value * 1.2, 5);
};

const zoomOut = () => {
  scale.value = Math.max(scale.value / 1.2, 0.1);
};

const resetZoom = () => {
  scale.value = 0.5;
  translateX.value = 0;
  translateY.value = 0;
};

const startPan = (e) => {
  isPanning.value = true;
  startX.value = e.clientX - translateX.value;
  startY.value = e.clientY - translateY.value;
};

const pan = (e) => {
  if (isPanning.value) {
    translateX.value = e.clientX - startX.value;
    translateY.value = e.clientY - startY.value;
  }
};

const endPan = () => {
  isPanning.value = false;
};

const handleWheel = (e) => {
  const delta = (e.deltaY * -0.01) / 10;
  const newScale = Math.max(0.1, Math.min(scale.value + delta, 5));

  scale.value = newScale;
};
</script>
