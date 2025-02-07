<template>
  <div class="contents">
    <div class="flex items-center gap-4">
      <div class="relative w-full">
        <input
          v-model="searchInput"
          type="text"
          placeholder="Search logs..."
          class="h-12 !w-full !pl-10 pr-10"
          @keydown.escape="clearSearch"
        />
        <SearchIcon class="absolute left-4 top-1/2 -translate-y-1/2" />
        <ButtonStyled v-if="searchInput" @click="clearSearch">
          <button class="absolute right-2 top-1/2 -translate-y-1/2">
            <XIcon class="h-5 w-5" />
          </button>
        </ButtonStyled>
      </div>
      <span
        v-if="pyroConsole.filteredOutput.value.length && searchInput"
        class="whitespace-pre text-sm text-contrast"
      >
        {{ pyroConsole.filteredOutput.value.length }}
        {{ pyroConsole.filteredOutput.value.length === 1 ? "result" : "results" }}
      </span>
    </div>
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
          class="absolute -right-1 bottom-16 top-4 z-[4] w-4 overflow-hidden"
          @mousedown="handleTrackClick"
        >
          <div
            v-if="hasSelection"
            class="absolute h-full w-full opacity-20"
            :style="{
              background: `linear-gradient(to bottom,
                transparent ${getSelectionPosition().start}%,
                var(--color-blue) ${getSelectionPosition().start}%,
                var(--color-blue) ${getSelectionPosition().end}%,
                transparent ${getSelectionPosition().end}%)`,
            }"
          />
          <div
            data-pyro-terminal-scrollbar
            class="relative flex h-full justify-center rounded-full transition-all"
            :style="{ opacity: bottomThreshold > 0 ? '0.7' : '0.3' }"
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
          @scroll.passive="() => handleScrollEvent()"
        >
          <div data-pyro-terminal-virtual-height-watcher :style="{ height: `${totalHeight}px` }">
            <ul
              class="m-0 list-none p-0"
              data-pyro-terminal-virtual-list
              :class="{ 'scrolling-active': isScrolling && !isSelecting }"
              :style="virtualListStyle"
              aria-live="polite"
              role="listbox"
              @mousedown.prevent="handleMouseDown"
              @mousemove="updateLineSelection"
              @mouseup="endLineSelection"
            >
              <template
                v-for="(item, index) in visibleItems"
                :key="`${visibleStartIndex + index}-${item}`"
              >
                <li :class="{ 'selected-line': isLineSelected(visibleStartIndex + index) }">
                  <UiServersLogLine :log="item" @show-full-log="showFullLogMessage" />
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
        class="experimental-styles-within absolute right-4 top-4 z-[3] grid h-12 w-12 place-content-center rounded-full border-[1px] border-solid border-button-border bg-bg-raised text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
        @click="toggleFullscreen"
      >
        <LazyUiServersIconsMinimizeIconVue v-if="isFullScreen" />
        <LazyUiServersIconsFullscreenIcon v-else />
      </button>

      <Transition name="fade">
        <div v-if="hasSelection" class="absolute right-20 top-4 z-[3] flex flex-row gap-2">
          <button
            data-pyro-copy
            label="Copy selected lines"
            class="experimental-styles-within flex h-12 flex-row items-center justify-center gap-2 rounded-full border-[1px] border-solid border-button-border bg-bg-raised px-4 text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
            @click="copySelectedLines"
          >
            <CopyIcon class="h-5 w-5" />
            <span class="">Copy</span>
          </button>
          <button
            data-pyro-view
            label="View full content"
            class="experimental-styles-within flex h-12 flex-row items-center justify-center gap-2 rounded-full border-[1px] border-solid border-button-border bg-bg-raised px-4 text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
            @click="showSelectedLines"
          >
            <EyeIcon class="h-5 w-5" />
            <span class="">View</span>
          </button>
        </div>
      </Transition>

      <Transition name="scroll-to-bottom">
        <button
          v-if="bottomThreshold > 0 && !isScrolledToBottom"
          data-pyro-scrolltobottom
          label="Scroll to bottom"
          class="scroll-to-bottom-btn experimental-styles-within absolute bottom-[4.5rem] right-4 z-[3] grid h-12 w-12 place-content-center rounded-full border-[1px] border-solid border-button-border bg-bg-raised text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
          @click="scrollToBottom"
        >
          <RightArrowIcon class="rotate-90" />
          <span class="sr-only">Scroll to bottom</span>
        </button>
      </Transition>
    </div>
    <NewModal ref="logModal" class="z-[9999]" header="Viewing full log">
      <div class="text-contrast">
        <pre class="select-text overflow-x-auto whitespace-pre font-mono">{{ selectedLog }}</pre>
      </div>
    </NewModal>
  </div>
</template>

<script setup lang="ts">
import { RightArrowIcon, CopyIcon, XIcon, SearchIcon, EyeIcon } from "@modrinth/assets";
import { ref, computed, onMounted, onUnmounted, watch, nextTick, shallowRef } from "vue";
import { useThrottleFn, useDebounceFn } from "@vueuse/core";
import { NewModal } from "@modrinth/ui";
import ButtonStyled from "@modrinth/ui/src/components/base/ButtonStyled.vue";
import { usePyroConsole } from "~/store/console.ts";

const { $cosmetics } = useNuxtApp();
const cosmetics = $cosmetics;

const props = defineProps<{
  fullScreen: boolean;
}>();

const pyroConsole = usePyroConsole();
const consoleOutput = pyroConsole.output;

const scrollContainer = ref<HTMLElement | null>(null);
const bottomThreshold = ref(0);
const bufferSize = 5;
const cachedHeights = shallowRef<Map<string, number>>(new Map());
const isAutoScrolling = ref(false);

const progressiveBlurIterations = ref(8);

const scrollTop = ref(0);
const clientHeight = ref(0);
const isFullScreen = ref(props.fullScreen);

const initial = ref(false);
const userHasScrolled = ref(false);
const isScrolledToBottom = ref(true);

const BATCH_SIZE = 50;
const SCROLL_THROTTLE = 16;

const LINE_HEIGHT = 32;

const SCROLL_END_DELAY = 150;

const scrollEndTimeout = ref<NodeJS.Timeout | null>(null);
const isScrolling = ref(false);

const handleScrollEvent = useThrottleFn(() => {
  handleListScroll();
}, SCROLL_THROTTLE);

const searchInput = ref("");
const updateSearch = useDebounceFn((value: string) => {
  pyroConsole.setSearchQuery(value);

  selectionStart.value = null;
  selectionEnd.value = null;
  lastClickIndex.value = null;

  nextTick(() => {
    if (!scrollContainer.value) return;

    if (!value) {
      handleListScroll();
      return;
    }

    if (pyroConsole.filteredOutput.value.length > 0) {
      scrollContainer.value.scrollTop = 0;
      handleListScroll();
    }
  });
}, 300);

watch(searchInput, (value) => {
  updateSearch(value);
});

const clearSearch = () => {
  searchInput.value = "";
  pyroConsole.setSearchQuery("");
};

const activeOutput = computed(() => {
  return searchInput.value ? pyroConsole.filteredOutput.value : consoleOutput.value;
});

const totalHeight = computed(() => activeOutput.value.length * LINE_HEIGHT);

const visibleStartIndex = computed(() => {
  return Math.max(0, Math.floor(scrollTop.value / LINE_HEIGHT) - bufferSize);
});

const visibleEndIndex = computed(() => {
  return Math.min(
    activeOutput.value.length - 1,
    Math.ceil((scrollTop.value + clientHeight.value) / LINE_HEIGHT) + bufferSize,
  );
});

const offsetY = computed(() => visibleStartIndex.value * LINE_HEIGHT);

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

const visibleItems = computed(() => {
  const start = visibleStartIndex.value;
  const end = visibleEndIndex.value;
  const items = [];

  for (let i = start; i <= end; i += BATCH_SIZE) {
    const chunk = activeOutput.value.slice(i, Math.min(i + BATCH_SIZE, end + 1));
    items.push(...chunk);
  }

  return items;
});

const handleListScroll = () => {
  if (!scrollContainer.value) return;

  const container = scrollContainer.value;
  scrollTop.value = container.scrollTop;
  clientHeight.value = container.clientHeight;

  const scrollHeight = container.scrollHeight;
  const threshold = 32;

  isScrolling.value = true;

  if (scrollEndTimeout.value) {
    clearTimeout(scrollEndTimeout.value);
  }

  scrollEndTimeout.value = setTimeout(() => {
    if (!isSelecting.value) {
      isScrolling.value = false;
    }
    const finalPosition = scrollHeight - container.scrollTop - clientHeight.value;
    isScrolledToBottom.value = finalPosition <= threshold;
    bottomThreshold.value = Math.min(1, finalPosition / 256);
  }, SCROLL_END_DELAY);

  isScrolledToBottom.value = scrollHeight - scrollTop.value - clientHeight.value <= threshold;

  if (!isScrolledToBottom.value && !isAutoScrolling.value) {
    userHasScrolled.value = true;
  }

  bottomThreshold.value = Math.min(1, (scrollHeight - scrollTop.value - clientHeight.value) / 256);

  if (searchInput.value) {
    nextTick(() => {
      container.style.transform = "translateZ(0)";
      requestAnimationFrame(() => {
        container.style.transform = "";
      });
    });
  }
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

const setBodyScroll = (enabled: boolean) => {
  if (enabled) {
    document.body.style.overflow = "";
    document.body.style.position = "";
    document.body.style.width = "";
  } else {
    document.body.style.overflow = "hidden";
    document.body.style.position = "fixed";
    document.body.style.width = "100%";
  }
};

const enterFullScreen = () => {
  isFullScreen.value = true;
  setBodyScroll(false);
  nextTick(() => {
    updateClientHeight();
  });
};

const exitFullScreen = () => {
  isFullScreen.value = false;
  setBodyScroll(true);
  nextTick(() => {
    updateClientHeight();
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
  if (event.key === "Escape") {
    if (selectionStart.value !== null || selectionEnd.value !== null) {
      selectionStart.value = null;
      selectionEnd.value = null;
      lastClickIndex.value = null;
      return;
    }
    if (isFullScreen.value) {
      exitFullScreen();
    }
  }
};

const handleGlobalMouseUp = () => {
  if (isSelecting.value) {
    endLineSelection();
  }
};

const initializeTerminal = () => {
  if (!scrollContainer.value) return;

  updateClientHeight();

  nextTick(() => {
    nextTick(() => {
      const container = scrollContainer.value;
      if (container) {
        container.scrollTop = container.scrollHeight;
        handleListScroll();
        initial.value = true;
      }
    });
  });
};

const isInitialLoad = ref(true);

const logModal = ref<InstanceType<typeof NewModal>>();
const selectedLog = ref("");

const showFullLogMessage = (log: string) => {
  selectedLog.value = log;
  logModal.value?.show();
};

const showSelectedLines = () => {
  if (!hasSelection.value) return;

  const start = Math.min(selectionStart.value!, selectionEnd.value!);
  const end = Math.max(selectionStart.value!, selectionEnd.value!);
  const selectedLines = activeOutput.value.slice(start, end + 1);

  selectedLog.value = selectedLines.join("\n");
  logModal.value?.show();
};

watch(
  () => consoleOutput.value,
  (newOutput, oldOutput) => {
    if (!oldOutput || newOutput.length <= oldOutput.length) return;

    const shouldScroll = isScrolledToBottom.value || !userHasScrolled.value || isInitialLoad.value;

    if (shouldScroll) {
      if (isInitialLoad.value) {
        setTimeout(() => {
          scrollToBottom();
          isInitialLoad.value = false;
        }, 100);
      } else {
        nextTick(scrollToBottom);
      }
    }
  },
  { flush: "post" },
);

watch(
  () => pyroConsole.filteredOutput.value,
  () => {
    if (searchInput.value && scrollContainer.value) {
      nextTick(() => {
        handleListScroll();
      });
    }
  },
);

onMounted(() => {
  initializeTerminal();

  window.addEventListener("resize", updateClientHeight);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("mouseup", handleGlobalMouseUp);
  window.addEventListener("contextmenu", handleContextMenu);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateClientHeight);
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("mouseup", handleGlobalMouseUp);
  window.removeEventListener("contextmenu", handleContextMenu);
  stopDragging();
  cachedHeights.value.clear();
  setBodyScroll(true);
  if (scrollEndTimeout.value) {
    clearTimeout(scrollEndTimeout.value);
  }
});

const virtualListStyle = computed(() => ({
  transform: `translateY(${offsetY.value}px)`,
}));

watch(
  () => props.fullScreen,
  (newValue) => {
    isFullScreen.value = newValue;
    nextTick(() => {
      updateClientHeight();
    });
  },
);

watch(isFullScreen, () => {
  nextTick(() => {
    updateClientHeight();
  });
});

const selectionStart = ref<number | null>(null);
const selectionEnd = ref<number | null>(null);
const isSelecting = ref(false);
const autoScrollInterval = ref<NodeJS.Timeout | null>(null);

const isLineSelected = (index: number) => {
  if (selectionStart.value === null || selectionEnd.value === null) return false;
  const start = Math.min(selectionStart.value, selectionEnd.value);
  const end = Math.max(selectionStart.value, selectionEnd.value);
  return index >= start && index <= end;
};

const lastClickIndex = ref<number | null>(null);

const handleContextMenu = () => {
  if (isSelecting.value) {
    endLineSelection();
  }
};

const handleMouseDown = (event: MouseEvent) => {
  if (event.button === 2) {
    if (isSelecting.value) {
      endLineSelection();
    }
    return;
  }

  const lineIndex = getLineIndexFromEvent(event);
  if (lineIndex === null) return;

  if (event.shiftKey && lastClickIndex.value !== null) {
    selectionStart.value = lastClickIndex.value;
    selectionEnd.value = lineIndex;
    isSelecting.value = false;
  } else {
    isSelecting.value = true;
    selectionStart.value = lineIndex;
    selectionEnd.value = lineIndex;
    lastClickIndex.value = lineIndex;
    startAutoScroll();
  }
};

const updateLineSelection = (event: MouseEvent) => {
  if (!isSelecting.value) return;

  const lineIndex = getLineIndexFromEvent(event);
  if (lineIndex === null) return;

  selectionEnd.value = lineIndex;
  lastMouseEvent.value = event;

  const rect = scrollContainer.value?.getBoundingClientRect();
  if (!rect) return;

  const threshold = 80;
  const y = event.clientY;

  const distanceFromTop = Math.max(0, threshold - (y - rect.top));
  const distanceFromBottom = Math.max(0, threshold - (rect.bottom - y));

  if (distanceFromTop > 0) {
    autoScrollSpeed.value = -Math.min(20, distanceFromTop / 2);
  } else if (distanceFromBottom > 0) {
    autoScrollSpeed.value = Math.min(20, distanceFromBottom / 2);
  } else {
    autoScrollSpeed.value = 0;
  }
};

const endLineSelection = () => {
  if (!isSelecting.value) return;
  isSelecting.value = false;
  stopAutoScroll();
};

const getLineIndexFromEvent = (event: MouseEvent): number | null => {
  if (!scrollContainer.value) return null;

  const rect = scrollContainer.value.getBoundingClientRect();
  const relativeY = event.clientY - rect.top + scrollContainer.value.scrollTop - 24;
  const lineIndex = Math.floor(relativeY / LINE_HEIGHT);

  return Math.max(0, Math.min(lineIndex, activeOutput.value.length - 1));
};

const autoScrollSpeed = ref(0);

const startAutoScroll = () => {
  if (autoScrollInterval.value) return;
  autoScrollInterval.value = setInterval(() => {
    if (!scrollContainer.value || autoScrollSpeed.value === 0) return;
    scrollContainer.value.scrollTop += autoScrollSpeed.value;
    if (lastMouseEvent.value) {
      updateLineSelection(lastMouseEvent.value);
    }
  }, 16);
};

const stopAutoScroll = () => {
  if (autoScrollInterval.value) {
    clearInterval(autoScrollInterval.value);
    autoScrollInterval.value = null;
  }
  autoScrollSpeed.value = 0;
};

const handleCopy = (event: KeyboardEvent) => {
  if (!event.metaKey && !event.ctrlKey) return;
  if (event.key !== "c") return;
  if (selectionStart.value === null || selectionEnd.value === null) return;

  event.preventDefault();

  const start = Math.min(selectionStart.value, selectionEnd.value);
  const end = Math.max(selectionStart.value, selectionEnd.value);
  const selectedLines = activeOutput.value.slice(start, end + 1);

  navigator.clipboard.writeText(selectedLines.join("\n"));

  selectionStart.value = null;
  selectionEnd.value = null;
  lastClickIndex.value = null;
};

onMounted(() => {
  window.addEventListener("keydown", handleCopy);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleCopy);
  stopAutoScroll();
});

const lastMouseEvent = ref<MouseEvent | null>(null);

const hasSelection = computed(
  () =>
    selectionStart.value !== null &&
    selectionEnd.value !== null &&
    selectionStart.value !== selectionEnd.value,
);

const copySelectedLines = () => {
  if (!hasSelection.value) return;

  const start = Math.min(selectionStart.value!, selectionEnd.value!);
  const end = Math.max(selectionStart.value!, selectionEnd.value!);
  const selectedLines = activeOutput.value.slice(start, end + 1);

  navigator.clipboard.writeText(selectedLines.join("\n"));
};

const getSelectionPosition = () => {
  if (!hasSelection.value || !scrollContainer.value) return { start: 0, end: 0 };

  const start = Math.min(selectionStart.value!, selectionEnd.value!);
  const end = Math.max(selectionStart.value!, selectionEnd.value!);

  const totalLines = activeOutput.value.length;

  return {
    start: (start / totalLines) * 100,
    end: ((end + 1) / totalLines) * 100,
  };
};
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

[data-pyro-terminal-virtual-list] {
  will-change: transform;
  transform: translateZ(0);
}

[data-pyro-terminal-scrollbar-thumb] {
  will-change: transform;
  transform: translateZ(0);
}

[data-pyro-terminal-virtual-list] li {
  height: 32px;
  transition: background-color 0.1s ease;
}

.selected-line {
  background: color-mix(in srgb, var(--color-blue) 20%, transparent) !important;
  position: relative;
}

.selected-line::before {
  content: "";
  position: absolute;
  left: 0;
  top: -2px;
  bottom: -2px;
  width: 3px;
  border-radius: 0.5rem;
  background: var(--color-blue);
}

.selected-line:hover {
  background: color-mix(in srgb, var(--color-blue) 24%, transparent) !important;
}

[data-pyro-terminal-virtual-list] li {
  height: 32px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scrolling-active {
  pointer-events: none;
}

.scrolling-active li {
  pointer-events: none;
}

[data-pyro-terminal-scrollbar-track] {
  background: color-mix(in srgb, var(--color-bg) 50%, transparent);
  backdrop-filter: blur(4px);
  border-radius: 24px;
}

[data-pyro-terminal-scrollbar-thumb] {
  background: color-mix(in srgb, var(--color-contrast) 40%, transparent);
}
</style>
