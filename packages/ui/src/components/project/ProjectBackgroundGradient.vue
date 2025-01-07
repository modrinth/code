<template>
  <div :style="{ '--_accent-color': rgbToOklchHue(props.project.color) }" />
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

</script>
<style scoped lang="scss">
div {
  width: 100%;
  height: 60rem;
  background: linear-gradient(to bottom, oklch(40% 10% var(--_accent-color) / 25%), transparent);
  opacity: 1;
}
</style>
