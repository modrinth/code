<script setup lang="ts">
import { type Component, ref } from 'vue'
import { useVIntl, type MessageDescriptor } from '@vintl/vintl'

const { formatMessage } = useVIntl()

export type Tab<Props> = {
  name: MessageDescriptor
  icon: Component
  content: Component<Props>
  props?: Props
}

defineProps<{
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  tabs: Tab<any>[]
}>()

const selectedTab = ref(0)

function setTab(index: number) {
  selectedTab.value = index
}

defineExpose({ selectedTab, setTab })
</script>
<template>
  <div class="grid grid-cols-[auto_1fr]">
    <div
      class="flex flex-col gap-1 border-solid pr-4 border-0 border-r-[1px] border-divider min-w-[200px]"
    >
      <button
        v-for="(tab, index) in tabs"
        :key="index"
        :class="`flex gap-2 items-center text-left rounded-xl px-4 py-2 border-none text-nowrap font-semibold cursor-pointer active:scale-[0.97] transition-all ${selectedTab === index ? 'bg-button-bgSelected text-button-textSelected' : 'bg-transparent text-button-text hover:bg-button-bg hover:text-contrast'}`"
        @click="() => (selectedTab = index)"
      >
        <component :is="tab.icon" class="w-4 h-4" />
        <span>{{ formatMessage(tab.name) }}</span>
      </button>

      <slot name="footer" />
    </div>
    <div class="w-[600px] h-[500px] overflow-y-auto px-4">
      <component :is="tabs[selectedTab].content" v-bind="tabs[selectedTab].props ?? {}" />
    </div>
  </div>
</template>
