<!--suppress HtmlFormInputWithoutLabel -->
<template>
  <div class="popup card">
    <div class="consent-container">
      <div class="h1">Tweak your privacy settings</div>
      <div>
        Modrinth relies on different providers and in-house tools to allow us to
        provide custom-tailored experiences, and personalized advertising. You
        can at any moment change your privacy settings by going to the setting
        page, or at the footer of any page.
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
    <div class="actions">
      <button class="btn button" @click="toggleOff">Refuse All</button>
      <button class="btn button" @click="toggleOn">Accept All</button>
      <button class="btn brand-button" @click="confirm">
        Confirm my choices
      </button>
    </div>
  </div>
</template>

<script>
/* eslint-disable require-await */
import scopes from '@/privacy-toggles'
export default {
  name: 'Privacy',
  fetch() {
    this.$store.dispatch('consent/loadFromCookies', this.$cookies)

    if (this.$store.state.consent.is_consent_given) {
      Object.keys(scopes.settings).forEach((key) => {
        scopes.settings[key].value = false
      })

      // Load the allowed scopes from the store
      this.$store.state.consent.scopes_allowed.forEach((scope) => {
        if (this.scopes[scope] != null)
          this.$set(this.scopes[scope], 'value', true)
      })
    } else {
      Object.keys(scopes.settings).forEach((key) => {
        scopes.settings[key].value = scopes.settings[key].default
      })
    }
  },
  data: () => {
    const settings = scopes.settings
    return {
      scopes: settings,
    }
  },
  options: {
    auth: false,
  },
  methods: {
    toggleOff() {
      for (const elem in this.scopes) {
        this.$set(this.scopes[elem], 'value', false)
      }
    },
    toggleOn() {
      for (const elem in this.scopes) {
        this.$set(this.scopes[elem], 'value', true)
      }
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
  head: {
    title: 'Privacy Settings - Modrinth',
  },
}
</script>

<style scoped lang="scss">
.card {
  @extend %card;
  padding: var(--spacing-card-lg);
}
.popup {
  display: flex;
  flex-direction: column;
}
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
  .btn {
    margin-top: 0.5rem;
    margin-right: 0.5rem;
  }
}
.consent-container {
  .h1 {
    font-size: 2rem;
    font-weight: bold;
    margin-bottom: 0.6rem;
  }
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
</style>
