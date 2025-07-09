<script setup>
defineProps({
  collapsible: {
    type: Boolean,
    default: false,
  },
  rightSidebar: {
    type: Boolean,
    default: false,
  },
})
</script>

<template>
  <div
    class="omorphia__page"
    :class="{
      'right-sidebar': rightSidebar,
      'has-sidebar': !!$slots.sidebar,
      'has-header': !!$slots.header,
      'has-footer': !!$slots.footer,
    }"
  >
    <div v-if="!!$slots.header" class="header">
      <slot name="header" />
    </div>
    <div v-if="!!$slots.sidebar" class="sidebar">
      <slot name="sidebar" />
    </div>
    <div class="content">
      <slot />
    </div>
    <div v-if="!!$slots.footer" class="footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.omorphia__page {
  display: flex;
  flex-direction: column;
  padding: 0 0.75rem;

  .header {
    grid-area: header;
  }

  .sidebar {
    grid-area: sidebar;
  }

  .footer {
    grid-area: footer;
  }

  .content {
    grid-area: content;
  }
}

@media (min-width: 1024px) {
  .omorphia__page {
    margin: 0 auto;
    max-width: 80rem;
    column-gap: 0.75rem;

    &.has-sidebar {
      display: grid;
      grid-template:
        'sidebar content' auto
        'footer content' auto
        'dummy content' 1fr
        / 20rem 1fr;

      &.has-header {
        grid-template:
          'header header' auto
          'sidebar content' auto
          'footer content' auto
          'dummy content' 1fr
          / 20rem 1fr;
      }

      &.right-sidebar {
        grid-template:
          'content sidebar' auto
          'content footer' auto
          'content dummy' 1fr
          / 1fr 20rem;

        &.has-header {
          grid-template:
            'header header' auto
            'content sidebar' auto
            'content footer' auto
            'content dummy' 1fr
            / 1fr 20rem;
        }
      }

      .content {
        max-width: calc(60rem - 0.75rem);
      }
    }
  }

  .sidebar {
    min-width: 20rem;
    width: 20rem;
  }
}

@media (min-width: 80rem) {
  .omorphia__page.has-sidebar {
    .content {
      width: calc(60rem - 0.75rem);
    }
  }
}
</style>
