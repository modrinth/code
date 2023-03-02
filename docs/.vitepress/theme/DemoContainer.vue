<template>
  <div class="demo bg-raised radius-lg padding-md"><slot /></div>
</template>

<script>
export default {}
</script>

<style lang="scss" scoped>
.demo {
  border: 1px solid var(--color-button-bg);
  display: flex;
  gap: var(--gap-md);
  align-items: center;
  flex-wrap: wrap;
}
</style>
