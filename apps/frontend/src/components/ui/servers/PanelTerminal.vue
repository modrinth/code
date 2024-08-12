<template>
  <div
    class="monocraft-font console relative h-full w-full overflow-hidden rounded-xl bg-black p-6 text-sm"
  >
    <div class="h-full overflow-y-auto">
      <VirtualScroller
        ref="scroller"
        :default-size="30"
        :items="consoleOutput"
        style="white-space: pre; word-wrap: break-word; width: 100%; line-height: 170%"
      >
        <template #item="{ index, offset, ref }">
          <LogParser v-if="ref" :log="ref" />
        </template>
      </VirtualScroller>
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
import { ref } from "vue";
import { ExpandIcon } from "@modrinth/assets";
import { createVirtualScroller } from "vue-typed-virtual-list";
import LogParser from "~/components/ui/servers/LogParser.vue";

const VirtualScroller = createVirtualScroller<string>();

defineProps<{
  consoleOutput: string[];
  fullScreen: boolean;
}>();

defineEmits<{
  (e: "toggle-full-screen"): void;
}>();

const scroller = ref<InstanceType<typeof VirtualScroller> | null>(null);
</script>
