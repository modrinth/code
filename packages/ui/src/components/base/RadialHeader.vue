<template>
  <div>
    <div :style="colorClasses" class="radial-header relative" v-bind="$attrs">
      <slot />
    </div>
    <div class="radial-header-divider" />
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<{
    color?: 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple' | 'gray'
  }>(),
  {
    color: 'brand',
  },
)

const colorClasses = computed(
  () =>
    `--_radial-bg: var(--color-${props.color}-highlight);--_radial-border: var(--color-${props.color});`,
)
</script>
<style scoped lang="scss">
.radial-header {
  background-image: radial-gradient(50% 100% at 50% 100%, var(--_radial-bg) 10%, #ffffff00 100%);
  position: relative;

  &::before {
    content: '';
    position: absolute;
    left: 0;
    bottom: 0;
    background-image: linear-gradient(
      90deg,
      #ffffff00 0%,
      var(--_radial-border) 50%,
      #ffffff00 100%
    );
    width: 100%;
    height: 1px;
    opacity: 0.8;
  }
}
</style>
