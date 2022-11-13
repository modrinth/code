<template>
  <nav class="navigation" :class="{ 'use-animation': useAnimation }">
    <NuxtLink
      v-for="(link, index) in filteredLinks"
      v-show="link.shown === undefined ? true : link.shown"
      :key="index"
      ref="linkElements"
      :to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
      class="nav-link button-animation"
      :class="{ 'is-active': index === activeIndex }"
    >
      <span>{{ link.label }}</span>
    </NuxtLink>

    <div
      class="nav-indicator"
      :style="`visibility: ${
        useAnimation && activeIndex !== -1 ? 'visible' : 'hidden'
      }; left: ${indicator.left}px; right: ${indicator.right}px;
          top: ${indicator.top}px; transition: left 350ms ${
        indicator.direction === 'left'
          ? 'cubic-bezier(1,0,.3,1) -140ms'
          : 'cubic-bezier(.75,-0.01,.24,.99) -40ms'
      },right 350ms ${
        indicator.direction === 'right'
          ? 'cubic-bezier(1,0,.3,1) -140ms'
          : 'cubic-bezier(.75,-0.01,.24,.99) -40ms'
      }, top 100ms ease-in-out`"
    />
  </nav>
</template>

<script>
export default {
  name: 'NavRow',
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
      useAnimation: false,
      oldIndex: -1,
      activeIndex: -1,
      indicator: {
        left: 0,
        right: 0,
        top: 22,
        direction: 'right',
      },
    }
  },
  computed: {
    filteredLinks() {
      return this.links.filter((x) => (x.shown === undefined ? true : x.shown))
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
    this.pickLink()
  },
  methods: {
    pickLink() {
      if (this.oldIndex === -1) {
        this.useAnimation = false

        setTimeout(() => {
          this.useAnimation = true
        }, 300)
      }

      this.activeIndex = this.query
        ? this.filteredLinks.findIndex(
            (x) =>
              (x.href === '' ? undefined : x.href) ===
              this.$route.query[this.query]
          )
        : this.filteredLinks.findIndex(
            (x) => x.href === decodeURIComponent(this.$route.path)
          )

      if (this.activeIndex !== -1) {
        this.startAnimation()
      } else {
        this.oldIndex = -1
      }
    },
    startAnimation() {
      if (this.$refs.linkElements[this.activeIndex]) {
        this.indicator.direction =
          this.activeIndex < this.oldIndex ? 'left' : 'right'

        this.indicator.left =
          this.$refs.linkElements[this.activeIndex].$el.offsetLeft
        this.indicator.right =
          this.$refs.linkElements[this.activeIndex].$el.parentElement
            .offsetWidth -
          this.$refs.linkElements[this.activeIndex].$el.offsetLeft -
          this.$refs.linkElements[this.activeIndex].$el.offsetWidth
        this.indicator.top =
          this.$refs.linkElements[this.activeIndex].$el.offsetTop +
          this.$refs.linkElements[this.activeIndex].$el.offsetHeight +
          1
      }

      this.oldIndex = this.activeIndex
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
    color: var(--color-text);
    position: relative;

    &::after {
      content: '';
      display: block;
      position: absolute;
      bottom: -5px;
      width: 100%;
      border-radius: var(--size-rounded-max);
      height: 0.25rem;
      transition: opacity 0.1s ease-in-out;
      background-color: var(--color-brand);
      opacity: 0;
    }

    &:hover {
      color: var(--color-text);

      &::after {
        opacity: 0.4;
      }
    }

    &:active::after {
      opacity: 0.2;
    }

    &.is-active {
      color: var(--color-text);

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
    border-radius: var(--size-rounded-max);
    background-color: var(--color-brand);
    transition-property: left, right, top;
    transition-duration: 350ms;
    visibility: hidden;

    @media (prefers-reduced-motion) {
      transition: none !important;
    }
  }
}
</style>
