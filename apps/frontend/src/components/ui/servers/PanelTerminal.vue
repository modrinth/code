<template>
  <div
    data-pyro-terminal
    :class="[
      'terminal-font console relative flex h-full w-full select-text flex-col items-center justify-between overflow-hidden rounded-t-xl pb-4 text-sm transition-transform duration-300',
      { 'scale-fullscreen fixed inset-0 z-50 !rounded-none': isFullScreen },
    ]"
    tabindex="-1"
  >
    <div
      v-if="cosmetics.advancedRendering"
      class="progressive-gradient pointer-events-none absolute -bottom-6 left-0 z-[9999] h-[10rem] w-full overflow-hidden rounded-xl"
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
      class="pointer-events-none absolute bottom-0 left-0 right-0 z-[9999] h-[196px] w-full"
      :style="
        bottomThreshold > 0
          ? { background: 'linear-gradient(transparent 30%, var(--console-bg) 70%)' }
          : {}
      "
    ></div>
    <div
      aria-hidden="true"
      class="pointer-events-none absolute left-0 top-0 z-[9999999] h-full w-full"
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
    <div
      ref="scrollContainer"
      data-pyro-terminal-root
      class="absolute left-0 top-0 h-full w-full overflow-x-auto overflow-y-auto py-6 pb-[72px]"
      @scroll="handleScroll"
    >
      <div data-pyro-terminal-virtual-height-watcher :style="{ height: `${totalHeight}px` }">
        <ul
          class="m-0 list-none p-0"
          data-pyro-terminal-virtual-list
          :style="{ transform: `translateY(${offsetY}px)` }"
          aria-live="polite"
          role="listbox"
        >
          <template v-for="(item, index) in visibleItems" :key="index">
            <li
              ref="itemRefs"
              class="relative w-full list-none"
              :data-pyro-terminal-recycle-tracker="index"
              aria-setsize="-1"
            >
              <UiServersLogParser :log="item" />
            </li>
          </template>
        </ul>
      </div>
    </div>
    <div
      class="absolute bottom-4 z-[99999] w-full"
      :style="{
        filter: `drop-shadow(0 8px 12px rgba(0, 0, 0, ${lerp(0.1, 0.5, bottomThreshold)}))`,
      }"
    >
      <slot />
    </div>
    <button
      data-pyro-fullscreen
      :label="isFullScreen ? 'Exit full screen' : 'Enter full screen'"
      class="absolute right-4 top-4 grid size-12 place-content-center rounded-lg bg-bg-raised text-contrast transition-transform duration-200 hover:scale-110 active:scale-95"
      @click="toggleFullscreen"
    >
      <UiServersPanelTerminalFullscreen v-if="isFullScreen" />
      <UiServersPanelTerminalMinimize v-else />
    </button>

    <Transition name="scroll-to-bottom">
      <button
        v-if="bottomThreshold > 0"
        data-pyro-scrolltobottom
        label="Scroll to bottom"
        class="scroll-to-bottom-btn absolute bottom-20 right-4 z-[999999] grid size-12 place-content-center rounded-lg bg-bg-raised text-contrast transition-transform duration-200 hover:scale-110 active:scale-95"
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

const { $cosmetics } = useNuxtApp();
const cosmetics = $cosmetics;

const props = defineProps<{
  consoleOutput: string[];
  fullScreen: boolean;
}>();

const scrollContainer = ref<HTMLElement | null>(null);
const itemRefs = ref<HTMLElement[]>([]);
const itemHeights = ref<number[]>([]);
const averageItemHeight = ref(36);
const bottomThreshold = ref(0);
const bufferSize = 5;

const progressiveBlurIterations = ref(8);

const scrollTop = ref(0);
const clientHeight = ref(0);
const isFullScreen = ref(props.fullScreen);

const initial = ref(false);
const userHasScrolled = ref(false);
const isScrolledToBottom = ref(true);

const totalHeight = computed(
  () =>
    itemHeights.value.reduce((sum, height) => sum + height, 0) ||
    props.consoleOutput.length * averageItemHeight.value,
);

watch(totalHeight, () => {
  if (!initial.value) {
    scrollToBottom();
  }
  initial.value = true;
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

    const scrollHeight = scrollContainer.value.scrollHeight;
    isScrolledToBottom.value = scrollTop.value + clientHeight.value >= scrollHeight - 32; // threshold

    if (!isScrolledToBottom.value) {
      userHasScrolled.value = true;
    }
  }

  const maxBottom = 256;
  bottomThreshold.value = Math.min(
    1,
    ((scrollContainer.value?.scrollHeight || 1) - scrollTop.value - clientHeight.value) / maxBottom,
  );
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
    scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight + 99999999;
    userHasScrolled.value = false;
    isScrolledToBottom.value = true;
  }
};

const debouncedScrollToBottom = () => {
  requestAnimationFrame(() => {
    setTimeout(scrollToBottom, 0);
  });
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

onMounted(() => {
  updateClientHeight();
  updateItemHeights();
  nextTick(() => {
    updateItemHeights();
    setTimeout(scrollToBottom, 200);
  });
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
      if (!userHasScrolled.value || isScrolledToBottom.value) {
        debouncedScrollToBottom();
      }
    });
  },
  { deep: true, immediate: true },
);

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

[data-pyro-terminal-root]::-webkit-scrollbar {
  background: none;
  width: 10px;
  height: 16px;
}

[data-pyro-terminal-root]::-webkit-scrollbar-thumb {
  border: solid 0 rgb(0 0 0 / 0%);
  border-right-width: 3px;
  border-left-width: 3px;
  -webkit-border-radius: 9px 4px;
  -webkit-box-shadow: inset 0 0 0 3px var(--color-button-bg);
}

[data-pyro-terminal-root]::-webkit-scrollbar-track-piece {
  margin: 4px 0;
}

[data-pyro-terminal-root]::-webkit-scrollbar-thumb:horizontal {
  border-right-width: 0;
  border-left-width: 0;
  border-top-width: 4px;
  border-bottom-width: 4px;
  -webkit-border-radius: 4px 9px;
}

[data-pyro-terminal-root]::-webkit-scrollbar-corner {
  background: transparent;
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
</style>
