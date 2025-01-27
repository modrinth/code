<template>
  <div :style="{ '--_accent-color': color }" />
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { rgbToOklchHue } from '@modrinth/utils'

const props = withDefaults(
  defineProps<{
    project: {
      body: string
      color: number
    }
  }>(),
  {},
)

const color = computed(() => rgbToOklchHue(props.project.color))

</script>
<style scoped lang="scss">
div {
  width: 100%;
  height: 60rem;
  background: linear-gradient(to bottom, oklch(40% 30% var(--_accent-color) / 10%), transparent);
  opacity: 1;
}
</style>
