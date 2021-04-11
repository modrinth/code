<template>
  <div>
    <ReviewPopup ref="popup" />
    <div
      ref="container"
      class="container"
      :style="{ visibility: shown ? 'visible' : 'hidden' }"
    >
      <div class="banner">
        <span>
          Modrinth uses cookies for various purposes, including advertising.<br />
          We encourage you to review your privacy settings by clicking on the
          button below:
        </span>
        <div class="actions">
          <button class="btn button" @click="review">Review</button>
          <button class="btn brand-button" @click="hide">Accept all</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'CookieConsent',
  fetch() {
    // Get informations in the store
    this.$store.dispatch('consent/loadFromCookies', this.$cookies)
    if (
      !this.$store.state.consent.is_consent_given &&
      this.$route.path !== '/dashboard/privacy'
    ) {
      this.shown = true
    }
  },
  data() {
    return {
      shown: false,
    }
  },
  methods: {
    hide() {
      this.$store.commit('consent/set_consent', true)
      this.$store.commit('consent/add_scope', true)
      this.$store.commit('consent/remove_scope', true)
      this.$store.dispatch('consent/save', this.$cookies)
    },
    review() {
      this.shown = false
      this.$router.push('/dashboard/privacy')
    },
  },
}
</script>

<style scoped lang="scss">
.container {
  z-index: 20;
  position: fixed;
  bottom: 0;
  right: 0;
  .banner {
    @extend %card;
    margin: 0 2rem 2rem 0;
    padding: 1rem;
    max-width: 18vw;
    border-left: solid 5px var(--color-brand);
    font-size: 1.05rem;
  }
  .actions {
    display: flex;
    flex-direction: row;
    margin-top: 1rem;
    .btn {
      margin-right: 0.5rem;
    }
  }
}
</style>
