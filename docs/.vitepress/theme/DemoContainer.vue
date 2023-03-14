<template>
  <div class="demo"><slot /></div>
</template>

<script>
export default {}
</script>

<style lang="scss" scoped>
.demo {
  background: var(--color-raised-bg);
  border: 1px solid var(--color-button-bg);
  border-radius: var(--gap-lg);
  display: flex;
  padding: var(--gap-md);
  gap: var(--gap-md);
  align-items: center;
  flex-wrap: wrap;
}
</style>
