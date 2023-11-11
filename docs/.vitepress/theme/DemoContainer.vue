<template>
  <div class="demo"><slot /></div>
</template>

<style lang="scss" scoped>
.demo {
  background: var(--color-raised-bg);
  border: 1px solid var(--color-button-bg);
  display: flex;
  padding: var(--gap-md);
  gap: var(--gap-md);
  align-items: center;
  flex-wrap: wrap;
  border-radius: var(--radius-lg);
}
</style>
