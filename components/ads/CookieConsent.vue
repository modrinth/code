<template>
  <div>
    <div
      ref="container"
      class="container"
      :class="{ 'mobile-menu-open': mobileMenuOpen }"
      :style="{
        visibility: shown ? 'visible' : 'hidden',
      }"
    >
      <div class="card banner">
        <span>
          Modrinth uses cookies for various purposes. We encourage you to review
          your privacy settings by clicking on the button below:
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
  props: {
    mobileMenuOpen: {
      type: Boolean,
      default: true,
    },
  },
  data() {
    return {
      shown: false,
    }
  },
  fetch() {
    this.checkVisibility()
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
        this.$route.path !== '/settings/privacy'
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
      this.$router.push('/settings/privacy')
    },
  },
}
</script>

<style scoped lang="scss">
.container {
  width: 100%;
  text-align: center;

  z-index: 2;
  position: fixed;
  right: 0;
  bottom: 0;

  .banner {
    font-size: 1.05rem;
    border-radius: 0;
    margin-bottom: 0;
    box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
    padding: 1rem 1rem calc(var(--size-mobile-navbar-height) + 1rem);
    transition: padding-bottom 0.25s ease-in-out;
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

  .banner {
    margin-bottom: 0;
  }

  &.mobile-menu-open {
    .banner {
      padding-bottom: calc(var(--size-mobile-navbar-height-expanded) + 1rem);
    }
  }

  @media screen and (min-width: 750px) {
    .banner {
      padding-bottom: 1rem;
    }

    &.mobile-menu-open {
      bottom: 0;
    }
  }

  @media screen and (min-width: 1024px) {
    width: unset;
    text-align: unset;

    .banner {
      border-radius: var(--size-rounded-card);
      width: 18vw;
      min-width: 16rem;
      border-left: solid 5px var(--color-brand);
      margin: 0 2rem 2rem 0;
    }

    .actions {
      justify-content: unset;
    }
  }
}
</style>
