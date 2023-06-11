<template>
  <div>
    <ModalConfirm
      ref="modal_confirm"
      title="Are you sure you want to delete your account?"
      description="This will **immediately delete all of your user data and follows**. This will not delete your projects. Deleting your account cannot be reversed.<br><br>If you need help with your account, get support on the [Modrinth Discord](https://discord.gg/EUHuJHt)."
      proceed-label="Delete this account"
      :confirmation-text="auth.user.username"
      :has-to-type="true"
      @proceed="deleteAccount"
    />

    <Modal ref="modal_revoke_token" header="Revoke your Modrinth token">
      <div class="modal-revoke-token markdown-body">
        <p>
          Revoking your Modrinth token can have unintended consequences. Please be aware that the
          following could break:
        </p>
        <ul>
          <li>Any application that uses your token to access the API.</li>
          <li>Gradle - if Minotaur is given a incorrect token, your Gradle builds could fail.</li>
          <li>
            GitHub - if you use a GitHub action that uses the Modrinth API, it will cause errors.
          </li>
        </ul>
        <p>If you are willing to continue, complete the following steps:</p>
        <ol>
          <li>
            <a
              href="https://github.com/settings/connections/applications/3acffb2e808d16d4b226"
              target="_blank"
              rel="noopener"
            >
              Head to the Modrinth Application page on GitHub.
            </a>
            Make sure to be logged into the GitHub account you used for Modrinth!
          </li>
          <li>Press the big red "Revoke Access" button next to the "Permissions" header.</li>
        </ol>
        <p>Once you have completed those steps, press the continue button below.</p>
        <p>
          <strong>
            This will log you out of Modrinth, however, when you log back in, your token will be
            regenerated.
          </strong>
        </p>
        <div class="button-group">
          <button class="iconified-button" @click="$refs.modal_revoke_token.hide()">
            <CrossIcon />
            Cancel
          </button>
          <button class="iconified-button brand-button" @click="logout">
            <RightArrowIcon />
            Log out
          </button>
        </div>
      </div>
    </Modal>

    <section class="universal-card">
      <h2>User profile</h2>
      <p>Visit your user profile to edit your profile information.</p>
      <NuxtLink class="iconified-button" :to="`/user/${auth.user.username}`">
        <UserIcon /> Visit your profile
      </NuxtLink>
    </section>

    <section class="universal-card">
      <h2>Account information</h2>
      <p>Your account information is not displayed publicly.</p>
      <ul class="known-errors">
        <li v-if="hasMonetizationEnabled() && !email">
          You must have an email address set since you are enrolled in the Creator Monetization
          Program.
        </li>
      </ul>
      <label for="email-input"><span class="label__title">Email address</span> </label>
      <input
        id="email-input"
        v-model="email"
        maxlength="2048"
        type="email"
        :placeholder="`Enter your email address...`"
      />
      <div class="button-group">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="hasMonetizationEnabled() && !email"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>

    <section class="universal-card">
      <h2>Authorization token</h2>
      <p>
        Your authorization token can be used with the Modrinth API, the Minotaur Gradle plugin, and
        other applications that interact with Modrinth's API. Be sure to keep this secret!
      </p>
      <div class="input-group">
        <button type="button" class="iconified-button" value="Copy to clipboard" @click="copyToken">
          <template v-if="copied">
            <CheckIcon />
            Copied token to clipboard
          </template>
          <template v-else> <CopyIcon />Copy token to clipboard </template>
        </button>
        <button type="button" class="iconified-button" @click="$refs.modal_revoke_token.show()">
          <SlashIcon />
          Revoke token
        </button>
      </div>
    </section>

    <section id="delete-account" class="universal-card">
      <h2>Delete account</h2>
      <p>
        Once you delete your account, there is no going back. Deleting your account will remove all
        attached data, excluding projects, from our servers.
      </p>
      <button
        type="button"
        class="iconified-button danger-button"
        @click="$refs.modal_confirm.show()"
      >
        <TrashIcon />
        Delete account
      </button>
    </section>
  </div>
</template>

<script>
import ModalConfirm from '~/components/ui/ModalConfirm.vue'
import Modal from '~/components/ui/Modal.vue'

import CrossIcon from '~/assets/images/utils/x.svg'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg'
import CheckIcon from '~/assets/images/utils/check.svg'
import UserIcon from '~/assets/images/utils/user.svg'
import SaveIcon from '~/assets/images/utils/save.svg'
import CopyIcon from '~/assets/images/utils/clipboard-copy.svg'
import TrashIcon from '~/assets/images/utils/trash.svg'
import SlashIcon from '~/assets/images/utils/slash.svg'

export default defineNuxtComponent({
  components: {
    Modal,
    ModalConfirm,
    CrossIcon,
    RightArrowIcon,
    CheckIcon,
    SaveIcon,
    UserIcon,
    CopyIcon,
    TrashIcon,
    SlashIcon,
  },
  async setup() {
    definePageMeta({
      middleware: 'auth',
    })

    const auth = await useAuth()

    return { auth }
  },
  data() {
    return {
      copied: false,
      email: this.auth.user.email,
      showKnownErrors: false,
    }
  },
  head: {
    title: 'Account settings - Modrinth',
  },
  methods: {
    async copyToken() {
      this.copied = true
      await navigator.clipboard.writeText(this.auth.token)
    },
    async deleteAccount() {
      startLoading()
      try {
        await useBaseFetch(`user/${this.auth.user.id}`, {
          method: 'DELETE',
          ...this.$defaultHeaders(),
        })
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data.description,
          type: 'error',
        })
      }

      useCookie('auth-token').value = null
      alert('Please note that logging back in with GitHub will create a new account.')
      window.location.href = '/'

      stopLoading()
    },
    logout() {
      this.$refs.modal_revoke_token.hide()
      useCookie('auth-token').value = null

      window.location.href = getAuthUrl()
    },
    hasMonetizationEnabled() {
      return (
        this.auth.user.payout_data.payout_wallet &&
        this.auth.user.payout_data.payout_wallet_type &&
        this.auth.user.payout_data.payout_address
      )
    },
    async saveChanges() {
      if (this.hasMonetizationEnabled() && !this.email) {
        this.showKnownErrors = true
        return
      }
      startLoading()
      try {
        const data = {
          email: this.email ? this.email : null,
        }

        await useBaseFetch(`user/${this.auth.user.id}`, {
          method: 'PATCH',
          body: data,
          ...this.$defaultHeaders(),
        })
        await useAuth(this.auth.token)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data.description,
          type: 'error',
        })
      }
      stopLoading()
    },
  },
})
</script>
<style lang="scss" scoped>
.modal-revoke-token {
  padding: var(--spacing-card-bg);

  .button-group {
    width: fit-content;
    margin-left: auto;
  }
}
</style>
