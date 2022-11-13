<template>
  <nav class="navigation">
    <slot />
  </nav>
</template>

<script>
export default {
  name: 'NavStack',
}
</script>

<style lang="scss" scoped>
.navigation {
  display: flex;
  flex-direction: column;
  grid-gap: var(--spacing-card-xs);
  flex-wrap: wrap;
}
</style>
