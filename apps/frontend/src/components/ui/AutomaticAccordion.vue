<template>
  <div class="accordion-wrapper">
    <div class="accordion-content">
      <div>
        <div class="content-container" v-bind="$attrs">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});
</script>
<style scoped>
.accordion-content {
  display: grid;
  grid-template-rows: 1fr;
  transition: grid-template-rows 0.3s ease-in-out;
}

@media (prefers-reduced-motion) {
  .accordion-content {
    transition: none !important;
  }
}

.accordion-content:has(* .content-container:empty) {
  grid-template-rows: 0fr;
}

.accordion-content > div {
  overflow: hidden;
}

.accordion-wrapper:has(* .content-container:empty) {
  display: contents;
}
</style>
