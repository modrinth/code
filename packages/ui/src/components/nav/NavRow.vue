<template>
  <nav class="navigation">
    <router-link
      v-for="(link, index) in filteredLinks"
      v-show="link.shown === undefined ? true : link.shown"
      :key="index"
      ref="linkElements"
      :to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
      class="nav-link button-animation"
    >
      <span>{{ link.label }}</span>
    </router-link>
    <div
      class="nav-indicator"
      :style="{
        left: positionToMoveX,
        top: positionToMoveY,
        width: sliderWidth,
        opacity: activeIndex === -1 ? 0 : 1,
      }"
      aria-hidden="true"
    />
  </nav>
</template>

<script>
export default {
  props: {
    links: {
      default: () => [],
      type: Array,
    },
    query: {
      default: null,
      type: String,
    },
  },
  data() {
    return {
      sliderPositionX: 0,
      sliderPositionY: 18,
      selectedElementWidth: 0,
      activeIndex: -1,
      oldIndex: -1,
    }
  },
  computed: {
    filteredLinks() {
      return this.links.filter((x) => (x.shown === undefined ? true : x.shown))
    },
    positionToMoveX() {
      return `${this.sliderPositionX}px`
    },
    positionToMoveY() {
      return `${this.sliderPositionY}px`
    },
    sliderWidth() {
      return `${this.selectedElementWidth}px`
    },
  },
  watch: {
    '$route.path': {
      handler() {
        this.pickLink()
      },
    },
    '$route.query': {
      handler() {
        if (this.query) this.pickLink()
      },
    },
  },
  mounted() {
    window.addEventListener('resize', this.pickLink)
    this.pickLink()
  },
  unmounted() {
    window.removeEventListener('resize', this.pickLink)
  },
  methods: {
    pickLink() {
      this.activeIndex = this.query
        ? this.filteredLinks.findIndex(
            (x) => (x.href === '' ? undefined : x.href) === this.$route.path[this.query],
          )
        : this.filteredLinks.findIndex((x) => x.href === decodeURIComponent(this.$route.path))

      if (this.activeIndex !== -1) {
        this.startAnimation()
      } else {
        this.oldIndex = -1
        this.sliderPositionX = 0
        this.selectedElementWidth = 0
      }
    },
    startAnimation() {
      const el = this.$refs.linkElements[this.activeIndex].$el

      this.sliderPositionX = el.offsetLeft
      this.sliderPositionY = el.offsetTop + el.offsetHeight
      this.selectedElementWidth = el.offsetWidth
    },
  },
}
</script>

<style lang="scss" scoped>
.navigation {
  display: flex;
  flex-direction: row;
  align-items: center;
  grid-gap: 1rem;
  flex-wrap: wrap;
  position: relative;

  .nav-link {
    text-transform: capitalize;
    font-weight: var(--font-weight-bold);
    color: var(--color-base);
    position: relative;

    &:hover {
      color: var(--color-base);

      &::after {
        opacity: 0.4;
      }
    }

    &:active::after {
      opacity: 0.2;
    }

    &.router-link-exact-active {
      color: var(--color-base);

      &::after {
        opacity: 1;
      }
    }
  }

  &.use-animation {
    .nav-link {
      &.is-active::after {
        opacity: 0;
      }
    }
  }

  .nav-indicator {
    position: absolute;
    height: 0.25rem;
    bottom: -5px;
    left: 0;
    width: 3rem;
    transition: all ease-in-out 0.2s;
    border-radius: var(--radius-max);
    background-color: var(--color-brand);

    @media (prefers-reduced-motion) {
      transition: none !important;
    }
  }
}
</style>
