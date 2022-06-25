<template>
  <div>
    <ConfirmPopup
      ref="delete_popup"
      title="Are you sure you want to delete your account?"
      description="If you proceed, your user and all attached data will be removed from our
        servers. This cannot be reversed, so be careful!"
      proceed-label="Delete account"
      :confirmation-text="$auth.user.username"
      :has-to-type="true"
      @proceed="deleteAccount"
    />

    <section class="card">
      <div class="header">
        <h2 class="title">Security settings</h2>
      </div>
      <label>
        <span>
          <h3>Authorization token</h3>
          <span>
            Your authorization token can be used with the Modrinth API, the
            Minotaur Gradle plugin, and other applications that interact with
            Modrinth's API. Be sure to keep this secret!
          </span>
        </span>
        <input
          type="button"
          class="iconified-button"
          value="Copy to clipboard"
          @click="copyToken"
        />
      </label>
      <label>
        <span>
          <h3>Revoke your token</h3>
          <span
            >This will log you out of Modrinth, and you will have to log in
            again to access Modrinth with a new token.</span
          >
        </span>
        <input
          type="button"
          class="iconified-button"
          value="Revoke token"
          @click="$router.replace('/settings/revoke-token')"
        />
      </label>
      <label>
        <span>
          <h3>Delete your account</h3>
          <span
            >Clicking on this WILL delete your account. Do not click on this
            unless you want your account deleted. If you delete your account,
            all attached data, including projects, will be removed from our
            servers. This cannot be reversed, so be careful!</span
          >
        </span>
        <input
          value="Delete account"
          type="button"
          class="iconified-button"
          @click="$refs.delete_popup.show()"
        />
      </label>
    </section>
  </div>
</template>

<script>
import ConfirmPopup from '~/components/ui/ConfirmPopup'

export default {
  components: { ConfirmPopup },
  head: {
    title: 'Security - Modrinth',
  },
  methods: {
    async copyToken() {
      await navigator.clipboard.writeText(this.$auth.token)
      this.$notify({
        group: 'main',
        title: 'Copied to clipboard.',
        text: 'Copied your Modrinth token to the clipboard.',
        type: 'success',
      })
    },
    async deleteAccount() {
      this.$nuxt.$loading.start()
      try {
        await this.$axios.delete(
          `user/${this.$auth.user.id}`,
          this.$defaultHeaders()
        )
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }
      this.$nuxt.$loading.finish()
    },
  },
}
</script>
<style lang="scss" scoped>
.card span {
  margin-bottom: 1rem;
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
