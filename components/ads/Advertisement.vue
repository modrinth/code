<template>
  <div class="ad-wrapper">
    <div v-if="displayed" class="ad">
      <GptAd
        :key="format.adUnit"
        ref="ad_slot"
        :ad-unit="format.adUnit"
        :size="format.size"
        :is-responsive="true"
      />
    </div>
  </div>
</template>

<script>
const sizes = {
  banner: {
    adUnit: 'banner',
    size: '728x90,468x60',
  },
  square: {
    adUnit: 'square',
    size: '250x250,200x200',
  },
}

/* eslint-disable no-undef */
export default {
  name: 'Advertisement',
  props: {
    type: {
      type: String,
      required: true,
    },
    smallScreen: {
      type: String,
      required: true,
    },
  },
  data() {
    return {
      format: null,
      displayed: false,
      onSmallScreen: false,
      windowResizeListenerDebounce: null,
    }
  },

  mounted() {
    // Register hook on resize
    window.addEventListener('resize', this.handleWindowResize)

    // Find ad
    if (!(this.type in sizes)) {
      console.error('Ad type not recognized.')
      return
    }
    // Set the informations
    this.format = sizes[this.type]
    this.displayed = true
    if (process.browser) {
      this.handleWindowResize()
    }
  },
  methods: {
    handleWindowResize() {
      clearTimeout(this.windowResizeListenerDebounce)
      this.windowResizeListenerDebounce = setTimeout(() => {
        if (window.innerWidth > 1024) {
          if (this.onSmallScreen) {
            // Return everything to normal size
            this.onSmallScreen = false
            this.format = sizes[this.type]
            this.displayed = true
          }
          return
        }
        this.onSmallScreen = true
        if (this.smallScreen === 'destroy') {
          this.displayed = false
        } else if (this.smallScreen in sizes) {
          console.log('Changing ad size to ', this.smallScreen)
          this.format = sizes[this.smallScreen]
        }
      }, 300)
    },
  },
}
</script>

<style lang="scss" scoped>
.ad-wrapper {
  width: 100%;
  @extend %card;
  display: flex;
  flex-direction: row;
  margin-bottom: var(--spacing-card-md);
  justify-content: center;
}
</style>
