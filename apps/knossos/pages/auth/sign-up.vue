<template>
  <div>
    <h1>{{ formatMessage(messages.signUpWithTitle) }}</h1>

    <section class="third-party">
      <a class="btn discord-btn" :href="getAuthUrl('discord', redirectTarget)">
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

    <h1>{{ formatMessage(messages.createAccountTitle) }}</h1>

    <section class="auth-form">
      <div class="iconified-input">
        <label for="email" hidden>{{ formatMessage(messages.emailLabel) }}</label>
        <MailIcon />
        <input
          id="email"
          v-model="email"
          type="email"
          autocomplete="username"
          class="auth-form__input"
          :placeholder="formatMessage(messages.emailLabel)"
        />
      </div>

      <div class="iconified-input">
        <label for="username" hidden>{{ formatMessage(messages.usernameLabel) }}</label>
        <UserIcon />
        <input
          id="username"
          v-model="username"
          type="text"
          autocomplete="username"
          class="auth-form__input"
          :placeholder="formatMessage(messages.usernameLabel)"
        />
      </div>

      <div class="iconified-input">
        <label for="password" hidden>{{ formatMessage(messages.passwordLabel) }}</label>
        <KeyIcon />
        <input
          id="password"
          v-model="password"
          class="auth-form__input"
          type="password"
          autocomplete="new-password"
          :placeholder="formatMessage(messages.passwordLabel)"
        />
      </div>

      <div class="iconified-input">
        <label for="confirm-password" hidden>{{ formatMessage(messages.passwordLabel) }}</label>
        <KeyIcon />
        <input
          id="confirm-password"
          v-model="confirmPassword"
          type="password"
          autocomplete="new-password"
          class="auth-form__input"
          :placeholder="formatMessage(messages.confirmPasswordLabel)"
        />
      </div>

      <Checkbox
        v-model="subscribe"
        class="subscribe-btn"
        :label="formatMessage(messages.subscribeLabel)"
        :description="formatMessage(messages.subscribeLabel)"
      />

      <p>
        <IntlFormatted :message-id="messages.legalDisclaimer">
          <template #terms-link="{ children }">
            <NuxtLink to="/legal/terms" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
          <template #privacy-policy-link="{ children }">
            <NuxtLink to="/legal/privacy" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
        </IntlFormatted>
      </p>

      <NuxtTurnstile
        ref="turnstile"
        v-model="token"
        class="turnstile"
        :options="{ theme: $colorMode.value === 'light' ? 'light' : 'dark' }"
      />

      <button
        class="btn btn-primary continue-btn centered-btn"
        :disabled="!token"
        @click="createAccount"
      >
        {{ formatMessage(messages.createAccountButton) }} <RightArrowIcon />
      </button>

      <div class="auth-form__additional-options">
        {{ formatMessage(messages.alreadyHaveAccountLabel) }}
        <NuxtLink class="text-link" :to="signInLink">
          {{ formatMessage(commonMessages.signInButton) }}
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<script setup>
import {
  RightArrowIcon,
  UserIcon,
  Checkbox,
  SSOGitHubIcon,
  SSOMicrosoftIcon,
  SSOGoogleIcon,
  SSOSteamIcon,
  SSODiscordIcon,
  KeyIcon,
  MailIcon,
  SSOGitLabIcon,
} from 'omorphia'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  title: {
    id: 'auth.sign-up.title',
    defaultMessage: 'Sign Up',
  },
  signUpWithTitle: {
    id: 'auth.sign-up.title.sign-up-with',
    defaultMessage: 'Sign up with',
  },
  createAccountTitle: {
    id: 'auth.sign-up.title.create-account',
    defaultMessage: 'Or create an account yourself',
  },
  emailLabel: {
    id: 'auth.sign-up.email.label',
    defaultMessage: 'Email',
  },
  usernameLabel: {
    id: 'auth.sign-up.label.username',
    defaultMessage: 'Username',
  },
  passwordLabel: {
    id: 'auth.sign-up.password.label',
    defaultMessage: 'Password',
  },
  confirmPasswordLabel: {
    id: 'auth.sign-up.confirm-password.label',
    defaultMessage: 'Confirm password',
  },
  subscribeLabel: {
    id: 'auth.sign-up.subscribe.label',
    defaultMessage: 'Subscribe to updates about Modrinth',
  },
  legalDisclaimer: {
    id: 'auth.sign-up.legal-dislaimer',
    defaultMessage:
      "By creating an account, you agree to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
  },
  createAccountButton: {
    id: 'auth.sign-up.action.create-account',
    defaultMessage: 'Create account',
  },
  alreadyHaveAccountLabel: {
    id: 'auth.sign-up.sign-in-option.title',
    defaultMessage: 'Already have an account?',
  },
})

useHead({
  title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()
const route = useNativeRoute()

const redirectTarget = route.query.redirect

if (route.fullPath.includes('new_account=true')) {
  await navigateTo(
    `/auth/welcome?authToken=${route.query.code}${
      route.query.redirect ? `&redirect=${encodeURIComponent(route.query.redirect)}` : ''
    }`
  )
}

if (auth.value.user) {
  await navigateTo('/dashboard')
}

const turnstile = ref()

const email = ref('')
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const token = ref('')
const subscribe = ref(true)

const signInLink = computed(
  () => `/auth/sign-in${route.query.redirect ? `?redirect=${route.query.redirect}` : ''}`
)

async function createAccount() {
  startLoading()
  try {
    if (confirmPassword.value !== password.value) {
      addNotification({
        group: 'main',
        title: formatMessage(commonMessages.errorNotificationTitle),
        text: formatMessage({
          id: 'auth.sign-up.notification.password-mismatch.text',
          defaultMessage: 'Passwords do not match!',
        }),
        type: 'error',
      })
      turnstile.value?.reset()
    }

    const res = await useBaseFetch('auth/create', {
      method: 'POST',
      body: {
        username: username.value,
        password: password.value,
        email: email.value,
        challenge: token.value,
        sign_up_newsletter: subscribe.value,
      },
    })

    await useAuth(res.session)
    await useUser()

    if (route.query.redirect) {
      await navigateTo(route.query.redirect)
    } else {
      await navigateTo('/dashboard')
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
</script>
