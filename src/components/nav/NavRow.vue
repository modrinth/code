<script setup>
defineProps({});
</script>

<template>
  <div class="omorphia__navrow">
    <slot />
    <div class="margin-left-auto">
      <slot name="right" />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.omorphia__navrow {
  display: flex;
  flex-direction: row;

  :deep(.omorphia__button) {
    position: relative;

    &::after {
      content: "";
      position: absolute;
      width: calc(100% - var(--gap-lg) * 2);
      height: 4px;
      bottom: 4px;
      border-radius: var(--radius-max);
      background-color: var(--color-brand);
      opacity: 0;
    }

    &:hover::after {
      opacity: 50%;
    }

    &.selected {
      color: var(--color-contrast);
      font-weight: bold;

      &::after {
        opacity: 1;
      }
    }
  }
}
</style>
