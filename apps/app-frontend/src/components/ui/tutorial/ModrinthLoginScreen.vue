<script setup>
import { UserIcon, LockIcon, MailIcon, RightArrowIcon } from '@modrinth/assets'
import { ButtonStyled, Checkbox, ThirdPartyAuthButtons } from '@modrinth/ui'

import { login, login_2fa, create_account, login_pass } from '@/helpers/mr_auth.js'
import { handleError, useNotifications } from '@/store/state.js'
import { ref, computed } from 'vue'
import { handleSevereError } from '@/store/error.js'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const props = defineProps({
  callback: {
    type: Function,
    required: true,
  },
})

const modal = ref()
const turnstileToken = ref()
const widgetId = ref()

defineExpose({
  show: () => {
    modal.value.show()

    if (window.turnstile === null || !window.turnstile) {
      const script = document.createElement('script')
      script.src =
        'https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback'
      script.async = true
      script.defer = true
      document.head.appendChild(script)

      window.onloadTurnstileCallback = loadWidget
    } else {
      loadWidget()
    }
  },
})

function loadWidget() {
  widgetId.value = window.turnstile.render('#turnstile-container', {
    sitekey: '0x4AAAAAAAW3guHM6Eunbgwu',
    callback: (token) => (turnstileToken.value = token),
    expiredCallback: () => (turnstileToken.value = null),
  })
}

function removeWidget() {
  if (widgetId.value) {
    window.turnstile.remove(widgetId.value)
    widgetId.value = null
    turnstileToken.value = null
  }
}

const loggingIn = ref(true)
const twoFactorFlow = ref(null)
const twoFactorCode = ref('')

const email = ref('')
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const subscribe = ref(true)

async function signInOauth(provider) {
  const creds = await login(provider).catch(handleSevereError)

  if (creds && creds.type === 'two_factor_required') {
    twoFactorFlow.value = creds.flow
  } else if (creds && creds.session) {
    props.callback()
    modal.value.hide()
  }
}

async function signIn2fa() {
  const creds = await login_2fa(twoFactorCode.value, twoFactorFlow.value).catch(handleError)

  if (creds && creds.session) {
    props.callback()
    modal.value.hide()
  }
}

async function signIn() {
  const creds = await login_pass(username.value, password.value, turnstileToken.value).catch(
    handleError,
  )
  window.turnstile.reset(widgetId.value)

  if (creds && creds.type === 'two_factor_required') {
    twoFactorFlow.value = creds.flow
  } else if (creds && creds.session) {
    props.callback()
    modal.value.hide()
  }
}

async function createAccount() {
  if (password.value !== confirmPassword.value) {
    const notifs = useNotifications()
    notifs.addNotification({
      title: 'An error occurred',
      text: 'Passwords do not match!',
      type: 'error',
    })
    return
  }

  const creds = await create_account(
    username.value,
    email.value,
    password.value,
    turnstileToken.value,
    subscribe.value,
  ).catch(handleError)
  window.turnstile.reset(widgetId.value)

  if (creds && creds.session) {
    props.callback()
    modal.value.hide()
  }
}

const modalHeader = computed(() => {
  if (twoFactorFlow.value) {
    return 'Enter two-factor code'
  } else if (loggingIn.value) {
    return 'Sign in to Modrinth'
  } else {
    return 'Create a Modrinth account'
  }
})
</script>

<template>
  <ModalWrapper ref="modal" :on-hide="removeWidget" :header="modalHeader">
    <div class="flex flex-col w-[25rem]">
      <template v-if="twoFactorFlow">
        <p>Please enter a two-factor code to proceed.</p>
        <input v-model="twoFactorCode" maxlength="11" type="text" placeholder="Enter code..." />
      </template>
      <template v-else>
        <ThirdPartyAuthButtons type="action" :handler="(service) => signInOauth(service)" />
        <div class="divider">
          <hr />
          <p>or</p>
        </div>
        <div v-if="!loggingIn" class="iconified-input username">
          <MailIcon />
          <input v-model="email" type="text" placeholder="Email" />
        </div>
        <div class="iconified-input username">
          <UserIcon />
          <input
            v-model="username"
            type="text"
            :placeholder="loggingIn ? 'Email or username' : 'Username'"
          />
        </div>
        <div class="iconified-input" :class="{ username: !loggingIn }">
          <LockIcon />
          <input v-model="password" type="password" placeholder="Password" />
        </div>
        <div v-if="!loggingIn" class="iconified-input username">
          <LockIcon />
          <input v-model="confirmPassword" type="password" placeholder="Confirm password" />
        </div>
        <Checkbox
          v-if="!loggingIn"
          v-model="subscribe"
          class="subscribe-btn"
          label="Subscribe to updates about Modrinth"
        />
        <div class="turnstile mt-4">
          <div id="turnstile-container"></div>
          <div id="turnstile-container-2"></div>
        </div>
        <div class="mx-auto mt-4">
          <ButtonStyled color="brand" size="large">
            <button v-if="twoFactorCode" @click="signIn2fa">
              Sign in <RightArrowIcon />
            </button>
            <button
              v-else-if="loggingIn"
              :disabled="!turnstileToken"
              @click="signIn"
            >
              Sign in <RightArrowIcon />
            </button>
            <button v-else :disabled="!turnstileToken" @click="createAccount">
              Create account
            </button>
          </ButtonStyled>
        </div>
        <div class="flex items-center justify-center gap-6 mt-3 flex-wrap">
          <a v-if="!loggingIn" class="button-base hover:underline" @click="loggingIn = true">I already have an account</a>
          <template v-else>
            <a class="button-base hover:underline" @click="loggingIn = false"> Create account </a>
            <a class="button-base hover:underline" href="https://modrinth.com/auth/reset-password">
              Forgot password?
            </a>
          </template>
        </div>
      </template>
    </div>
  </ModalWrapper>
</template>

<style scoped lang="scss">
.button-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-gap: var(--gap-md);

  .btn {
    width: 100%;
    justify-content: center;
  }
}

.divider {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: var(--gap-md) 0;

  p {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background-color: var(--color-raised-bg);
    padding: 0 1rem;
    margin: 0;
  }

  hr {
    border: none;
    width: 100%;
    border-top: 2px solid var(--color-button-bg);
  }
}

.iconified-input {
  width: 100%;

  input {
    width: 100%;
    flex-basis: auto;
  }
}

.username {
  margin-bottom: var(--gap-sm);
}

.link-row {
  display: flex;
  justify-content: space-between;
  margin: var(--gap-md) 0;

  a {
    color: var(--color-blue);
    text-decoration: underline;

    &:hover {
      cursor: pointer;
    }
  }
}

.button-row {
  display: flex;
  justify-content: space-between;

  .btn {
    flex-basis: auto;
  }

  .transparent {
    padding: var(--gap-md) 0;
  }
}

:deep(.checkbox) {
  border: none;
}

.turnstile {
  display: flex;
  justify-content: center;
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 2px solid var(--color-button-bg);
  height: 65px;
  width: 100%;

  > div {
    position: relative;
    top: -2px;
    min-width: calc(100% + 4px);
  }
}
</style>
