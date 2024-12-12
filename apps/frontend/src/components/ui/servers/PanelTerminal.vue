<template>
  <div
    data-pyro-terminal
    :class="[
      'terminal-font console relative z-[1] flex h-full w-full flex-col items-center justify-between overflow-hidden rounded-t-xl px-1 text-sm transition-transform duration-300',
      { 'scale-fullscreen screen-fixed inset-0 z-50 !rounded-none': isFullScreen },
    ]"
    tabindex="-1"
  >
    <div
      v-if="cosmetics.advancedRendering"
      class="progressive-gradient pointer-events-none absolute -bottom-6 left-0 z-[2] h-[10rem] w-full overflow-hidden rounded-xl"
      :style="`--transparency: ${Math.max(0, lerp(100, 0, bottomThreshold * 8))}%`"
      aria-hidden="true"
    >
      <div
        v-for="i in progressiveBlurIterations"
        :key="i"
        aria-hidden="true"
        class="absolute left-0 top-0 h-full w-full"
        :style="getBlurStyle(i)"
      />
    </div>
    <div
      v-else
      class="pointer-events-none absolute bottom-0 left-0 right-0 z-[2] h-[196px] w-full"
      :style="
        bottomThreshold > 0
          ? { background: 'linear-gradient(transparent 30%, var(--console-bg) 70%)' }
          : {}
      "
    ></div>
    <div
      aria-hidden="true"
      class="pointer-events-none absolute left-0 top-0 z-[60] h-full w-full"
      :style="{
        visibility: isFullScreen ? 'hidden' : 'visible',
      }"
    >
      <div
        aria-hidden="true"
        class="absolute -bottom-2 -right-2 h-7 w-7"
        :style="{
          background: `radial-gradient(circle at 0% 0%, transparent 50%, var(--color-raised-bg) 52%)`,
        }"
      ></div>
      <div
        aria-hidden="true"
        class="absolute -bottom-2 -left-2 h-7 w-7"
        :style="{
          background: `radial-gradient(circle at 100% 0%, transparent 50%, var(--color-raised-bg) 52%)`,
        }"
      ></div>
    </div>
    <div data-pyro-terminal-scroll-root class="relative h-full w-full">
      <div
        ref="scrollbarTrack"
        data-pyro-terminal-scrollbar-track
        class="absolute -right-1 bottom-16 top-4 z-[4] w-4"
        @mousedown="handleTrackClick"
      >
        <div
          data-pyro-terminal-scrollbar
          class="flex h-full justify-center rounded-full transition-all"
          :style="{ opacity: bottomThreshold > 0 ? '1' : '0.5' }"
        >
          <div
            ref="scrollbarThumb"
            data-pyro-terminal-scrollbar-thumb
            class="absolute w-1.5 cursor-default rounded-full bg-button-bg"
            :style="{
              height: `${getThumbHeight()}px`,
              transform: `translateY(${getThumbPosition()}px)`,
            }"
            @mousedown="startDragging"
          ></div>
        </div>
      </div>
      <div
        ref="scrollContainer"
        data-pyro-terminal-root
        class="scrollbar-none absolute left-0 top-0 h-full w-full select-text overflow-x-auto overflow-y-auto py-6 pb-[72px]"
        @scroll="handleScrollEvent"
      >
        <div data-pyro-terminal-virtual-height-watcher :style="{ height: `${totalHeight}px` }">
          <ul
            class="m-0 list-none p-0"
            data-pyro-terminal-virtual-list
            :style="virtualListStyle"
            aria-live="polite"
            role="listbox"
          >
            <template v-for="(item, index) in visibleItems" :key="index">
              <li>
                <UiServersLogParser :log="item" :index="visibleStartIndex + index" />
              </li>
            </template>
          </ul>
        </div>
      </div>
    </div>

    <div
      class="absolute bottom-4 z-[3] w-full"
      :style="{
        filter: `drop-shadow(0 8px 12px rgba(0, 0, 0, ${lerp(0.1, 0.5, bottomThreshold)}))`,
      }"
    >
      <slot />
    </div>
    <button
      data-pyro-fullscreen
      :label="isFullScreen ? 'Exit full screen' : 'Enter full screen'"
      class="experimental-styles-within absolute right-4 top-4 z-[3] grid h-12 w-12 place-content-center rounded-lg border-[1px] border-solid border-button-border bg-bg-raised text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
      @click="toggleFullscreen"
    >
      <UiServersPanelTerminalMinimize v-if="isFullScreen" />
      <UiServersPanelTerminalFullscreen v-else />
    </button>

    <Transition name="scroll-to-bottom">
      <button
        v-if="bottomThreshold > 0 && !isScrolledToBottom"
        data-pyro-scrolltobottom
        label="Scroll to bottom"
        class="scroll-to-bottom-btn experimental-styles-within absolute bottom-[4.5rem] right-4 z-[3] grid h-12 w-12 place-content-center rounded-lg border-[1px] border-solid border-button-border bg-bg-raised text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
        @click="scrollToBottom"
      >
        <RightArrowIcon class="rotate-90" />
        <span class="sr-only">Scroll to bottom</span>
      </button>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { RightArrowIcon } from "@modrinth/assets";
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { usePyroConsole } from "~/store/console.ts";

const { $cosmetics } = useNuxtApp();
const cosmetics = $cosmetics;

const props = defineProps<{
  fullScreen: boolean;
}>();

const pyroConsole = usePyroConsole();
const consoleOutput = pyroConsole.output;

const scrollContainer = ref<HTMLElement | null>(null);
const itemHeights = ref<number[]>([]);
const averageItemHeight = ref(36);
const bottomThreshold = ref(0);
const bufferSize = 5;
const cachedHeights = ref<Map<string, number>>(new Map());
const isAutoScrolling = ref(false);

const progressiveBlurIterations = ref(8);

const scrollTop = ref(0);
const clientHeight = ref(0);
const isFullScreen = ref(props.fullScreen);

const initial = ref(false);
const userHasScrolled = ref(false);
const isScrolledToBottom = ref(true);

const handleScrollEvent = () => {
  handleListScroll();
};

const totalHeight = computed(
  () =>
    itemHeights.value.reduce((sum, height) => sum + height, 0) ||
    consoleOutput.value.length * averageItemHeight.value,
);

watch(totalHeight, () => {
  if (isScrolledToBottom.value) {
    scrollToBottom();
  }
  if (!initial.value) {
    initial.value = true;
  }
});

const lerp = (start: number, end: number, t: number) => start * (1 - t) + end * t;

const getBlurStyle = (i: number) => {
  const properBlurIteration = i + 1;
  const blur = lerp(0, 2 ** (properBlurIteration - 3), bottomThreshold.value);
  const singular = 100 / progressiveBlurIterations.value;
  let mask = "linear-gradient(";

  switch (i) {
    case 0:
      mask += `rgba(0, 0, 0, 0) 0%, rgb(0, 0, 0) ${singular}%`;
      break;
    case 1:
      mask += `rgba(0, 0, 0, 0) 0%, rgba(0, 0, 0, 0) ${singular}%, rgb(0, 0, 0) ${singular * 2}%`;
      break;
    case 2:
      mask += `rgba(0, 0, 0, 0) 0%, rgba(0, 0, 0, 0) ${singular}%, rgba(0, 0, 0, 0) ${singular * 2}%, rgb(0, 0, 0) ${singular * 3}%`;
      break;
    default:
      mask += `rgba(0, 0, 0, 0) ${singular * (i - 3)}%, rgb(0, 0, 0) ${singular * (i + 1 - 3)}%, rgb(0, 0, 0) ${singular * (i + 2 - 3)}%, rgba(0, 0, 0, 0) ${singular * (i + 3 - 3)}%`;
      break;
  }

  mask += `)`;

  return {
    backdropFilter: `blur(${blur}px)`,
    mask,
    position: "absolute" as any,
    zIndex: progressiveBlurIterations.value - i,
  };
};

const getItemOffset = (index: number) => {
  return itemHeights.value.slice(0, index).reduce((sum, height) => sum + height, 0);
};

const visibleStartIndex = computed(() => {
  let index = 0;
  let offset = 0;
  while (
    index < consoleOutput.value.length &&
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
    index < consoleOutput.value.length &&
    offset < scrollTop.value + clientHeight.value + bufferSize * averageItemHeight.value
  ) {
    offset += itemHeights.value[index] || averageItemHeight.value;
    index++;
  }
  return Math.min(consoleOutput.value.length - 1, index);
});

const visibleItems = computed(() =>
  consoleOutput.value.slice(visibleStartIndex.value, visibleEndIndex.value + 1),
);

const offsetY = computed(() => getItemOffset(visibleStartIndex.value));

const handleListScroll = () => {
  if (!scrollContainer.value) return;

  const container = scrollContainer.value;
  scrollTop.value = container.scrollTop;
  clientHeight.value = container.clientHeight;

  const scrollHeight = container.scrollHeight;
  const threshold = 32;

  isScrolledToBottom.value = scrollHeight - scrollTop.value - clientHeight.value <= threshold;

  if (!isScrolledToBottom.value && !isAutoScrolling.value) {
    userHasScrolled.value = true;
  }

  bottomThreshold.value = Math.min(1, (scrollHeight - scrollTop.value - clientHeight.value) / 256);
};

const updateItemHeights = async () => {
  if (!scrollContainer.value) return;

  await nextTick();
  const items =
    scrollContainer.value?.querySelectorAll("[data-pyro-terminal-virtual-list] li") || [];
  items.forEach((el, idx) => {
    const index = visibleStartIndex.value + idx;
    const height = el.getBoundingClientRect().height;
    itemHeights.value[index] = height;
    const content = consoleOutput.value[index];
    if (content) {
      cachedHeights.value.set(content, height);
    }
  });
};

const updateClientHeight = () => {
  if (scrollContainer.value) {
    clientHeight.value = scrollContainer.value.clientHeight;
  }
};

const scrollToBottom = () => {
  if (!scrollContainer.value) return;

  isAutoScrolling.value = true;
  const container = scrollContainer.value;

  nextTick(() => {
    const maxScroll = container.scrollHeight - container.clientHeight;
    container.scrollTop = maxScroll;

    setTimeout(() => {
      if (container.scrollTop < maxScroll) {
        container.scrollTop = maxScroll;
      }
      isAutoScrolling.value = false;
      userHasScrolled.value = false;
      isScrolledToBottom.value = true;
      handleListScroll();
    }, 50);
  });
};

const scrollbarTrack = ref<HTMLElement | null>(null);
const scrollbarThumb = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const startY = ref(0);
const startScrollTop = ref(0);

const getThumbHeight = () => {
  if (!scrollContainer.value || !scrollbarTrack.value) return 30;

  const contentHeight = scrollContainer.value.scrollHeight;
  const viewportHeight = scrollContainer.value.clientHeight;
  const trackHeight = scrollbarTrack.value.clientHeight;

  const heightRatio = viewportHeight / contentHeight;

  const minThumbHeight = Math.min(40, trackHeight / 2);

  const proposedHeight = Math.max(heightRatio * trackHeight, minThumbHeight);

  return Math.min(proposedHeight, trackHeight);
};

const getThumbPosition = () => {
  if (!scrollContainer.value || !scrollbarTrack.value) return 0;

  const contentHeight = scrollContainer.value.scrollHeight;
  const viewportHeight = scrollContainer.value.clientHeight;
  const trackHeight = scrollbarTrack.value.clientHeight;
  const scrollProgress = scrollTop.value / (contentHeight - viewportHeight);

  const thumbHeight = getThumbHeight();
  const availableTrackSpace = trackHeight - thumbHeight;

  return Math.max(0, Math.min(scrollProgress * availableTrackSpace, availableTrackSpace));
};

const startDragging = (event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();

  if (!scrollContainer.value || !scrollbarTrack.value) return;

  isDragging.value = true;
  startY.value = event.clientY;
  startScrollTop.value = scrollContainer.value.scrollTop;

  window.addEventListener("mousemove", handleDragging);
  window.addEventListener("mouseup", stopDragging);

  document.body.style.userSelect = "none";
  document.body.style.pointerEvents = "none";
};

const handleDragging = (event: MouseEvent) => {
  if (!isDragging.value || !scrollContainer.value || !scrollbarTrack.value) return;

  const trackRect = scrollbarTrack.value.getBoundingClientRect();
  const deltaY = event.clientY - startY.value;

  const trackHeight = trackRect.height;
  const contentHeight = scrollContainer.value.scrollHeight;
  const viewportHeight = scrollContainer.value.clientHeight;
  const maxScroll = contentHeight - viewportHeight;

  const moveRatio = deltaY / trackHeight;
  const scrollDelta = moveRatio * maxScroll;

  const newScrollTop = Math.max(0, Math.min(startScrollTop.value + scrollDelta, maxScroll));
  scrollContainer.value.scrollTop = newScrollTop;
};

const stopDragging = () => {
  isDragging.value = false;

  window.removeEventListener("mousemove", handleDragging);
  window.removeEventListener("mouseup", stopDragging);

  document.body.style.userSelect = "";
  document.body.style.pointerEvents = "";
};

const handleTrackClick = (event: MouseEvent) => {
  if (!scrollContainer.value || !scrollbarTrack.value || event.target === scrollbarThumb.value)
    return;

  const trackRect = scrollbarTrack.value.getBoundingClientRect();
  const thumbHeight = getThumbHeight();

  const clickOffset = event.clientY - trackRect.top;

  const currentThumbPosition = getThumbPosition();
  const thumbCenterPosition = currentThumbPosition + thumbHeight / 2;
  const scrollAmount = clientHeight.value * (clickOffset < thumbCenterPosition ? -1 : 1);

  const newScrollTop = Math.max(
    0,
    Math.min(
      scrollContainer.value.scrollTop + scrollAmount,
      scrollContainer.value.scrollHeight - scrollContainer.value.clientHeight,
    ),
  );

  scrollContainer.value.scrollTop = newScrollTop;
};

const enterFullScreen = () => {
  isFullScreen.value = true;
  document.body.style.overflow = "hidden";
  nextTick(() => {
    updateClientHeight();
    updateItemHeights();
  });
};

const exitFullScreen = () => {
  isFullScreen.value = false;
  document.body.style.overflow = "";
  nextTick(() => {
    updateClientHeight();
    updateItemHeights();
  });
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

const initializeTerminal = async () => {
  if (!scrollContainer.value) return;

  updateClientHeight();

  const initialHeights = consoleOutput.value.map(
    (content) => cachedHeights.value.get(content) || averageItemHeight.value,
  );
  itemHeights.value = initialHeights;

  await nextTick();
  await updateItemHeights();
  await nextTick();

  const container = scrollContainer.value;
  container.scrollTop = container.scrollHeight;

  handleListScroll();
  initial.value = true;
};

onMounted(async () => {
  await initializeTerminal();

  window.addEventListener("resize", updateClientHeight);
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateClientHeight);
  window.removeEventListener("keydown", handleKeydown);
  stopDragging();
});

watch(
  () => consoleOutput.value,
  async (newOutput) => {
    const newItemsCount = newOutput.length - itemHeights.value.length;

    if (newItemsCount > 0) {
      const shouldScroll = isScrolledToBottom.value || !userHasScrolled.value;

      const newHeights = Array(newItemsCount)
        .fill(0)
        .map((_, i) => {
          const index = itemHeights.value.length + i;
          const content = newOutput[index];
          return cachedHeights.value.get(content) || averageItemHeight.value;
        });

      itemHeights.value.push(...newHeights);

      if (shouldScroll) {
        await nextTick();
        scrollToBottom();

        await nextTick();
        await updateItemHeights();
        scrollToBottom();
      }
    }
  },
  { deep: true },
);
const virtualListStyle = computed(() => ({
  transform: `translateY(${offsetY.value}px)`,
}));

watch([visibleStartIndex, visibleEndIndex], updateItemHeights);

watch(
  () => props.fullScreen,
  (newValue) => {
    isFullScreen.value = newValue;
    nextTick(() => {
      updateClientHeight();
      updateItemHeights();
    });
  },
);

watch(isFullScreen, () => {
  nextTick(() => {
    updateClientHeight();
    updateItemHeights();
  });
});

watch(
  itemHeights,
  () => {
    const totalHeight = itemHeights.value.reduce((sum, height) => sum + height, 0);
    averageItemHeight.value = totalHeight / itemHeights.value.length || averageItemHeight.value;
  },
  { deep: true },
);
</script>

<style scoped>
:root {
  --console-bg: var(--color-bg);
}

.terminal-font {
  font-family: var(--mono-font);
  font-size: 1rem;
  line-height: 1.5em;
}

html.light-mode .console {
  --console-bg: var(--color-bg);
}

html.dark-mode .console {
  --console-bg: black;
}

html.oled-mode .console {
  --console-bg: black;
}

.console {
  background: var(--console-bg);
}

.scrollbar-none {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.scrollbar-none::-webkit-scrollbar {
  display: none;
}

[data-pyro-terminal-root]::-webkit-scrollbar,
[data-pyro-terminal-root]::-webkit-scrollbar-thumb,
[data-pyro-terminal-root]::-webkit-scrollbar-track-piece,
[data-pyro-terminal-root]::-webkit-scrollbar-corner {
  display: none;
}

.screen-fixed {
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

.progressive-gradient {
  background: linear-gradient(
    to top,
    color-mix(in srgb, var(--color-bg), transparent var(--transparency)) 0%,
    rgba(0, 0, 0, 0) 100%
  );
}

html.dark-mode .progressive-gradient {
  background: linear-gradient(
    to top,
    color-mix(in srgb, black, transparent var(--transparency)) 0%,
    rgba(0, 0, 0, 0) 100%
  );
}

.scroll-to-bottom-enter-active,
.scroll-to-bottom-leave-active {
  transition:
    opacity 300ms ease,
    transform 300ms ease;
}

.scroll-to-bottom-enter-from,
.scroll-to-bottom-leave-to {
  opacity: 0;
  transform: scale(0.4) translateY(2rem);
}

[data-pyro-terminal-selected="true"] {
  border-radius: 0;
}

[data-pyro-terminal-selected="true"].first-selected {
  border-top-left-radius: 0.5rem;
  border-top-right-radius: 0.5rem;
  overflow: hidden !important;
}

[data-pyro-terminal-selected="true"].last-selected {
  border-bottom-left-radius: 0.5rem;
  border-bottom-right-radius: 0.5rem;
  overflow: hidden !important;
}

[data-pyro-terminal-root] {
  will-change: transform;
  backface-visibility: hidden;
  transform: translateZ(0);
  -webkit-font-smoothing: subpixel-antialiased;
}

[data-pyro-terminal-root] {
  user-select: none;
}

[data-pyro-terminal-root] * {
  user-select: text;
}

.selection-in-progress {
  pointer-events: none;
}
</style>
