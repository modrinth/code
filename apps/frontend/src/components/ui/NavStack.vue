<template>
  <nav>
    <ul>
      <slot />
    </ul>
  </nav>
</template>

<script>
export default {}
</script>

<style lang="scss" scoped>
ul {
  display: flex;
  flex-direction: column;
  grid-gap: var(--spacing-card-xs);
  flex-wrap: wrap;
  list-style-type: none;
  margin: 0;
  padding: 0;

  > :first-child {
    margin-top: 0;
  }
}

li {
  display: unset;
  text-align: unset;
}
</style>
