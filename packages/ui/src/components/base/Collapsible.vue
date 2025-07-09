<template>
  <div class="accordion-content" :class="(baseClass ?? ``) + (collapsed ? `` : ` open`)">
    <div v-bind="$attrs" :inert="collapsed">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  baseClass?: string
  collapsed: boolean
}>()

defineOptions({
  inheritAttrs: false,
})
</script>
<style scoped>
.accordion-content {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.3s ease-in-out;
}

@media (prefers-reduced-motion) {
  .accordion-content {
    transition: none !important;
  }
}

.accordion-content.open {
  grid-template-rows: 1fr;
}

.accordion-content > div {
  overflow: hidden;
}
</style>
