<template>
  <div>
    <template v-if="flow">
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
        autocomplete="one-time-code"
        autofocus
        @keyup.enter="begin2FASignIn"
      />

      <button class="btn btn-primary continue-btn" @click="begin2FASignIn">
        Sign in <RightArrowIcon />
      </button>
    </template>
    <template v-else>
      <h1>Sign in with</h1>

      <section class="third-party">
        <a class="btn" :href="getAuthUrl('discord', redirectTarget)">
          <DiscordIcon />
          <span>Discord</span>
        </a>
        <a class="btn" :href="getAuthUrl('github', redirectTarget)">
          <GitHubIcon />
          <span>GitHub</span>
        </a>
        <a class="btn" :href="getAuthUrl('microsoft', redirectTarget)">
          <MicrosoftIcon />
          <span>Microsoft</span>
        </a>
        <a class="btn" :href="getAuthUrl('google', redirectTarget)">
          <GoogleIcon />
          <span>Google</span>
        </a>
        <a class="btn" :href="getAuthUrl('steam', redirectTarget)">
          <SteamIcon />
          <span>Steam</span>
        </a>
        <a class="btn" :href="getAuthUrl('gitlab', redirectTarget)">
          <GitLabIcon />
          <span>GitLab</span>
        </a>
      </section>

      <h1>Or use a password</h1>

      <section class="auth-form">
        <div class="iconified-input">
          <label for="email" hidden>Email or username</label>
          <MailIcon />
          <input
            id="email"
            v-model="email"
            type="text"
            autocomplete="username"
            class="auth-form__input"
            placeholder="Email or username"
          />
        </div>

        <div class="iconified-input">
          <label for="password" hidden>Password</label>
          <KeyIcon />
          <input
            id="password"
            v-model="password"
            type="password"
            autocomplete="current-password"
            class="auth-form__input"
            placeholder="Password"
          />
        </div>

        <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />

        <button class="btn btn-primary continue-btn centered-btn" @click="beginPasswordSignIn()">
          Sign in <RightArrowIcon />
        </button>

        <div class="auth-form__additional-options">
          <NuxtLink class="text-link" to="/auth/reset-password">Forgot password?</NuxtLink>
          <p>•</p>
          <NuxtLink class="text-link" :to="signUpLink"> Create an account</NuxtLink>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup>
import { RightArrowIcon } from 'omorphia'
import GitHubIcon from 'assets/icons/auth/sso-github.svg'
import MicrosoftIcon from 'assets/icons/auth/sso-microsoft.svg'
import GoogleIcon from 'assets/icons/auth/sso-google.svg'
import SteamIcon from 'assets/icons/auth/sso-steam.svg'
import DiscordIcon from 'assets/icons/auth/sso-discord.svg'
import KeyIcon from 'assets/icons/auth/key.svg'
import MailIcon from 'assets/icons/auth/mail.svg'
import GitLabIcon from 'assets/icons/auth/sso-gitlab.svg'

useHead({
  title: 'Sign In - Modrinth',
})

const auth = await useAuth()
const route = useRoute()

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
      title: 'An error occurred',
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
      title: 'An error occurred',
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
