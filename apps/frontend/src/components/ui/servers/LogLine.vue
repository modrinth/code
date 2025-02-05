<template>
  <div
    class="parsed-log relative flex h-8 w-full items-center overflow-hidden px-6"
    @mouseenter="checkOverflow"
    @touchstart="checkOverflow"
  >
    <div ref="logContent" class="log-content flex-1 truncate whitespace-nowrap">
      <span v-html="sanitizedLog"></span>
    </div>
    <button
      v-if="isOverflowing"
      class="ml-2 flex h-6 items-center rounded-md bg-bg px-2 text-xs opacity-50 transition-opacity hover:opacity-100"
      type="button"
      @click.stop="$emit('show-full-log', props.log)"
    >
      ...
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Convert from "ansi-to-html";
import DOMPurify from "dompurify";

const props = defineProps<{
  log: string;
}>();

defineEmits<{
  "show-full-log": [log: string];
}>();

const logContent = ref<HTMLElement | null>(null);
const isOverflowing = ref(false);

const checkOverflow = () => {
  if (logContent.value && !isOverflowing.value) {
    isOverflowing.value = logContent.value.scrollWidth > logContent.value.clientWidth;
  }
};

const convert = new Convert({
  fg: "#FFF",
  bg: "#000",
  newline: false,
  escapeXML: true,
  stream: false,
});

const sanitizedLog = computed(() =>
  DOMPurify.sanitize(convert.toHtml(props.log), {
    ALLOWED_TAGS: ["span"],
    ALLOWED_ATTR: ["style"],
    USE_PROFILES: { html: true },
  }),
);
</script>

<style scoped>
.parsed-log {
  background: transparent;
  transition: background-color 0.2s;
}

.parsed-log:hover {
  background: rgba(128, 128, 128, 0.1);
}

:deep(.log-content) {
  user-select: text;
}
</style>
