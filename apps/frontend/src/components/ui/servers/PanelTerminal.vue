<template>
  <div class="contents">
    <div class="flex items-center gap-4">
      <div class="relative w-full">
        <input
          v-model="searchInput"
          type="text"
          placeholder="Search logs"
          class="h-12 !w-full !pl-10 !pr-48"
          @keydown.escape="clearSearch"
        />
        <SearchIcon class="absolute left-4 top-1/2 -translate-y-1/2" />
        <ButtonStyled v-if="searchInput" @click="clearSearch">
          <button class="absolute right-2 top-1/2 -translate-y-1/2">
            <XIcon class="h-5 w-5" />
          </button>
        </ButtonStyled>
        <span
          v-if="pyroConsole.filteredOutput.value.length && searchInput"
          class="pointer-events-none absolute right-12 top-1/2 -translate-y-1/2 select-none whitespace-pre text-sm"
        >
          {{ pyroConsole.filteredOutput.value.length }}
          {{ pyroConsole.filteredOutput.value.length === 1 ? "result" : "results" }}
        </span>
      </div>
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
          @scroll.passive="() => handleListScroll()"
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
                <li
                  data-pyro-terminal-line
                  :class="{ 'selected-line': isLineSelected(visibleStartIndex + index) }"
                  class="group"
                >
                  <div class="flex items-center gap-2">
                    <UiServersLogLine :log="item" @show-full-log="showFullLogMessage" />
                    <div @mousedown.stop @click.stop>
                      <button
                        v-if="searchInput"
                        class="jump-button mr-4 flex items-center gap-1 rounded-md bg-bg-blue px-2 py-1 text-xs text-blue transition-all hover:scale-105 active:scale-95"
                        @mousedown.stop
                        @click.stop="() => jumpToLine(item)"
                      >
                        <RightArrowIcon class="h-3 w-3" />
                        Jump
                      </button>
                    </div>
                  </div>
                </li>
                <li
                  v-if="
                    searchInput &&
                    shouldShowSeparator(visibleStartIndex + index, visibleStartIndex + index + 1)
                  "
                  data-pyro-terminal-separator
                  class="flex h-8 select-none items-center justify-center opacity-50"
                  aria-hidden="true"
                >
                  <div class="h-[1px] w-full bg-contrast opacity-50"></div>
                  <div class="mx-4 flex flex-row items-center gap-4">
                    <div class="flex flex-row items-center gap-1">
                      <div class="size-1 rounded-full bg-contrast opacity-70"></div>
                      <div class="size-1 rounded-full bg-contrast opacity-70"></div>
                      <div class="size-1 rounded-full bg-contrast opacity-70"></div>
                    </div>
                    <span class="select-none whitespace-pre text-xs text-contrast">
                      {{
                        shouldShowSeparator(
                          visibleStartIndex + index,
                          visibleStartIndex + index + 1,
                        )
                      }}
                      line{{
                        shouldShowSeparator(
                          visibleStartIndex + index,
                          visibleStartIndex + index + 1,
                        ) === 1
                          ? ""
                          : "s"
                      }}
                      between
                    </span>
                  </div>
                  <div class="h-[1px] w-full bg-contrast opacity-50"></div>
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
        :class="{ hidden: searchInput || hasSelection || isSingleLineSelected }"
        @click="toggleFullscreen"
      >
        <LazyUiServersIconsMinimizeIconVue v-if="isFullScreen" />
        <LazyUiServersIconsFullscreenIcon v-else />
      </button>

      <Transition name="fade">
        <div
          v-if="hasSelection || isSingleLineSelected"
          class="absolute right-20 top-4 z-[3] flex flex-row items-center"
          :class="{ '!right-4': searchInput || hasSelection || isSingleLineSelected }"
        >
          <button
            data-pyro-copy
            label="Copy selected lines"
            class="experimental-styles-within flex h-12 flex-row items-center justify-center gap-2 rounded-full border-[1px] border-solid border-button-border bg-bg-raised px-4 text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
            @click="copySelectedLines"
          >
            <CopyIcon class="h-5 w-5" />
            <span class="">Copy</span>
          </button>
          <Transition name="view-button">
            <button
              v-if="hasSelection"
              data-pyro-view
              label="View selected lines"
              class="experimental-styles-within ml-2 flex h-12 w-[100px] flex-row items-center justify-center gap-2 rounded-full border-[1px] border-solid border-button-border bg-bg-raised px-4 text-contrast transition-all duration-200 hover:scale-110 active:scale-95"
              @click="showSelectedLines"
            >
              <EyeIcon class="h-5 w-5" />
              <span class="">View</span>
            </button>
          </Transition>
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
    <NewModal ref="viewLogModal" class="z-[9999]" header="Viewing selected logs">
      <div class="text-contrast">
        <pre
          class="select-text overflow-x-auto whitespace-pre rounded-lg bg-bg font-mono"
          v-html="processedLogWithLinks"
        ></pre>
        <div v-if="detectedLinks.length" class="border-contrast/20 mt-4 border-t pt-4">
          <h2>Detected Links</h2>
          <ul class="flex flex-col gap-2">
            <li v-for="(link, index) in detectedLinks" :key="index">
              <a
                :href="link"
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-blue hover:underline"
              >
                {{ link }}
              </a>
            </li>
          </ul>
        </div>
      </div>
    </NewModal>
  </div>
</template>

<script setup lang="ts">
import { RightArrowIcon, CopyIcon, XIcon, SearchIcon, EyeIcon } from "@modrinth/assets";
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useDebounceFn } from "@vueuse/core";
import { NewModal } from "@modrinth/ui";
import ButtonStyled from "@modrinth/ui/src/components/base/ButtonStyled.vue";
import DOMPurify from "dompurify";
import { usePyroConsole } from "~/store/console.ts";

const { $cosmetics } = useNuxtApp();
const cosmetics = $cosmetics;

const props = defineProps<{
  fullScreen: boolean;
}>();

const BUFFER_SIZE = 5;
const BATCH_SIZE = 50;
const LINE_HEIGHT = 32;
const SEPARATOR_HEIGHT = 32;
const SCROLL_END_DELAY = 150;
const progressiveBlurIterations = ref(8);

const pyroConsole = usePyroConsole();
const consoleOutput = pyroConsole.output;

const scrollContainer = ref<HTMLElement | null>(null);

const isFullScreen = ref(props.fullScreen);

const initialBatch = ref(false);
const isInitialLoad = ref(true);

const startY = ref(0);
const scrollTop = ref(0);
const clientHeight = ref(0);
const startScrollTop = ref(0);
const bottomThreshold = ref(0);
const isAutoScrolling = ref(false);
const userHasScrolled = ref(false);
const isScrolledToBottom = ref(true);
const scrollEndTimeout = ref<NodeJS.Timeout | null>(null);
const isScrolling = ref(false);

const scrollbarTrack = ref<HTMLElement | null>(null);
const scrollbarThumb = ref<HTMLElement | null>(null);
const isDragging = ref(false);

const searchInput = ref("");

const viewLogModal = ref<InstanceType<typeof NewModal>>();
const selectedLog = ref("");
const selectionStart = ref<number | null>(null);
const selectionEnd = ref<number | null>(null);
const isSelecting = ref(false);
const autoScrollSpeed = ref(0);
const autoScrollInterval = ref<NodeJS.Timeout | null>(null);
const lastClickIndex = ref<number | null>(null);
const lastMouseEvent = ref<MouseEvent | null>(null);

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

const clearSearch = () => {
  searchInput.value = "";
  pyroConsole.setSearchQuery("");
};

const activeOutput = computed(() => {
  return searchInput.value ? pyroConsole.filteredOutput.value : consoleOutput.value;
});

const shouldShowSeparator = (currentIndex: number, nextIndex: number) => {
  if (!searchInput.value || nextIndex >= activeOutput.value.length) return 0;

  const currentLine = activeOutput.value[currentIndex];
  const nextLine = activeOutput.value[nextIndex];

  const currentOriginalIndex = consoleOutput.value.indexOf(currentLine);
  const nextOriginalIndex = consoleOutput.value.indexOf(nextLine);

  const linesBetween = nextOriginalIndex - currentOriginalIndex - 1;
  return linesBetween > 0 ? linesBetween : 0;
};

const totalHeight = computed(() => {
  const baseHeight = activeOutput.value.length * LINE_HEIGHT;
  if (!searchInput.value) return baseHeight;

  let separatorCount = 0;
  for (let i = 0; i < activeOutput.value.length - 1; i++) {
    if (shouldShowSeparator(i, i + 1)) {
      separatorCount++;
    }
  }

  return baseHeight + separatorCount * SEPARATOR_HEIGHT;
});

const getLineIndexForPosition = (position: number) => {
  if (!searchInput.value) {
    return Math.floor(position / LINE_HEIGHT);
  }

  let accHeight = 0;
  let index = 0;

  while (index < activeOutput.value.length) {
    if (accHeight >= position) {
      break;
    }
    accHeight += LINE_HEIGHT;
    if (shouldShowSeparator(index, index + 1)) {
      accHeight += SEPARATOR_HEIGHT;
    }
    index++;
  }

  return index;
};

const getPositionForLineIndex = (index: number) => {
  if (!searchInput.value) {
    return index * LINE_HEIGHT;
  }

  let position = 0;
  for (let i = 0; i < index; i++) {
    position += LINE_HEIGHT;
    if (shouldShowSeparator(i, i + 1)) {
      position += SEPARATOR_HEIGHT;
    }
  }

  return position;
};

const visibleStartIndex = computed(() => {
  const rawPosition = Math.max(0, scrollTop.value - LINE_HEIGHT * BUFFER_SIZE);
  return getLineIndexForPosition(rawPosition);
});

const visibleEndIndex = computed(() => {
  const rawPosition = scrollTop.value + clientHeight.value + LINE_HEIGHT * BUFFER_SIZE;
  return Math.min(activeOutput.value.length - 1, getLineIndexForPosition(rawPosition));
});

const offsetY = computed(() => {
  return getPositionForLineIndex(visibleStartIndex.value);
});

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
  const trackHeight = trackRect.height;
  const clickRatio = (event.clientY - trackRect.top) / trackHeight;

  const maxScroll = scrollContainer.value.scrollHeight - scrollContainer.value.clientHeight;
  const newScrollTop = clickRatio * maxScroll;

  scrollContainer.value.scrollTop = Math.max(0, Math.min(newScrollTop, maxScroll));
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

    if (searchInput.value) {
      clearSearch();
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

const handleClickOutside = (event: MouseEvent) => {
  if (!event.target || !(event.target instanceof Node)) return;
  const terminalElement = document.querySelector("[data-pyro-terminal]");
  if (!terminalElement || !terminalElement.contains(event.target)) {
    selectionStart.value = null;
    selectionEnd.value = null;
    lastClickIndex.value = null;
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
        initialBatch.value = true;
      }
    });
  });
};

const showFullLogMessage = (log: string) => {
  selectedLog.value = log;
  viewLogModal.value?.show();
};

const showSelectedLines = () => {
  if (!hasSelection.value) return;

  const start = Math.min(selectionStart.value!, selectionEnd.value!);
  const end = Math.max(selectionStart.value!, selectionEnd.value!);
  const selectedLines = activeOutput.value.slice(start, end + 1);

  selectedLog.value = selectedLines.join("\n");
  viewLogModal.value?.show();
};

const virtualListStyle = computed(() => ({
  transform: `translateY(${offsetY.value}px)`,
}));

const isLineSelected = (index: number) => {
  if (selectionStart.value === null || selectionEnd.value === null) return false;
  const start = Math.min(selectionStart.value, selectionEnd.value);
  const end = Math.max(selectionStart.value, selectionEnd.value);
  return index >= start && index <= end;
};

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
  const rawY = event.clientY - rect.top + scrollContainer.value.scrollTop;

  if (searchInput.value) {
    const adjustedY = Math.max(0, rawY - 48);
    return Math.max(0, Math.min(getLineIndexForPosition(adjustedY), activeOutput.value.length - 1));
  }

  const adjustedY = Math.max(0, rawY - 24);
  return Math.floor(adjustedY / LINE_HEIGHT);
};

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

const hasSelection = computed(
  () =>
    selectionStart.value !== null &&
    selectionEnd.value !== null &&
    selectionStart.value !== selectionEnd.value,
);

const isSingleLineSelected = computed(
  () =>
    selectionStart.value !== null &&
    selectionEnd.value !== null &&
    selectionStart.value === selectionEnd.value,
);

const copySelectedLines = () => {
  if (!hasSelection.value && !isSingleLineSelected.value) return;

  let selectedLines;
  if (isSingleLineSelected.value) {
    const index = selectionStart.value!;
    selectedLines = [activeOutput.value[index]];
  } else {
    const start = Math.min(selectionStart.value!, selectionEnd.value!);
    const end = Math.max(selectionStart.value!, selectionEnd.value!);
    selectedLines = activeOutput.value.slice(start, end + 1);
  }

  navigator.clipboard.writeText(selectedLines.join("\n"));

  selectionStart.value = null;
  selectionEnd.value = null;
  lastClickIndex.value = null;
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

const jumpToLine = (line: string, event?: MouseEvent) => {
  event?.preventDefault();
  event?.stopPropagation();

  const index = pyroConsole.findLineIndex(line);
  if (index === -1) return;

  const filteredLineIndex = pyroConsole.filteredOutput.value.indexOf(line);
  if (filteredLineIndex === -1) return;

  let filteredTargetPosition = 0;
  for (let i = 0; i < filteredLineIndex; i++) {
    filteredTargetPosition += LINE_HEIGHT;
    if (shouldShowSeparator(i, i + 1)) {
      filteredTargetPosition += SEPARATOR_HEIGHT;
    }
  }

  const viewportHeight = scrollContainer.value?.clientHeight ?? 0;
  const currentScrollPosition = scrollContainer.value?.scrollTop ?? 0;
  const relativePosition = (filteredTargetPosition - currentScrollPosition) / viewportHeight;

  const targetLineRelativeOffset = relativePosition;
  const targetLine = line;

  clearSearch();

  nextTick(() => {
    if (!scrollContainer.value) return;

    const targetPosition = index * LINE_HEIGHT;
    const containerHeight = scrollContainer.value.clientHeight;
    const targetScrollTop = Math.max(
      0,
      targetPosition - containerHeight * targetLineRelativeOffset,
    );

    scrollContainer.value.scrollTop = targetScrollTop;

    requestAnimationFrame(() => {
      handleListScroll();

      const elements = scrollContainer.value?.getElementsByTagName("li");
      if (!elements) return;

      const targetElement = Array.from(elements).find(
        (el) =>
          el.textContent?.includes(targetLine) && !el.hasAttribute("data-pyro-terminal-separator"),
      );

      if (targetElement) {
        targetElement.classList.add("jumped-line");
        requestAnimationFrame(() => {
          targetElement.classList.add("jumped-line-active");
          setTimeout(() => {
            targetElement.classList.remove("jumped-line-active");
            targetElement.classList.remove("jumped-line");
          }, 4000);
        });
      }
    });
  });
};

const sanitizeUrl = (url: string): string => {
  try {
    const parsed = new URL(url);
    if (!["http:", "https:"].includes(parsed.protocol)) {
      return "#";
    }
    return parsed.toString();
  } catch {
    return "#";
  }
};

const detectedLinks = computed(() => {
  const urlRegex = /(https?:\/\/[^\s,<]+(?=[,\s<]|$))/g;
  const matches = [...selectedLog.value.matchAll(urlRegex)].map((match) => match[0]);
  return matches.filter((url) => sanitizeUrl(url) !== "#");
});

const processedLogWithLinks = computed(() => {
  const urlRegex = /(https?:\/\/[^\s,<]+(?=[,\s<]|$))/g;
  const sanitizedLog = DOMPurify.sanitize(selectedLog.value, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
  });

  return sanitizedLog.replace(urlRegex, (url) => {
    const safeUrl = sanitizeUrl(url);
    if (safeUrl === "#") return url;
    return `<a href="${safeUrl}" target="_blank" rel="noopener noreferrer nofollow" class="text-blue hover:underline">${url}</a>`;
  });
});

watch(
  () => pyroConsole.filteredOutput.value,
  () => {
    nextTick(() => {
      handleListScroll();
    });
  },
);

watch(searchInput, (value) => {
  updateSearch(value);
});

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

onMounted(() => {
  initializeTerminal();

  window.addEventListener("resize", updateClientHeight);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("mouseup", handleGlobalMouseUp);
  window.addEventListener("contextmenu", handleContextMenu);
  window.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateClientHeight);
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("mouseup", handleGlobalMouseUp);
  window.removeEventListener("contextmenu", handleContextMenu);
  window.removeEventListener("click", handleClickOutside);
  stopDragging();
  setBodyScroll(true);
  if (scrollEndTimeout.value) {
    clearTimeout(scrollEndTimeout.value);
  }
});

onMounted(() => {
  window.addEventListener("keydown", handleCopy);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleCopy);
  stopAutoScroll();
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
  border-radius: 24px;
}

[data-pyro-terminal-scrollbar-thumb] {
  background: color-mix(in srgb, var(--color-contrast) 40%, transparent);
}

.view-button-enter-active {
  transition: all 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
  width: 100px;
  margin-left: 0.5rem;
  overflow: hidden;
}

.view-button-leave-active {
  transition: all 200ms cubic-bezier(0.4, 0, 1, 1);
  overflow: hidden;
}

.view-button-enter-from,
.view-button-leave-to {
  opacity: 0;
  width: 0;
  margin-left: 0;
  padding-left: 0;
  padding-right: 0;
}

.view-button-enter-active,
.view-button-leave-active {
  transition: all 200ms ease;
  width: 100px;
  margin-left: 0.5rem;
}

.view-button-enter-from,
.view-button-leave-to {
  opacity: 0;
  width: 0;
  margin-left: 0;
}

.jumped-line {
  position: relative;
  z-index: 1;
}

.jumped-line-active {
  animation: highlight-jump 2s ease;
}

@keyframes highlight-jump {
  0% {
    background: transparent;
  }
  15%,
  85% {
    background: color-mix(in srgb, var(--color-blue) 15%, transparent);
  }
  100% {
    background: transparent;
  }
}

.jump-button {
  opacity: 0.4;
  transition: all 0.2s ease;
}

.group:hover .jump-button {
  opacity: 0.8;
}

.group:hover .jump-button:hover {
  opacity: 1;
}
</style>
