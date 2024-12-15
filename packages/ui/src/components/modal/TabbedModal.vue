<script setup lang="ts">
import { type Component, ref } from 'vue'
import { useVIntl, type MessageDescriptor } from '@vintl/vintl'

const { formatMessage } = useVIntl()

type Tab = {
  name: MessageDescriptor
  icon: Component
  content: Component
}

defineProps<{
  tabs: Tab[]
}>()

const selectedTab = ref(0)

function setTab(index: number) {
  selectedTab.value = index
}

defineExpose({ selectedTab, setTab })
</script>
<template>
  <div class="grid grid-cols-[auto_1fr] gap-4">
    <div class="flex flex-col gap-1 border-solid pr-4 border-0 border-r-[1px] border-divider">
      <button
        v-for="(tab, index) in tabs"
        :key="index"
        :class="`flex gap-2 items-center text-left rounded-xl px-4 py-2 border-none text-nowrap font-semibold cursor-pointer active:scale-[0.97] transition-transform ${selectedTab === index ? 'bg-highlight text-brand' : 'bg-transparent text-button-text'}`"
        @click="() => (selectedTab = index)"
      >
        <component :is="tab.icon" class="w-4 h-4" />
        <span>{{ formatMessage(tab.name) }}</span>
      </button>

      <slot name="footer" />
    </div>
    <div class="w-[600px] h-[500px] overflow-y-auto">
      <component :is="tabs[selectedTab].content" />
    </div>
  </div>
</template>
