<script setup lang="ts">
import { computed } from 'vue'
import AutoLink from '../base/AutoLink.vue'
import { MemoryStickIcon, DatabaseIcon, CpuIcon, SparklesIcon, UnknownIcon } from '@modrinth/assets'

const emit = defineEmits<{
  (e: 'click-bursting-link'): void
}>()

const props = withDefaults(
  defineProps<{
    ram: number
    storage: number
    cpus: number
    burstingLink?: string
  }>(),
  {
    burstingLink: undefined,
  },
)

const formattedRam = computed(() => {
  return props.ram / 1024
})

const formattedStorage = computed(() => {
  return props.storage / 1024
})

const sharedCpus = computed(() => {
  return props.cpus / 2
})
</script>
<template>
  <ul class="m-0 flex list-none flex-col gap-2 px-0 text-sm leading-normal text-secondary">
    <li class="flex items-center gap-2">
      <MemoryStickIcon class="h-5 w-5 shrink-0" /> {{ formattedRam }} GB RAM
    </li>
    <li class="flex items-center gap-2">
      <DatabaseIcon class="h-5 w-5 shrink-0" /> {{ formattedStorage }} GB Storage
    </li>
    <li class="flex items-center gap-2">
      <CpuIcon class="h-5 w-5 shrink-0" /> {{ sharedCpus }} Shared CPUs
    </li>
    <li class="flex items-center gap-2">
      <SparklesIcon class="h-5 w-5 shrink-0" /> Bursts up to {{ cpus }} CPUs
      <AutoLink
        v-if="burstingLink"
        v-tooltip="
          `CPU bursting allows your server to temporarily use additional threads to help mitigate TPS spikes. Click for more info.`
        "
        class="flex"
        :to="burstingLink"
        @click="() => emit('click-bursting-link')"
      >
        <UnknownIcon class="h-4 w-4 text-secondary opacity-80" />
      </AutoLink>
    </li>
  </ul>
</template>
