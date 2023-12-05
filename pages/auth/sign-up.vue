<template>
  <div>
    <h1>Sign up with</h1>

    <section class="third-party">
      <a class="btn discord-btn" :href="getAuthUrl('discord', redirectTarget)">
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

    <h1>Or create an account yourself</h1>

    <section class="auth-form">
      <div class="iconified-input">
        <label for="email" hidden>Email</label>
        <MailIcon />
        <input
          id="email"
          v-model="email"
          type="email"
          autocomplete="username"
          class="auth-form__input"
          placeholder="Email"
        />
      </div>

      <div class="iconified-input">
        <label for="username" hidden>Username</label>
        <UserIcon />
        <input
          id="username"
          v-model="username"
          type="text"
          autocomplete="username"
          class="auth-form__input"
          placeholder="Username"
        />
      </div>

      <div class="iconified-input">
        <label for="password" hidden>Password</label>
        <KeyIcon />
        <input
          id="password"
          v-model="password"
          class="auth-form__input"
          type="password"
          autocomplete="new-password"
          placeholder="Password"
        />
      </div>

      <div class="iconified-input">
        <label for="confirm-password" hidden>Password</label>
        <KeyIcon />
        <input
          id="confirm-password"
          v-model="confirmPassword"
          type="password"
          autocomplete="new-password"
          class="auth-form__input"
          placeholder="Confirm password"
        />
      </div>

      <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />

      <Checkbox
        v-model="subscribe"
        class="subscribe-btn"
        label="Subscribe to updates about Modrinth"
        description="Subscribe to updates about Modrinth"
      />

      <p>
        By creating an account, you agree to Modrinth's
        <NuxtLink to="/legal/terms" class="text-link">Terms</NuxtLink> and
        <NuxtLink to="/legal/privacy" class="text-link">Privacy Policy</NuxtLink>.
      </p>

      <button class="btn btn-primary continue-btn centered-btn" @click="createAccount">
        Create account <RightArrowIcon />
      </button>

      <div class="auth-form__additional-options">
        Already have an account?
        <NuxtLink class="text-link" :to="signInLink">Sign in</NuxtLink>
      </div>
    </section>
  </div>
</template>

<script setup>
import { RightArrowIcon, UserIcon, Checkbox } from 'omorphia'
import GitHubIcon from 'assets/icons/auth/sso-github.svg'
import MicrosoftIcon from 'assets/icons/auth/sso-microsoft.svg'
import GoogleIcon from 'assets/icons/auth/sso-google.svg'
import SteamIcon from 'assets/icons/auth/sso-steam.svg'
import DiscordIcon from 'assets/icons/auth/sso-discord.svg'
import KeyIcon from 'assets/icons/auth/key.svg'
import MailIcon from 'assets/icons/auth/mail.svg'
import GitLabIcon from 'assets/icons/auth/sso-gitlab.svg'

useHead({
  title: 'Sign Up - Modrinth',
})

const auth = await useAuth()
const route = useRoute()

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
        title: 'An error occurred',
        text: 'Passwords do not match!',
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
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    turnstile.value?.reset()
  }
  stopLoading()
}
</script>
