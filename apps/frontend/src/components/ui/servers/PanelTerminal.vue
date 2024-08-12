<template>
  <div
    data-pyro-terminal-holy-shit
    class="monocraft-font console relative h-full w-full overflow-hidden rounded-xl bg-black text-sm"
  >
    <div
      data-pyro-terminal-root
      ref="scrollContainer"
      class="h-full w-full overflow-x-auto overflow-y-auto py-6"
      @scroll="handleScroll"
    >
      <div data-pyro-terminal-virtual-height-watcher :style="{ height: `${totalHeight}px` }">
        <ul
          class="m-0 list-none p-0"
          data-pyro-terminal-virtual-list
          :style="{ transform: `translateY(${offsetY}px)` }"
        >
          <template v-for="(item, index) in visibleItems" :key="index">
            <li
              class="relative w-full list-none"
              :data-pyro-terminal-recycle-tracker="index"
              ref="itemRefs"
            >
              <LogParser :log="item" />
            </li>
          </template>
        </ul>
      </div>
    </div>
    <button
      class="absolute right-8 top-8 bg-transparent transition-transform duration-300 hover:scale-110"
      @click="$emit('toggle-full-screen')"
    >
      <ExpandIcon />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { ExpandIcon } from "@modrinth/assets";
import LogParser from "~/components/ui/servers/LogParser.vue";

const props = defineProps<{
  consoleOutput: string[];
  fullScreen: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-full-screen"): void;
}>();

const scrollContainer = ref<HTMLElement | null>(null);
const itemRefs = ref<HTMLElement[]>([]);
const itemHeights = ref<number[]>([]);
const averageItemHeight = ref(30);
const bufferSize = 5;

const scrollTop = ref(0);
const clientHeight = ref(0);

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

onMounted(() => {
  updateClientHeight();
  updateItemHeights();
  window.addEventListener("resize", updateClientHeight);
  scrollToBottom();
});

onUnmounted(() => {
  window.removeEventListener("resize", updateClientHeight);
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
.monocraft-font {
  font-family: "Monocraft", monospace;
}
</style>
