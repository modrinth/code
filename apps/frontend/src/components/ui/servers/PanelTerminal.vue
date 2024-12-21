<template>
  <div
    data-pyro-terminal
    :class="[
      'terminal-font console relative z-[1] flex h-full w-full flex-col items-center justify-between !bg-white px-1 text-sm transition-transform duration-300',
    ]"
    tabindex="-1"
  >
    <div aria-hidden="true" class="pointer-events-none absolute left-0 top-0 z-[60] h-full w-full">
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
      :style="{
        height: `${height}px`,
        filter: `blur(${resizing ? 8 : 0}px)`,
        transition: `filter 0.2s`,
      }"
      data-pyro-terminal-scroll-root
      class="relative w-full"
      @wheel="wheel"
      @mousedown="mouseDown"
    >
      <canvas
        ref="terminalCanvas"
        :style="{
          objectPosition: 'top left',
          height: `${height}px`,
        }"
        class="absolute -top-0.5 left-0 w-full object-none"
      />
    </div>

    <div
      class="absolute bottom-4 z-[3] w-full"
      :style="{
        filter: `drop-shadow(0 8px 12px rgba(0, 0, 0, 0.1))`,
      }"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { usePyroConsole } from "@/store/console.ts";
import TerminalWorker from "@/workers/console.ts?worker";

const props = defineProps<{ search: string }>();

const pyroConsole = usePyroConsole();
const terminalCanvas = ref<HTMLCanvasElement | null>(null);
const worker = ref<Worker | null>(null);
const resizing = ref(false);
const resizeTimeout = ref<ReturnType<typeof setTimeout> | null>(null);
const height = ref(0);
const isMouseDown = ref(false);
const observer = ref<ResizeObserver | null>(null);
const terminalListenerId = ref<string | null>(null);

const mouseDown = (e: MouseEvent) => {
  if (!terminalCanvas.value || !worker.value) return;
  const rect = terminalCanvas.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const clientWidth = rect.width;
  const clientHeight = rect.height;
  worker.value.postMessage({
    type: "mousedown",
    x,
    y,
    clientWidth,
    clientHeight,
  });
  isMouseDown.value = true;
};

const mouseMove = (e: MouseEvent) => {
  if (!isMouseDown.value || !terminalCanvas.value || !worker.value) return;
  const rect = terminalCanvas.value.getBoundingClientRect();
  const y = e.clientY - rect.top;
  const clientHeight = rect.height;
  worker.value.postMessage({
    type: "mousemove",
    y,
    clientHeight,
  });
};

const mouseUp = (e: MouseEvent) => {
  e.preventDefault();
  if (!worker.value) return;
  isMouseDown.value = false;
  worker.value.postMessage({
    type: "mouseup",
  });
};

const wheel = (e: WheelEvent) => {
  e.preventDefault();
  if (!worker.value) return;
  worker.value.postMessage({
    type: "wheel",
    deltaY: e.deltaY,
  });
};

watch(
  () => props.search,
  (search) => {
    if (!worker.value) return;
    worker.value.postMessage({
      type: "search",
      query: search,
    });
  },
);

onMounted(() => {
  if (!terminalCanvas.value) return;
  window.addEventListener("mousemove", mouseMove);
  window.addEventListener("mouseup", mouseUp);
  worker.value = new TerminalWorker();
  const offscreenCanvas = terminalCanvas.value.transferControlToOffscreen();
  worker.value!.onmessage = (e) => {
    const { type } = e.data;
    switch (type) {
      case "init": {
        if (!worker.value) return;
        const { height: h } = e.data;
        height.value = h;
        break;
      }

      case "resize": {
        if (!terminalCanvas.value) return;
        const { width } = e.data;
        if (width === terminalCanvas.value.clientWidth) resizing.value = false;
        break;
      }
    }
  };

  worker.value!.postMessage({ type: "init", canvas: offscreenCanvas }, [offscreenCanvas]);
  terminalListenerId.value = pyroConsole.addListener((l) => {
    if (!terminalCanvas.value || !worker.value) return;
    worker.value.postMessage({
      type: "line",
      text: l,
    });
  });
  height.value = terminalCanvas.value.height;
  observer.value = new ResizeObserver(() => {
    if (!terminalCanvas.value) return;
    // worker.postMessage({ type: 'resize', width: consoleCanvas.clientWidth });
    if (resizeTimeout.value) clearTimeout(resizeTimeout.value);
    height.value = terminalCanvas.value.height;
    resizing.value = true;
    resizeTimeout.value = setTimeout(() => {
      worker.value?.postMessage({ type: "resize", width: terminalCanvas.value!.clientWidth });
      if (resizeTimeout.value) clearTimeout(resizeTimeout.value);
    }, 100);
  });

  observer.value.observe(terminalCanvas.value);
});

onUnmounted(() => {
  if (worker.value) {
    worker.value.terminate();
  }
  if (observer.value) {
    observer.value.disconnect();
  }
  if (resizeTimeout.value) {
    clearTimeout(resizeTimeout.value);
  }
  if (terminalListenerId.value) {
    pyroConsole.removeListener(terminalListenerId.value);
  }
  window.removeEventListener("mousemove", mouseMove);
  window.removeEventListener("mouseup", mouseUp);
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
</style>
