<template>
  <div
    class="parsed-log w-full overflow-hidden whitespace-nowrap text-wrap px-6 py-1 selection:bg-black selection:text-white dark:selection:bg-white dark:selection:text-black"
    v-html="sanitizedLog"
  ></div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Convert from "ansi-to-html";
import DOMPurify from "dompurify";

const props = defineProps<{
  log: string;
}>();

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
</script>

<style scoped>
html.light-mode .parsed-log:hover {
  background-color: #ccc;
}

html.dark-mode .parsed-log:hover {
  background-color: #333;
}

html.oled-mode .parsed-log:hover {
  background-color: #333;
}

.minecraft-username {
  font-weight: bold;
}
</style>
