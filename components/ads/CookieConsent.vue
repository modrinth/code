<template>
  <div>
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
import scopes from '~/privacy-toggles'
export default {
  name: 'CookieConsent',
  fetch() {
    this.checkVisibility()
  },
  data() {
    return {
      shown: false,
    }
  },
  watch: {
    $route() {
      this.checkVisibility()
    },
  },
  methods: {
    checkVisibility() {
      this.$store.dispatch('consent/loadFromCookies', this.$cookies)

      this.shown =
        !this.$store.state.consent.is_consent_given &&
        this.$route.path !== '/dashboard/privacy'
    },
    hide() {
      this.$store.commit('consent/set_consent', true)
      // Accept all scopes
      for (const elem in scopes.settings) {
        this.$store.commit('consent/add_scope', elem)
      }
      this.$store.dispatch('consent/save', this.$cookies)

      this.shown = false
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
  width: 100%;
  text-align: center;

  z-index: 20;
  position: fixed;
  bottom: 0;
  right: 0;
  .banner {
    @extend %card;
    padding: 1rem;
    font-size: 1.05rem;
    border-radius: 0;
  }
  .actions {
    display: flex;
    flex-direction: row;
    margin-top: 1rem;
    justify-content: center;

    .btn {
      margin-right: 0.5rem;
    }
  }

  @media screen and (min-width: 1024px) {
    width: unset;
    text-align: unset;

    .banner {
      max-width: 18vw;
      border-left: solid 5px var(--color-brand);
      margin: 0 2rem 2rem 0;
    }

    .actions {
      justify-content: unset;
    }
  }
}
</style>
