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
  box-shadow: 0px 0px 8px 3px #00000026;
}

.dark-mode {
  .nav-container {
    background: #17191c;
  }
}
</style>
