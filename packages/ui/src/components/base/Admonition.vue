<template>
  <div
    :class="[
      'flex rounded-2xl border-2 p-4 gap-4 font-semibold text-contrast',
      typeClasses[type],
    ]"
  >
    <component
      :is="icons[type]"
      :class="['hidden h-8 w-8 flex-none sm:block', iconClasses[type]]"
    />
    <div class="flex flex-col gap-2">
      <div class="font-semibold flex justify-between gap-4">
        <slot name="header">{{ header }}</slot>
      </div>
      <div class="font-normal">
        <slot>{{ body }}</slot>
      </div>
    </div>
    <div class="ml-auto w-fit">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { InfoIcon, IssuesIcon, XCircleIcon } from '@modrinth/assets'

defineProps({
  type: {
    type: String as () => 'info' | 'warning' | 'critical',
    default: 'info',
  },
  header: {
    type: String,
    default: '',
  },
  body: {
    type: String,
    default: '',
  },
})

const typeClasses = {
  info: 'border-blue bg-bg-blue',
  warning: 'border-orange bg-bg-orange',
  critical: 'border-red bg-bg-red',
}

const iconClasses = {
  info: 'text-blue',
  warning: 'text-orange',
  critical: 'text-red',
}

const icons = {
  info: InfoIcon,
  warning: IssuesIcon,
  critical: XCircleIcon,
}
</script>
