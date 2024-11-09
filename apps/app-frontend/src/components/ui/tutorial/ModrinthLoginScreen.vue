<script setup>
import { UserIcon, LockIcon, MailIcon } from '@modrinth/assets'
import { Button, Card, Checkbox } from '@modrinth/ui'
import {
  DiscordIcon,
  GithubIcon,
  MicrosoftIcon,
  GoogleIcon,
  SteamIcon,
  GitLabIcon,
} from '@/assets/external'
import { login, login_2fa, create_account, login_pass } from '@/helpers/mr_auth.js'
import { handleError, useNotifications } from '@/store/state.js'
import { ref } from 'vue'
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
</script>

<template>
  <ModalWrapper ref="modal" :on-hide="removeWidget">
    <Card>
      <template v-if="twoFactorFlow">
        <h1>Enter two-factor code</h1>
        <p>Please enter a two-factor code to proceed.</p>
        <input v-model="twoFactorCode" maxlength="11" type="text" placeholder="Enter code..." />
      </template>
      <template v-else>
        <h1 v-if="loggingIn">Login to Modrinth</h1>
        <h1 v-else>Create an account</h1>
        <div class="button-grid">
          <Button class="discord" large @click="signInOauth('discord')">
            <DiscordIcon />
            Discord
          </Button>
          <Button class="github" large @click="signInOauth('github')">
            <GithubIcon />
            Github
          </Button>
          <Button class="white" large @click="signInOauth('microsoft')">
            <MicrosoftIcon />
            Microsoft
          </Button>
          <Button class="google" large @click="signInOauth('google')">
            <GoogleIcon />
            Google
          </Button>
          <Button class="white" large @click="signInOauth('steam')">
            <SteamIcon />
            Steam
          </Button>
          <Button class="gitlab" large @click="signInOauth('gitlab')">
            <GitLabIcon />
            GitLab
          </Button>
        </div>
        <div class="divider">
          <hr />
          <p>Or</p>
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
        <div class="turnstile">
          <div id="turnstile-container"></div>
          <div id="turnstile-container-2"></div>
        </div>
        <Checkbox
          v-if="!loggingIn"
          v-model="subscribe"
          class="subscribe-btn"
          label="Subscribe to updates about Modrinth"
        />
        <div class="link-row">
          <a v-if="loggingIn" class="button-base" @click="loggingIn = false"> Create account </a>
          <a v-else class="button-base" @click="loggingIn = true">Sign in</a>
          <a class="button-base" href="https://modrinth.com/auth/reset-password">
            Forgot password?
          </a>
        </div>
      </template>
      <div class="button-row">
        <Button class="transparent" large>Close</Button>
        <Button v-if="twoFactorCode" color="primary" large @click="signIn2fa"> Login </Button>
        <Button
          v-else-if="loggingIn"
          color="primary"
          large
          :disabled="!turnstileToken"
          @click="signIn"
        >
          Login
        </Button>
        <Button v-else color="primary" large :disabled="!turnstileToken" @click="createAccount">
          Create account
        </Button>
      </div>
    </Card>
  </ModalWrapper>
</template>

<style scoped lang="scss">
:deep(.modal-container) {
  .modal-body {
    width: auto;

    .content {
      background: none;
    }
  }
}

.card {
  width: 25rem;
}

.button-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-gap: var(--gap-md);

  .btn {
    width: 100%;
    justify-content: center;
  }

  .discord {
    background-color: #5865f2;
    color: var(--color-contrast);
  }

  .github {
    background-color: #8740f1;
    color: var(--color-contrast);
  }

  .white {
    background-color: var(--color-contrast);
    color: var(--color-accent-contrast);
  }

  .google {
    background-color: #4285f4;
    color: var(--color-contrast);
  }

  .gitlab {
    background-color: #fc6d26;
    color: var(--color-contrast);
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
  height: 66px;
  margin-top: var(--gap-md);

  iframe {
    margin: -1px;
    min-width: calc(100% + 2px);
  }
}
</style>
