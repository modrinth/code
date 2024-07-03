<template>
  <div>
    <template v-if="flow">
      <label for="two-factor-code">
        <span class="label__title">{{ formatMessage(messages.twoFactorCodeLabel) }}</span>
        <span class="label__description">
          {{ formatMessage(messages.twoFactorCodeLabelDescription) }}
        </span>
      </label>
      <input
        id="two-factor-code"
        v-model="twoFactorCode"
        maxlength="11"
        type="text"
        :placeholder="formatMessage(messages.twoFactorCodeInputPlaceholder)"
        autocomplete="one-time-code"
        autofocus
        @keyup.enter="begin2FASignIn"
      />

      <button class="btn btn-primary continue-btn" @click="begin2FASignIn">
        {{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
      </button>
    </template>
    <template v-else>
      <h1>{{ formatMessage(messages.signInWithLabel) }}</h1>

      <section class="third-party">
        <a class="btn" :href="getAuthUrl('discord', redirectTarget)">
          <SSODiscordIcon />
          <span>Discord</span>
        </a>
        <a class="btn" :href="getAuthUrl('github', redirectTarget)">
          <SSOGitHubIcon />
          <span>GitHub</span>
        </a>
        <a class="btn" :href="getAuthUrl('microsoft', redirectTarget)">
          <SSOMicrosoftIcon />
          <span>Microsoft</span>
        </a>
        <a class="btn" :href="getAuthUrl('google', redirectTarget)">
          <SSOGoogleIcon />
          <span>Google</span>
        </a>
        <a class="btn" :href="getAuthUrl('steam', redirectTarget)">
          <SSOSteamIcon />
          <span>Steam</span>
        </a>
        <a class="btn" :href="getAuthUrl('gitlab', redirectTarget)">
          <SSOGitLabIcon />
          <span>GitLab</span>
        </a>
      </section>

      <h1>{{ formatMessage(messages.usePasswordLabel) }}</h1>

      <section class="auth-form">
        <div class="iconified-input">
          <label for="email" hidden>{{ formatMessage(messages.emailUsernameLabel) }}</label>
          <MailIcon />
          <input
            id="email"
            v-model="email"
            type="text"
            autocomplete="username"
            class="auth-form__input"
            :placeholder="formatMessage(messages.emailUsernameLabel)"
          />
        </div>

        <div class="iconified-input">
          <label for="password" hidden>{{ formatMessage(messages.passwordLabel) }}</label>
          <KeyIcon />
          <input
            id="password"
            v-model="password"
            type="password"
            autocomplete="current-password"
            class="auth-form__input"
            :placeholder="formatMessage(messages.passwordLabel)"
          />
        </div>

        <NuxtTurnstile
          ref="turnstile"
          v-model="token"
          class="turnstile"
          :options="{ theme: $colorMode.value === 'light' ? 'light' : 'dark' }"
        />

        <button
          class="btn btn-primary continue-btn centered-btn"
          :disabled="!token"
          @click="beginPasswordSignIn()"
        >
          {{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
        </button>

        <div class="auth-form__additional-options">
          <IntlFormatted :message-id="messages.additionalOptionsLabel">
            <template #forgot-password-link="{ children }">
              <NuxtLink class="text-link" to="/auth/reset-password">
                <component :is="() => children" />
              </NuxtLink>
            </template>
            <template #create-account-link="{ children }">
              <NuxtLink class="text-link" :to="signUpLink">
                <component :is="() => children" />
              </NuxtLink>
            </template>
          </IntlFormatted>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup>
import {
  RightArrowIcon,
  SSOGitHubIcon,
  SSOMicrosoftIcon,
  SSOSteamIcon,
  SSOGoogleIcon,
  SSODiscordIcon,
  SSOGitLabIcon,
  KeyIcon,
  MailIcon,
} from 'omorphia'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  additionalOptionsLabel: {
    id: 'auth.sign-in.additional-options',
    defaultMessage:
      '<forgot-password-link>Forgot password?</forgot-password-link> • <create-account-link>Create an account</create-account-link>',
  },
  emailUsernameLabel: {
    id: 'auth.sign-in.email-username.label',
    defaultMessage: 'Email or username',
  },
  passwordLabel: {
    id: 'auth.sign-in.password.label',
    defaultMessage: 'Password',
  },
  signInWithLabel: {
    id: 'auth.sign-in.sign-in-with',
    defaultMessage: 'Sign in with',
  },
  signInTitle: {
    id: 'auth.sign-in.title',
    defaultMessage: 'Sign In',
  },
  twoFactorCodeInputPlaceholder: {
    id: 'auth.sign-in.2fa.placeholder',
    defaultMessage: 'Enter code...',
  },
  twoFactorCodeLabel: {
    id: 'auth.sign-in.2fa.label',
    defaultMessage: 'Enter two-factor code',
  },
  twoFactorCodeLabelDescription: {
    id: 'auth.sign-in.2fa.description',
    defaultMessage: 'Please enter a two-factor code to proceed.',
  },
  usePasswordLabel: {
    id: 'auth.sign-in.use-password',
    defaultMessage: 'Or use a password',
  },
})

useHead({
  title() {
    return `${formatMessage(messages.signInTitle)} - Modrinth`
  },
})

const auth = await useAuth()
const route = useNativeRoute()

const redirectTarget = route.query.redirect || ''

if (route.fullPath.includes('new_account=true')) {
  await navigateTo(
    `/auth/welcome?authToken=${route.query.code}${
      route.query.redirect ? `&redirect=${encodeURIComponent(route.query.redirect)}` : ''
    }`
  )
} else if (route.query.code) {
  await finishSignIn()
}

if (auth.value.user) {
  await finishSignIn()
}

const turnstile = ref()

const email = ref('')
const password = ref('')
const token = ref('')

const flow = ref(route.query.flow)

const signUpLink = computed(
  () => `/auth/sign-up${route.query.redirect ? `?redirect=${route.query.redirect}` : ''}`
)

async function beginPasswordSignIn() {
  startLoading()
  try {
    const res = await useBaseFetch('auth/login', {
      method: 'POST',
      body: {
        username: email.value,
        password: password.value,
        challenge: token.value,
      },
    })

    if (res.flow) {
      flow.value = res.flow
    } else {
      await finishSignIn(res.session)
    }
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    turnstile.value?.reset()
  }
  stopLoading()
}

const twoFactorCode = ref(null)
async function begin2FASignIn() {
  startLoading()
  try {
    const res = await useBaseFetch('auth/login/2fa', {
      method: 'POST',
      body: {
        flow: flow.value,
        code: twoFactorCode.value ? twoFactorCode.value.toString() : twoFactorCode.value,
      },
    })

    await finishSignIn(res.session)
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    turnstile.value?.reset()
  }
  stopLoading()
}

async function finishSignIn(token) {
  if (token) {
    await useAuth(token)
    await useUser()
  }

  if (route.query.redirect) {
    const redirect = decodeURIComponent(route.query.redirect)
    await navigateTo(redirect, {
      replace: true,
    })
  } else {
    await navigateTo('/dashboard')
  }
}
</script>
