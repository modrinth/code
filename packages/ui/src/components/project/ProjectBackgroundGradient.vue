<template>
  <div :style="`--_color: ${color}`" />
</template>
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    project: {
      body: string
      color: number
    }
  }>(),
  {},
)

function clamp(value: number) {
  return Math.max(0, Math.min(255, value))
}

function toHex(value: number) {
  return clamp(value).toString(16).padStart(2, '0')
}

function decimalToHexColor(decimal: number) {
  const r = (decimal >> 16) & 255
  const g = (decimal >> 8) & 255
  const b = decimal & 255

  return `#${toHex(r)}${toHex(g)}${toHex(b)}`
}

const color = computed(() => {
  return decimalToHexColor(props.project.color)
})
</script>
<style scoped lang="scss">
div {
  width: 100%;
  height: 60rem;
  background: linear-gradient(to bottom, var(--_color), transparent);
  opacity: 0.075;
}
</style>
