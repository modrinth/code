<template>
  <div class="double-icon">
    <slot name="primary" />
    <div class="secondary">
      <slot name="secondary" />
    </div>
  </div>
</template>
<style lang="scss" scoped>
.double-icon {
  position: relative;
  height: fit-content;
  line-height: 0;

  .secondary {
    position: absolute;
    bottom: -4px;
    right: -4px;
    background-color: var(--color-bg);
    padding: var(--spacing-card-xs);
    border-radius: 50%;
    aspect-ratio: 1 / 1;
    width: fit-content;
    height: fit-content;
    line-height: 0;

    svg {
      width: 1rem;
      height: 1rem;
    }
  }
}
</style>
