<template>
  <div class="flex flex-wrap gap-2">
    <button
      v-for="(item, index) in items"
      :key="`radio-button-${index}`"
      class="p-0 py-2 px-2 border-0 flex gap-2 transition-all items-center cursor-pointer active:scale-95 hover:bg-button-bg rounded-xl"
      :class="{
        'text-contrast font-medium bg-button-bg': selected === item,
        'text-primary bg-transparent': selected !== item,
      }"
      @click="selected = item"
    >
      <RadioButtonCheckedIcon v-if="selected === item" class="text-brand h-5 w-5" />
      <RadioButtonIcon v-else class="h-5 w-5" />
      <slot :item="item" />
    </button>
  </div>
</template>
<script setup lang="ts" generic="T">
import { RadioButtonIcon, RadioButtonCheckedIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: T
    items: T[]
    forceSelection?: boolean
  }>(),
  {
    forceSelection: false,
  },
)

const emit = defineEmits(['update:modelValue'])

const selected = computed({
  get() {
    return props.modelValue
  },
  set(value) {
    emit('update:modelValue', value)
  },
})

if (props.items.length > 0 && props.forceSelection && !props.modelValue) {
  selected.value = props.items[0]
}
</script>
