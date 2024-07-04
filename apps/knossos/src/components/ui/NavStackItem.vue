<template>
  <NuxtLink v-if="link !== null" class="nav-link button-base" :to="link">
    <div class="nav-content">
      <slot />
      <span>{{ label }}</span>
      <span v-if="beta" class="beta-badge">BETA</span>
      <span v-if="chevron" class="chevron"><ChevronRightIcon /></span>
    </div>
  </NuxtLink>
  <button
    v-else-if="action"
    class="nav-link button-base"
    :class="{ 'danger-button': danger }"
    @click="action"
  >
    <span class="nav-content">
      <slot />
      <span>{{ label }}</span>
      <span v-if="beta" class="beta-badge">BETA</span>
    </span>
  </button>
  <span v-else>i forgor 💀</span>
</template>

<script>
import ChevronRightIcon from '~/assets/images/utils/chevron-right.svg?component'

export default {
  components: {
    ChevronRightIcon,
  },
  props: {
    link: {
      default: null,
      type: String,
    },
    action: {
      default: null,
      type: Function,
    },
    label: {
      required: true,
      type: String,
    },
    beta: {
      default: false,
      type: Boolean,
    },
    chevron: {
      default: false,
      type: Boolean,
    },
    danger: {
      default: false,
      type: Boolean,
    },
  },
}
</script>

<style lang="scss" scoped>
.nav-link {
  font-weight: var(--font-weight-bold);
  background-color: transparent;
  color: var(--text-color);
  position: relative;
  display: flex;
  flex-direction: row;
  gap: 0.25rem;
  box-shadow: none;
  padding: 0;
  width: 100%;
  outline: none;

  :where(.nav-link) {
    --text-color: var(--color-text);
    --background-color: var(--color-raised-bg);
  }

  .nav-content {
    box-sizing: border-box;
    padding: 0.5rem 0.75rem;
    border-radius: var(--size-rounded-sm);
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-grow: 1;
    background-color: var(--background-color);
  }

  &:focus-visible {
    .nav-content {
      border-radius: 0.25rem;
    }
  }

  &.router-link-exact-active {
    outline: 2px solid transparent;
    border-radius: 0.25rem;

    .nav-content {
      color: var(--color-button-text-active);
      background-color: var(--color-button-bg);
      box-shadow: none;
    }
  }

  .beta-badge {
    margin: 0;
  }

  .chevron {
    margin-left: auto;
  }
}
</style>
