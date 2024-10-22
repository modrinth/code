<template>
  <div
    class="parsed-log group relative w-full overflow-hidden px-6 py-1"
    @mouseenter="showCopyButton = true"
    @mouseleave="showCopyButton = false"
  >
    <div
      ref="logContent"
      class="log-content whitespace-pre-wrap selection:bg-black selection:text-white dark:selection:bg-white dark:selection:text-black"
      v-html="sanitizedLog"
    ></div>
    <Transition name="fade">
      <button
        v-show="showCopyButton"
        class="absolute right-4 top-1/2 -translate-y-1/2 select-none rounded-md bg-bg-raised px-2 py-1 text-xs text-contrast opacity-0 transition-opacity duration-150 hover:bg-button-bg group-hover:opacity-100"
        aria-label="Copy line"
        @click="copyLog"
      >
        <span v-if="!copied" class="select-none">Copy</span>
        <span v-else class="select-none">Copied!</span>
      </button>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Convert from "ansi-to-html";
import DOMPurify from "dompurify";

const props = defineProps<{
  log: string;
}>();

const logContent = ref<HTMLElement | null>(null);
const showCopyButton = ref(false);
const copied = ref(false);

const colors = {
  30: "#101010",
  31: "#EFA6A2",
  32: "#80C990",
  33: "#A69460",
  34: "#A3B8EF",
  35: "#E6A3DC",
  36: "#50CACD",
  37: "#808080",
  90: "#454545",
  91: "#E0AF85",
  92: "#5ACCAF",
  93: "#C8C874",
  94: "#CCACED",
  95: "#F2A1C2",
  96: "#74C3E4",
  97: "#C0C0C0",
};

const convert = new Convert({
  fg: "#FFF",
  bg: "#000",
  newline: false,
  escapeXML: true,
  stream: false,
  colors,
});

const urlRegex = /https?:\/\/[^\s]+/g;
const usernameRegex = /&lt;([^&]+)&gt;/g;

const sanitizedLog = computed(() => {
  let html = convert.toHtml(props.log);
  html = html.replace(
    urlRegex,
    (url) =>
      `<a style="color:var(--color-link);text-decoration:underline;" href="${url}" target="_blank" rel="noopener noreferrer">${url}</a>`,
  );
  html = html.replace(
    usernameRegex,
    (_, username) => `<span class="minecraft-username">&lt;${username}&gt;</span>`,
  );
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["span", "a"],
    ALLOWED_ATTR: ["style", "href", "target", "rel", "class"],
    ADD_ATTR: ["target"],
    RETURN_TRUSTED_TYPE: true,
    USE_PROFILES: { html: true },
  });
});

const copyLog = async () => {
  if (!logContent.value) return;

  try {
    const textContent = logContent.value.textContent || "";
    await navigator.clipboard.writeText(textContent);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.error("Failed to copy text:", err);
  }
};
</script>

<style scoped>
.parsed-log:hover {
  transition: none;
}

.parsed-log {
  transition: 80ms;
}

html.light-mode .parsed-log:hover {
  background-color: #ccc;
}

html.dark-mode .parsed-log:hover {
  background-color: #222;
}

html.oled-mode .parsed-log:hover {
  background-color: #222;
}

.minecraft-username {
  font-weight: bold;
}

.fade-enter-active,
.fade-leave-active {
  transition: 80ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

::v-deep(.log-content) {
  user-select: text;
}

::v-deep(.log-content *) {
  user-select: text;
}
</style>
