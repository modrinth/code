<template>
  <div>
    <ModalConfirm
      ref="modal_confirm"
      title="Are you sure you want to delete your account?"
      description="This will **immediately delete all of your user data and follows**. This will not delete your projects. Deleting your account cannot be reversed.<br><br>If you need help with your account, get support on the [Modrinth Discord](https://discord.modrinth.com)."
      proceed-label="Delete this account"
      :confirmation-text="auth.user.username"
      :has-to-type="true"
      @proceed="deleteAccount"
    />
    <Modal ref="changeEmailModal" :header="`${auth.user.email ? 'Change' : 'Add'} email`">
      <div class="universal-modal">
        <p>Your account information is not displayed publicly.</p>
        <label for="email-input"><span class="label__title">Email address</span> </label>
        <input
          id="email-input"
          v-model="email"
          maxlength="2048"
          type="email"
          :placeholder="`Enter your email address...`"
          @keyup.enter="saveEmail()"
        />
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.changeEmailModal.hide()">
            <XIcon />
            Cancel
          </button>
          <button
            type="button"
            class="iconified-button brand-button"
            :disabled="!email"
            @click="saveEmail()"
          >
            <SaveIcon />
            Save email
          </button>
        </div>
      </div>
    </Modal>
    <Modal
      ref="managePasswordModal"
      :header="`${
        removePasswordMode ? 'Remove' : auth.user.has_password ? 'Change' : 'Add'
      } password`"
    >
      <div class="universal-modal">
        <ul
          v-if="newPassword !== confirmNewPassword && confirmNewPassword.length > 0"
          class="known-errors"
        >
          <li>Input passwords do not match!</li>
        </ul>
        <label v-if="removePasswordMode" for="old-password">
          <span class="label__title">Confirm password</span>
          <span class="label__description">Please enter your password to proceed.</span>
        </label>
        <label v-else-if="auth.user.has_password" for="old-password">
          <span class="label__title">Old password</span>
        </label>
        <input
          v-if="auth.user.has_password"
          id="old-password"
          v-model="oldPassword"
          maxlength="2048"
          type="password"
          autocomplete="current-password"
          :placeholder="`${removePasswordMode ? 'Confirm' : 'Old'} password`"
        />
        <template v-if="!removePasswordMode">
          <label for="new-password"><span class="label__title">New password</span></label>
          <input
            id="new-password"
            v-model="newPassword"
            maxlength="2048"
            type="password"
            autocomplete="new-password"
            placeholder="New password"
          />
          <label for="confirm-new-password"
            ><span class="label__title">Confirm new password</span></label
          >
          <input
            id="confirm-new-password"
            v-model="confirmNewPassword"
            maxlength="2048"
            type="password"
            autocomplete="new-password"
            placeholder="Confirm new password"
          />
        </template>
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.managePasswordModal.hide()">
            <XIcon />
            Cancel
          </button>
          <template v-if="removePasswordMode">
            <button
              type="button"
              class="iconified-button danger-button"
              :disabled="!oldPassword"
              @click="savePassword"
            >
              <TrashIcon />
              Remove password
            </button>
          </template>
          <template v-else>
            <button
              v-if="auth.user.has_password && auth.user.auth_providers.length > 0"
              type="button"
              class="iconified-button danger-button"
              @click="removePasswordMode = true"
            >
              <TrashIcon />
              Remove password
            </button>
            <button
              type="button"
              class="iconified-button brand-button"
              :disabled="
                newPassword.length == 0 ||
                (auth.user.has_password && oldPassword.length == 0) ||
                newPassword !== confirmNewPassword
              "
              @click="savePassword"
            >
              <SaveIcon />
              Save password
            </button>
          </template>
        </div>
      </div>
    </Modal>
    <Modal
      ref="manageTwoFactorModal"
      :header="`${
        auth.user.has_totp && twoFactorStep === 0 ? 'Remove' : 'Setup'
      } two-factor authentication`"
    >
      <div class="universal-modal">
        <template v-if="auth.user.has_totp && twoFactorStep === 0">
          <label for="two-factor-code">
            <span class="label__title">Enter two-factor code</span>
            <span class="label__description">Please enter a two-factor code to proceed.</span>
          </label>
          <input
            id="two-factor-code"
            v-model="twoFactorCode"
            maxlength="11"
            type="text"
            placeholder="Enter code..."
            @keyup.enter="removeTwoFactor()"
          />
          <p v-if="twoFactorIncorrect" class="known-errors">The code entered is incorrect!</p>
          <div class="input-group push-right">
            <button class="iconified-button" @click="$refs.manageTwoFactorModal.hide()">
              <XIcon />
              Cancel
            </button>
            <button class="iconified-button danger-button" @click="removeTwoFactor">
              <TrashIcon />
              Remove 2FA
            </button>
          </div>
        </template>
        <template v-else>
          <template v-if="twoFactorStep === 0">
            <p>
              Two-factor authentication keeps your account secure by requiring access to a second
              device in order to sign in.
              <br /><br />
              Scan the QR code with <a href="https://authy.com/">Authy</a>,
              <a href="https://www.microsoft.com/en-us/security/mobile-authenticator-app">
                Microsoft Authenticator</a
              >, or any other 2FA app to begin.
            </p>
            <qrcode-vue
              v-if="twoFactorSecret"
              :value="`otpauth://totp/${encodeURIComponent(
                auth.user.email
              )}?secret=${twoFactorSecret}&issuer=Modrinth`"
              :size="250"
              :margin="2"
              level="H"
            />
            <p>
              If the QR code does not scan, you can manually enter the secret:
              <strong>{{ twoFactorSecret }}</strong>
            </p>
          </template>
          <template v-if="twoFactorStep === 1">
            <label for="verify-code">
              <span class="label__title">Verify code</span>
              <span class="label__description"
                >Enter the one-time code from authenticator to verify access.
              </span>
            </label>
            <input
              id="verify-code"
              v-model="twoFactorCode"
              maxlength="6"
              type="text"
              autocomplete="one-time-code"
              placeholder="Enter code..."
              @keyup.enter="verifyTwoFactorCode()"
            />
            <p v-if="twoFactorIncorrect" class="known-errors">The code entered is incorrect!</p>
          </template>
          <template v-if="twoFactorStep === 2">
            <p>
              Download and save these back-up codes in a safe place. You can use these in-place of a
              2FA code if you ever lose access to your device! You should protect these codes like
              your password.
            </p>
            <p>Backup codes can only be used once.</p>
            <ul>
              <li v-for="code in backupCodes" :key="code">{{ code }}</li>
            </ul>
          </template>
          <div class="input-group push-right">
            <button v-if="twoFactorStep === 1" class="iconified-button" @click="twoFactorStep = 0">
              <LeftArrowIcon />
              Back
            </button>
            <button
              v-if="twoFactorStep !== 2"
              class="iconified-button"
              @click="$refs.manageTwoFactorModal.hide()"
            >
              <XIcon />
              Cancel
            </button>
            <button
              v-if="twoFactorStep <= 1"
              class="iconified-button brand-button"
              @click="twoFactorStep === 1 ? verifyTwoFactorCode() : (twoFactorStep = 1)"
            >
              <RightArrowIcon />
              Continue
            </button>
            <button
              v-if="twoFactorStep === 2"
              class="iconified-button brand-button"
              @click="$refs.manageTwoFactorModal.hide()"
            >
              <CheckIcon />
              Complete setup
            </button>
          </div>
        </template>
      </div>
    </Modal>
    <Modal ref="manageProvidersModal" header="Authentication providers">
      <div class="universal-modal">
        <div class="table">
          <div class="table-row table-head">
            <div class="table-cell table-text">Provider</div>
            <div class="table-cell table-text">Actions</div>
          </div>
          <div v-for="provider in authProviders" :key="provider.id" class="table-row">
            <div class="table-cell table-text">
              <span><component :is="provider.icon" /> {{ provider.display }}</span>
            </div>
            <div class="table-cell table-text manage">
              <button
                v-if="auth.user.auth_providers.includes(provider.id)"
                class="btn"
                @click="removeAuthProvider(provider.id)"
              >
                <TrashIcon /> Remove
              </button>
              <a
                v-else
                class="btn"
                :href="`${getAuthUrl(provider.id, '/settings/account')}&token=${auth.token}`"
              >
                <ExternalIcon /> Add
              </a>
            </div>
          </div>
        </div>
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.manageProvidersModal.hide()">
            <XIcon />
            Close
          </button>
        </div>
      </div>
    </Modal>
    <section class="universal-card">
      <h2>Account security</h2>

      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Email</span>
          <span class="label__description">Changes the email associated with your account.</span>
        </label>
        <div>
          <button class="iconified-button" @click="$refs.changeEmailModal.show()">
            <template v-if="auth.user.email">
              <EditIcon />
              Change email
            </template>
            <template v-else>
              <PlusIcon />
              Add email
            </template>
          </button>
        </div>
      </div>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Password</span>
          <span v-if="auth.user.has_password" class="label__description">
            Change <template v-if="auth.user.auth_providers.length > 0">or remove</template> the
            password used to login to your account.
          </span>
          <span v-else class="label__description">
            Set a permanent password to login to your account.
          </span>
        </label>
        <div>
          <button
            class="iconified-button"
            @click="
              () => {
                oldPassword = ''
                newPassword = ''
                confirmNewPassword = ''
                removePasswordMode = false
                $refs.managePasswordModal.show()
              }
            "
          >
            <KeyIcon />
            <template v-if="auth.user.has_password"> Change password </template>
            <template v-else> Add password </template>
          </button>
        </div>
      </div>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Two-factor authentication</span>
          <span class="label__description">
            Add an additional layer of security to your account during login.
          </span>
        </label>
        <div>
          <button class="iconified-button" @click="showTwoFactorModal">
            <template v-if="auth.user.has_totp"> <TrashIcon /> Remove 2FA </template>
            <template v-else> <PlusIcon /> Setup 2FA </template>
          </button>
        </div>
      </div>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Manage authentication providers</span>
          <span class="label__description">
            Add or remove sign-on methods from your account, including GitHub, GitLab, Microsoft,
            Discord, Steam, and Google.
          </span>
        </label>
        <div>
          <button class="iconified-button" @click="$refs.manageProvidersModal.show()">
            <SettingsIcon /> Manage providers
          </button>
        </div>
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

<script setup>
import {
  EditIcon,
  SaveIcon,
  TrashIcon,
  PlusIcon,
  SettingsIcon,
  XIcon,
  LeftArrowIcon,
  RightArrowIcon,
  CheckIcon,
  ExternalIcon,
} from 'omorphia'
import QrcodeVue from 'qrcode.vue'
import GitHubIcon from 'assets/icons/auth/sso-github.svg'
import MicrosoftIcon from 'assets/icons/auth/sso-microsoft.svg'
import GoogleIcon from 'assets/icons/auth/sso-google.svg'
import SteamIcon from 'assets/icons/auth/sso-steam.svg'
import DiscordIcon from 'assets/icons/auth/sso-discord.svg'
import KeyIcon from 'assets/icons/auth/key.svg'
import GitLabIcon from 'assets/icons/auth/sso-gitlab.svg'
import ModalConfirm from '~/components/ui/ModalConfirm.vue'
import Modal from '~/components/ui/Modal.vue'

useHead({
  title: 'Account settings - Modrinth',
})

definePageMeta({
  middleware: 'auth',
})

const data = useNuxtApp()
const auth = await useAuth()

const changeEmailModal = ref()
const email = ref(auth.value.user.email)
async function saveEmail() {
  if (!email.value) {
    return
  }

  startLoading()
  try {
    await useBaseFetch(`auth/email`, {
      method: 'PATCH',
      body: {
        email: email.value,
      },
    })
    changeEmailModal.value.hide()
    await useAuth(auth.value.token)
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

const managePasswordModal = ref()
const removePasswordMode = ref(false)
const oldPassword = ref('')
const newPassword = ref('')
const confirmNewPassword = ref('')
async function savePassword() {
  if (newPassword.value !== confirmNewPassword.value) {
    return
  }

  startLoading()
  try {
    await useBaseFetch(`auth/password`, {
      method: 'PATCH',
      body: {
        old_password: auth.value.user.has_password ? oldPassword.value : null,
        new_password: removePasswordMode.value ? null : newPassword.value,
      },
    })
    managePasswordModal.value.hide()
    await useAuth(auth.value.token)
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

const manageTwoFactorModal = ref()
const twoFactorSecret = ref(null)
const twoFactorFlow = ref(null)
const twoFactorStep = ref(0)
async function showTwoFactorModal() {
  twoFactorStep.value = 0
  twoFactorCode.value = null
  twoFactorIncorrect.value = false
  if (auth.value.user.has_totp) {
    manageTwoFactorModal.value.show()
    return
  }

  twoFactorSecret.value = null
  twoFactorFlow.value = null
  backupCodes.value = []
  manageTwoFactorModal.value.show()

  startLoading()
  try {
    const res = await useBaseFetch('auth/2fa/get_secret', {
      method: 'POST',
    })

    twoFactorSecret.value = res.secret
    twoFactorFlow.value = res.flow
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

const twoFactorIncorrect = ref(false)
const twoFactorCode = ref(null)
const backupCodes = ref([])
async function verifyTwoFactorCode() {
  startLoading()
  try {
    const res = await useBaseFetch('auth/2fa', {
      method: 'POST',
      body: {
        code: twoFactorCode.value ? twoFactorCode.value : '',
        flow: twoFactorFlow.value,
      },
    })

    backupCodes.value = res.backup_codes
    twoFactorStep.value = 2
    await useAuth(auth.value.token)
  } catch (err) {
    twoFactorIncorrect.value = true
  }
  stopLoading()
}

async function removeTwoFactor() {
  startLoading()
  try {
    await useBaseFetch('auth/2fa', {
      method: 'DELETE',
      body: {
        code: twoFactorCode.value ? twoFactorCode.value.toString() : '',
      },
    })
    manageTwoFactorModal.value.hide()
    await useAuth(auth.value.token)
  } catch (err) {
    twoFactorIncorrect.value = true
  }
  stopLoading()
}

const authProviders = [
  {
    id: 'github',
    display: 'GitHub',
    icon: GitHubIcon,
  },
  {
    id: 'gitlab',
    display: 'GitLab',
    icon: GitLabIcon,
  },
  {
    id: 'steam',
    display: 'Steam',
    icon: SteamIcon,
  },
  {
    id: 'discord',
    display: 'Discord',
    icon: DiscordIcon,
  },
  {
    id: 'microsoft',
    display: 'Microsoft',
    icon: MicrosoftIcon,
  },
  {
    id: 'google',
    display: 'Google',
    icon: GoogleIcon,
  },
]

async function deleteAccount() {
  startLoading()
  try {
    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: 'DELETE',
    })
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }

  useCookie('auth-token').value = null
  window.location.href = '/'

  stopLoading()
}
</script>
<style lang="scss" scoped>
canvas {
  margin: 0 auto;
  border-radius: var(--size-rounded-card);
}

.table-row {
  grid-template-columns: 1fr 10rem;

  span {
    display: flex;
    align-items: center;
    margin: auto 0;

    svg {
      width: 1.25rem;
      height: 1.25rem;
      margin-right: 0.35rem;
    }
  }
}
</style>
