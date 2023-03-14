<template>
  <div class="nav-container">
    <slot />
  </div>
</template>

<style lang="scss" scoped>
.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 210px;
  height: 100vh;
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
}

.dark-mode {
  .nav-container {
    background: var(--color-bg);
  }
}
</style>
