<script setup lang="ts">
import { RadioButtonIcon, RadioButtonCheckedIcon } from '@modrinth/assets'

withDefaults(
  defineProps<{
    checked: boolean
  }>(),
  {
    checked: false,
  },
)
</script>
<template>
  <div class="" role="button" @click="() => {}">
    <slot name="preview" />
    <div>
      <RadioButtonIcon v-if="!checked" class="w-4 h-4" />
      <RadioButtonCheckedIcon v-else class="w-4 h-4" />
      <slot />
    </div>
  </div>
</template>
