<template>
  <div class="main">
    <h1>{{ error.message }}</h1>
    <NuxtLink to="/">
      An error occurred! Click this text to go back home, and find your way
      back!
    </NuxtLink>
  </div>
</template>

<script>
export default {
  props: ['error'],
  layout: 'home',
  created() {
    console.log(this.error)
  },
}
</script>

<style lang="scss" scoped>
.main {
  margin: 0 auto;
  max-width: 800px;
  text-align: center;
}
</style>
