<template>
  <div ref="listContainer" data-pyro-files-virtual-list-root class="relative w-full">
    <div
      :style="{
        position: 'relative',
        minHeight: `${totalHeight}px`,
      }"
      data-pyro-files-virtual-height-watcher
    >
      <ul
        class="list-none"
        :style="{
          position: 'absolute',
          top: `${visibleTop}px`,
          width: '100%',
          margin: 0,
          padding: 0,
        }"
        data-pyro-files-virtual-list
      >
        <UiServersFileItem
          v-for="item in visibleItems"
          :key="item.path"
          :count="item.count"
          :created="item.created"
          :modified="item.modified"
          :name="item.name"
          :path="item.path"
          :type="item.type"
          :size="item.size"
          @delete="$emit('delete', item)"
          @rename="$emit('rename', item)"
          @extract="$emit('extract', item)"
          @download="$emit('download', item)"
          @move="$emit('move', item)"
          @move-direct-to="$emit('moveDirectTo', $event)"
          @edit="$emit('edit', item)"
          @contextmenu="(x, y) => $emit('contextmenu', item, x, y)"
        />
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

const props = defineProps<{
  items: any[];
}>();

const emit = defineEmits<{
  (
    e: "delete" | "rename" | "download" | "move" | "edit" | "moveDirectTo" | "extract",
    item: any,
  ): void;
  (e: "contextmenu", item: any, x: number, y: number): void;
  (e: "loadMore"): void;
}>();

const ITEM_HEIGHT = 61;
const BUFFER_SIZE = 5;

const listContainer = ref<HTMLElement | null>(null);
const windowScrollY = ref(0);
const windowHeight = ref(0);

const totalHeight = computed(() => props.items.length * ITEM_HEIGHT);

const visibleRange = computed(() => {
  if (!listContainer.value) return { start: 0, end: 0 };

  const containerTop = listContainer.value.getBoundingClientRect().top + window.scrollY;
  const relativeScrollTop = Math.max(0, windowScrollY.value - containerTop);

  const start = Math.floor(relativeScrollTop / ITEM_HEIGHT);
  const visibleCount = Math.ceil(windowHeight.value / ITEM_HEIGHT);

  return {
    start: Math.max(0, start - BUFFER_SIZE),
    end: Math.min(props.items.length, start + visibleCount + BUFFER_SIZE * 2),
  };
});

const visibleTop = computed(() => {
  return visibleRange.value.start * ITEM_HEIGHT;
});

const visibleItems = computed(() => {
  return props.items.slice(visibleRange.value.start, visibleRange.value.end);
});

const handleScroll = () => {
  windowScrollY.value = window.scrollY;

  if (!listContainer.value) return;

  const containerBottom = listContainer.value.getBoundingClientRect().bottom;
  const remainingScroll = containerBottom - window.innerHeight;

  if (remainingScroll < windowHeight.value * 0.2) {
    emit("loadMore");
  }
};

const handleResize = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  windowHeight.value = window.innerHeight;
  window.addEventListener("scroll", handleScroll, { passive: true });
  window.addEventListener("resize", handleResize, { passive: true });
  handleScroll();
});

onUnmounted(() => {
  window.removeEventListener("scroll", handleScroll);
  window.removeEventListener("resize", handleResize);
});
</script>
