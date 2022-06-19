<template>
  <div class="rows card">
    <div class="header">
      <h2 class="title">Privacy settings</h2>
      <div class="controls">
        <button class="iconified-button" @click="toggleAll(false)">
          <DenyIcon />
          Deny all
        </button>
        <button class="iconified-button" @click="toggleAll(true)">
          <AllowIcon />
          Allow all
        </button>
        <button
          class="brand-button-colors iconified-button"
          title="Confirm privacy settings"
          @click="confirm()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </div>
    <div class="privacy-settings-container">
      <div>
        Modrinth relies on different providers and in-house tools to allow us to
        provide custom-tailored experiences and personalized advertising. You
        can change your privacy settings at any time by going to this settings
        page or via the footer of any page.
      </div>
      <br class="divider" />
      <div class="toggles">
        <div v-for="(scope, id) in scopes" :key="id" class="toggle">
          <div class="toggle-text">
            <div class="title">{{ scope.title }}</div>
            <div class="contents">
              {{ scope.description }}
            </div>
          </div>
          <div class="spacer"></div>
          <div class="toggle-action">
            <label :for="id"></label>
            <input
              :id="id"
              ref="toggles"
              v-model="scopes[id].value"
              type="checkbox"
              class="switch stylized-toggle"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import toggles from '@/privacy-toggles'
import DenyIcon from '~/assets/images/utils/clear.svg?inline'
import AllowIcon from '~/assets/images/utils/check-circle.svg?inline'
import SaveIcon from '~/assets/images/utils/save.svg?inline'

export default {
  auth: false,
  name: 'Privacy',
  components: {
    DenyIcon,
    AllowIcon,
    SaveIcon,
  },
  data: () => {
    const settings = toggles.settings
    return {
      scopes: settings,
    }
  },
  fetch() {
    this.$store.dispatch('consent/loadFromCookies', this.$cookies)
    if (this.$store.state.consent.is_consent_given) {
      Object.keys(toggles.settings).forEach((key) => {
        toggles.settings[key].value = false
      })
      // Load the allowed scopes from the store
      this.$store.state.consent.scopes_allowed.forEach((scope) => {
        if (this.scopes[scope] != null)
          this.$set(this.scopes[scope], 'value', true)
      })
    } else {
      Object.keys(toggles.settings).forEach((key) => {
        toggles.settings[key].value = toggles.settings[key].default
      })
    }
  },
  head: {
    title: 'Privacy Settings - Modrinth',
  },
  options: {
    auth: false,
  },
  methods: {
    toggleAll(value) {
      for (const elem in this.scopes) {
        this.scopes[elem].value = value
        this.$set(this.scopes[elem], 'value', value)
      }

      this.$forceUpdate()
      this.confirm()
    },
    confirm() {
      this.$store.commit('consent/set_consent', true)
      for (const elem in this.scopes) {
        if (this.scopes[elem].value === true) {
          this.$store.commit('consent/add_scope', elem)
        } else {
          this.$store.commit('consent/remove_scope', elem)
        }
      }
      this.$store.dispatch('consent/save', this.$cookies)
      this.$notify({
        group: 'main',
        title: 'Saved',
        text: 'Your preferences have been saved successfully.',
        type: 'success',
      })
    },
  },
}
</script>
<style lang="scss" scoped>
.spacer {
  margin-top: 1rem;
}

.actions {
  margin-top: 1rem;
  margin-right: -0.5rem;
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  flex-wrap: wrap;

  button {
    padding: 0.5rem 0.75rem;
    margin-top: 0.5rem;
    margin-right: 0.5rem;
  }
}

.privacy-settings-container {
  .divider {
    margin-top: 1rem;
  }

  .toggles {
    display: flex;
    flex-direction: column;
    width: 100%;

    .toggle {
      display: flex;
      flex-direction: row;
      margin-bottom: 1rem;

      .toggle-text {
        .title {
          color: var(--color-text-dark);
          font-weight: bold;
          margin-bottom: 0.5rem;
        }

        .contents {
          color: var(--color-text);
        }
      }

      .spacer {
        flex-grow: 1;
      }

      .toggle-action {
        margin-left: 1rem;
        display: flex;
        flex-direction: column;
        justify-content: center;
        margin-right: 1rem;
      }
    }
  }
}

.header {
  display: flex;
  align-items: center;
  padding-bottom: 1rem;
  grid-area: header;

  .title {
    flex-grow: 1;
    margin: 0;
  }

  .controls {
    display: flex;
    flex-direction: row;
    gap: 0.5rem;
  }
}
</style>
