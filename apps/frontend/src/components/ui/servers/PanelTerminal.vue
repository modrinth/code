<template>
  <div
    :class="[
      'terminal-font console relative flex h-full w-full flex-col items-center justify-between overflow-hidden rounded-xl pb-4 text-sm transition-transform duration-300',
      { 'scale-fullscreen fixed inset-0 z-50 !rounded-none': isFullScreen },
    ]"
    tabindex="-1"
  >
    <div
      ref="scrollContainer"
      data-pyro-terminal-root
      class="w-full select-text overflow-x-auto overflow-y-auto py-6"
      @scroll="handleScroll"
    >
      <div data-pyro-terminal-virtual-height-watcher :style="{ height: `${totalHeight}px` }">
        <ul
          class="m-0 select-text list-none p-0"
          data-pyro-terminal-virtual-list
          :style="{ transform: `translateY(${offsetY}px)` }"
        >
          <template v-for="(item, index) in visibleItems" :key="index">
            <li
              ref="itemRefs"
              class="relative w-full select-text list-none"
              :data-pyro-terminal-recycle-tracker="index"
            >
              <UiServersLogParser :log="item" />
            </li>
          </template>
        </ul>
      </div>
    </div>
    <slot />
    <button
      class="absolute right-4 top-4 grid size-12 place-content-center rounded-lg bg-bg-raised text-contrast transition-transform duration-300 hover:scale-110"
      @click="toggleFullscreen"
    >
      <svg
        v-if="!isFullScreen"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-6"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M4.5 9V4.5H9M4.5 9L9 4.5M15 4.5h4.5V9M15 4.5l4.5 4.5M4.5 15v4.5H9M4.5 15L9 19.5M15 19.5h4.5V15M15 19.5l4.5-4.5"
        />
      </svg>
      <svg
        v-else
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-6"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M9 9V4.5M9 9H4.5M9 9 3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5 5.25 5.25"
        />
      </svg>
      <span class="sr-only">Toggle browser full screen</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";

const props = defineProps<{
  consoleOutput: string[];
  fullScreen: boolean;
}>();

defineEmits<{
  (e: "toggle-full-screen"): void;
}>();

const scrollContainer = ref<HTMLElement | null>(null);
const itemRefs = ref<HTMLElement[]>([]);
const itemHeights = ref<number[]>([]);
const averageItemHeight = ref(36);
const bufferSize = 5;

const scrollTop = ref(0);
const clientHeight = ref(0);
const isFullScreen = ref(false);

const totalHeight = computed(
  () =>
    itemHeights.value.reduce((sum, height) => sum + height, 0) ||
    props.consoleOutput.length * averageItemHeight.value,
);

const getItemOffset = (index: number) => {
  return itemHeights.value.slice(0, index).reduce((sum, height) => sum + height, 0);
};

const visibleStartIndex = computed(() => {
  let index = 0;
  let offset = 0;
  while (
    index < props.consoleOutput.length &&
    offset < scrollTop.value - bufferSize * averageItemHeight.value
  ) {
    offset += itemHeights.value[index] || averageItemHeight.value;
    index++;
  }
  return Math.max(0, index - 1);
});

const visibleEndIndex = computed(() => {
  let index = visibleStartIndex.value;
  let offset = getItemOffset(index);
  while (
    index < props.consoleOutput.length &&
    offset < scrollTop.value + clientHeight.value + bufferSize * averageItemHeight.value
  ) {
    offset += itemHeights.value[index] || averageItemHeight.value;
    index++;
  }
  return Math.min(props.consoleOutput.length - 1, index);
});

const visibleItems = computed(() =>
  props.consoleOutput.slice(visibleStartIndex.value, visibleEndIndex.value + 1),
);

const offsetY = computed(() => getItemOffset(visibleStartIndex.value));

const handleScroll = () => {
  if (scrollContainer.value) {
    scrollTop.value = scrollContainer.value.scrollTop;
    clientHeight.value = scrollContainer.value.clientHeight;
  }
};

const updateItemHeights = () => {
  nextTick(() => {
    itemRefs.value.forEach((el, index) => {
      if (el) {
        const actualIndex = visibleStartIndex.value + index;
        itemHeights.value[actualIndex] = el.offsetHeight;
      }
    });

    const measuredHeights = itemHeights.value.filter((h) => h > 0);
    if (measuredHeights.length > 0) {
      averageItemHeight.value =
        measuredHeights.reduce((sum, height) => sum + height, 0) / measuredHeights.length;
    }
  });
};

const updateClientHeight = () => {
  if (scrollContainer.value) {
    clientHeight.value = scrollContainer.value.clientHeight;
  }
};

const scrollToBottom = () => {
  if (scrollContainer.value) {
    scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight;
  }
};

const enterFullScreen = () => {
  isFullScreen.value = true;
  document.body.style.overflow = "hidden";
};

const exitFullScreen = () => {
  isFullScreen.value = false;
  document.body.style.overflow = "";
};

const toggleFullscreen = () => {
  if (isFullScreen.value) {
    exitFullScreen();
  } else {
    enterFullScreen();
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape" && isFullScreen.value) {
    exitFullScreen();
  }
};

onMounted(() => {
  updateClientHeight();
  updateItemHeights();
  window.addEventListener("resize", updateClientHeight);
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateClientHeight);
  window.removeEventListener("keydown", handleKeydown);
});

watch(
  () => props.consoleOutput,
  () => {
    const newItemsCount = props.consoleOutput.length - itemHeights.value.length;
    if (newItemsCount > 0) {
      itemHeights.value.push(...Array(newItemsCount).fill(averageItemHeight.value));
    }

    nextTick(() => {
      updateItemHeights();
      scrollToBottom();
    });
  },
  { deep: true },
);

watch([visibleStartIndex, visibleEndIndex], updateItemHeights);
</script>

<style scoped>
.terminal-font {
  font-family: var(--mono-font);
  font-size: 1rem;
  line-height: 1.5em;
}

html.light-mode .console {
  background: var(--color-bg);
}

html.dark-mode .console {
  background: black;
}

html.oled-mode .console {
  background: black;
}

.fixed {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 50;
  background: var(--color-bg);
}

@keyframes scaleUp {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.scale-fullscreen {
  animation: scaleUp 190ms forwards;
}
</style>
