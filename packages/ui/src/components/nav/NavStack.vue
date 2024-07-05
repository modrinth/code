<template>
  <div class="omorphia__navstack">
    <slot />
  </div>
</template>

<style lang="scss" scoped>
.omorphia__navstack {
  display: flex;
  flex-direction: column;

  :deep(.btn) {
    position: relative;
    width: 100%;

    &.selected {
      background-color: var(--color-button-bg);
      font-weight: bold;
    }
  }
}
</style>
