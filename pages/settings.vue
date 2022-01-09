<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="content">
        <h1 v-if="$auth.user">Settings for {{ $auth.user.username }}</h1>
        <h1 v-else>Settings</h1>
        <div class="card styled-tabs">
          <nuxt-link v-if="$auth.user" class="tab" to="/settings" exact
            ><span>Profile</span></nuxt-link
          >
          <nuxt-link v-if="$auth.user" class="tab" to="/settings/follows">
            <span>Followed projects</span>
          </nuxt-link>
          <nuxt-link v-if="$auth.user" class="tab" to="/settings/security">
            <span>Security</span>
          </nuxt-link>
          <nuxt-link class="tab" to="/settings/privacy">
            <span>Privacy</span>
          </nuxt-link>
          <button
            v-if="actionButton"
            class="iconified-button brand-button-colors right"
            @click="actionButtonCallback"
          >
            <CheckIcon />
            {{ actionButton }}
          </button>
        </div>
        <NuxtChild
          :action-button.sync="actionButton"
          :action-button-callback.sync="actionButtonCallback"
        />
      </div>
    </div>
  </div>
</template>

<script>
import CheckIcon from '~/assets/images/utils/check.svg?inline'

export default {
  name: 'Settings',
  components: {
    CheckIcon,
  },
  data() {
    return {
      actionButton: '',
    }
  },
  watch: {
    '$route.path': {
      handler() {
        this.actionButton = ''
        this.actionButtonCallback = () => {}
      },
    },
  },
  methods: {
    actionButtonCallback() {},
    changeTheme() {
      const shift = event.shiftKey
      switch (this.$colorMode.preference) {
        case 'dark':
          this.$colorMode.preference = shift ? 'light' : 'oled'
          break
        case 'oled':
          this.$colorMode.preference = shift ? 'dark' : 'light'
          break
        default:
          this.$colorMode.preference = shift ? 'oled' : 'dark'
      }
    },
  },
}
</script>

<style lang="scss" scoped>
.page-contents {
  display: flex;
  flex-direction: column;

  h1 {
    color: var(--color-text-dark);
    margin: 0 0 0.5rem 1.5rem;
  }
}

@media screen and (min-width: 1024px) {
  .page-contents {
    max-width: 60rem !important;
  }
}
</style>
