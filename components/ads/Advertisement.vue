<template>
  <div v-if="displayed && !hidden" class="ad-wrapper">
    <div class="ad">
      <GptAd
        :key="format.adUnit"
        ref="ad_slot"
        :ad-unit="format.adUnit"
        :size="format.size"
        :is-responsive="true"
      />
    </div>
  </div>
  <div v-else-if="ethical_ads_on">
    <div v-if="ethical_ad_display && ethicalAdType === 'text'">
      <div
        :class="ethical_ad_style"
        data-ea-publisher="modrinth-com"
        :data-ea-type="ethicalAdType"
        data-ea-manual="true"
      ></div>
    </div>
    <div v-else-if="ethical_ad_display" class="ethical-wrapper">
      <div
        :class="ethical_ad_style"
        data-ea-publisher="modrinth-com"
        :data-ea-type="ethicalAdType"
        data-ea-manual="true"
      ></div>
    </div>
  </div>
  <div v-else></div>
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
    ethicalAdsBig: {
      type: Boolean,
      required: false,
      default: false,
    },
    ethicalAdsSmall: {
      type: Boolean,
      required: false,
      default: false,
    },
    ethicalAdType: {
      type: String,
      required: false,
      default: 'text',
    },
  },
  data() {
    return {
      isDark: false,
      format: null,
      displayed: false,
      onSmallScreen: false,
      windowResizeListenerDebounce: null,
      ethicalAdLoad: null,
    }
  },
  computed: {
    ethical_ads_on() {
      return (
        this.$store.app.$config.ads.ethicalAds === 'true' &&
        (this.ethicalAdsSmall || this.ethicalAdsBig)
      )
    },
    hidden() {
      return this.$store.app.$config.ads.ethicalAds === 'true'
    },
    ethical_ad_display() {
      return (
        (this.onSmallScreen && this.ethicalAdsSmall) ||
        (!this.onSmallScreen && this.ethicalAdsBig)
      )
    },
    ethical_ad_style() {
      return {
        dark: this.isDark,
        raised: true,
      }
    },
  },

  mounted() {
    // Register hook on resize
    window.addEventListener('resize', this.handleWindowResize)
    this.isDark = this.$colorMode.value !== 'light'
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
      this.refresh_ad()
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
            // Refresh ad
            this.refresh_ad()
          }
          return
        }
        if (this.onSmallScreen === false) {
          // Reload ad
          this.onSmallScreen = true
          this.refresh_ad()
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
    refresh_ad() {
      if (this.ethical_ads_on) {
        clearTimeout(this.ethicalAdLoad)
        this.ethicalAdLoad = setTimeout(() => {
          if (typeof window.ethicalads === 'undefined') {
            console.log('EthicalAds are not loaded yet, retrying...')
            this.refresh_ad()
          }
          ethicalads.load()
        }, 100)
      }
    },
  },
  head: {
    script: [
      {
        hid: 'ethical_ads_script',
        type: 'text/javascript',
        src: 'https://media.ethicalads.io/media/client/ethicalads.min.js',
        async: true,
        body: true,
        defer: true,
      }, // Insert in body
    ],
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
.ethical-wrapper {
  width: 100%;
  display: flex;
  flex-direction: row;
  margin-bottom: var(--spacing-card-md);
  justify-content: center;
}
</style>
