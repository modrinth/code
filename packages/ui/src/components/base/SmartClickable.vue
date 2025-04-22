<template>
  <div class="smart-clickable" :class="{ 'smart-clickable--has-clickable': !!$slots.clickable }">
    <slot name="clickable" />
    <div v-bind="$attrs" class="smart-clickable__contents pointer-events-none">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
})
</script>
<style scoped lang="scss">
.smart-clickable {
  display: grid;

  > * {
    grid-area: 1 / 1;
  }

  .smart-clickable__contents {
    // Utility classes for contents
    :deep(.smart-clickable\:allow-pointer-events) {
      pointer-events: all;
    }
  }
}

// Only apply effects when a clickable is present
.smart-clickable.smart-clickable--has-clickable {
  // Setup base styles for contents
  .smart-clickable__contents {
    transition: scale 0.125s ease-out;

    // Why? I don't know. It forces the SVGs to render differently, which fixes some shift on hover otherwise.
    //filter: brightness(1.00001);
  }

  // When clickable is being hovered or focus-visible, give contents an effect
  &:has(> *:first-child:hover, > *:first-child:focus-visible) .smart-clickable__contents {
    filter: var(--hover-filter-weak);

    // Utility classes for contents
    :deep(.smart-clickable\:underline-on-hover) {
      text-decoration: underline;
    }

    // Utility classes for contents
    :deep(.smart-clickable\:highlight-on-hover) {
      filter: brightness(var(--hover-brightness, 1.25));
    }
  }

  // When clickable is being clicked, give contents an effect
  &:has(> *:first-child:active) .smart-clickable__contents {
    scale: 0.97;
  }
}
</style>
