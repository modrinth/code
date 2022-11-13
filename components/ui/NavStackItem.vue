<template>
  <NuxtLink class="nav-link button-base" :to="link">
    <div class="nav-content">
      <slot />
      <span>{{ label }}</span>
      <span v-if="beta" class="beta-badge">BETA</span>
    </div>
  </NuxtLink>
</template>

<script>
export default {
  name: 'NavStackItem',
  props: {
    link: {
      required: true,
      type: String,
    },
    label: {
      required: true,
      type: String,
    },
    beta: {
      default: false,
      type: Boolean,
    },
  },
}
</script>

<style lang="scss" scoped>
.nav-link {
  font-weight: var(--font-weight-bold);
  color: var(--color-text);
  position: relative;
  display: flex;
  flex-direction: row;
  gap: 0.25rem;
  box-shadow: none;

  .nav-content {
    box-sizing: border-box;
    padding: 0.5rem 0.75rem;
    border-radius: var(--size-rounded-sm);
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-grow: 1;
    background-color: var(--color-raised-bg);
  }

  &.nuxt-link-exact-active {
    .nav-content {
      color: var(--color-button-text-active);
      background-color: var(--color-button-bg);
      box-shadow: none;
    }
  }

  .beta-badge {
    margin: 0;
  }
}
</style>
