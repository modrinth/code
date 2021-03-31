<template>
  <div class="main">
    <div class="container">
      <NuxtLink to="/">
        <h2>{{ error.message }}</h2>
        <p>
          An error occurred! Click this text to go back home, and find your way
          back!
        </p>
      </NuxtLink>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '~/components/layout/MFooter'

export default {
  components: {
    MFooter,
  },
  props: {
    error: {
      type: Object,
      default() {
        return {
          message: 'Unknown error',
        }
      },
    },
  },
  layout: 'home',
  created() {
    // console.log(this.error)
  },
}
</script>

<style lang="scss" scoped>
.main {
  margin: var(--spacing-card-sm) auto;
  max-width: 800px;
}

.container {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
}
</style>
