<template>
  <div class="iconified-input flex-1">
    <SearchIcon />
    <input
      :value="modelValue"
      @input="$emit('update:modelValue', $event.target.value)"
      type="text"
      :placeholder="placeholder"
    />
    <Button v-if="modelValue" class="r-btn" @click="$emit('clear')">
      <XIcon />
    </Button>
  </div>
</template>

<script setup>
import { SearchIcon, XIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'

defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  placeholder: {
    type: String,
    default: 'Search...',
  },
})

defineEmits(['update:modelValue', 'clear'])
</script>
