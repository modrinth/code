<template>
  <div class="auth-page-container">
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
      />

      <button class="btn btn-primary continue-btn" @click="loginTwoFactor">
        Sign in <RightArrowIcon />
      </button>
    </template>
    <template v-else>
      <h1>Continue with</h1>
      <div class="third-party">
        <a class="btn discord-btn" :href="getAuthUrl('discord')">
          <DiscordIcon /> <span>Discord</span>
        </a>
        <a class="btn github-btn" :href="getAuthUrl('github')"
          ><GitHubIcon /> <span>GitHub</span></a
        >
        <a class="btn microsoft-btn" :href="getAuthUrl('microsoft')">
          <MicrosoftIcon /> <span>Microsoft</span>
        </a>
        <a class="btn google-btn" :href="getAuthUrl('google')">
          <GoogleIcon /> <span>Google</span>
        </a>
        <a class="btn apple-btn" :href="getAuthUrl('steam')"><SteamIcon /> <span>Steam</span></a>
        <a class="btn gitlab-btn" :href="getAuthUrl('gitlab')">
          <GitLabIcon /> <span>GitLab</span></a
        >
      </div>
      <div class="text-divider">
        <div></div>
        <span>or</span>
        <div></div>
      </div>
      <label for="email" hidden>Email or username</label>
      <input id="email" v-model="email" type="text" placeholder="Email or username" />
      <label for="password" hidden>Password</label>
      <input id="password" v-model="password" type="password" placeholder="Password" />
      <div class="account-options">
        <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />
        <nuxt-link class="text-link" to="/auth/reset-password">Forgot password?</nuxt-link>
      </div>
      <button class="btn btn-primary continue-btn" @click="loginPassword()">
        Continue <RightArrowIcon />
      </button>
      <p>
        Don't have an account yet?
        <nuxt-link
          class="text-link"
          :to="`/auth/sign-up${route.query.redirect ? `?redirect=${route.query.redirect}` : ''}`"
        >
          Create one.
        </nuxt-link>
      </p>
    </template>
  </div>
</template>

<script setup>
import { GitHubIcon, RightArrowIcon } from 'omorphia'
import DiscordIcon from 'assets/images/utils/discord.svg'
import GoogleIcon from 'assets/images/utils/google.svg'
import SteamIcon from 'assets/images/utils/steam.svg'
import MicrosoftIcon from 'assets/images/utils/microsoft.svg'
import GitLabIcon from 'assets/images/utils/gitlab.svg'

useHead({
  title: 'Sign In - Modrinth',
})

const auth = await useAuth()

const route = useRoute()
if (route.fullPath.includes('new_account=true')) {
  await navigateTo(
    `/auth/welcome?authToken=${route.query.code}${
      route.query.redirect ? `&redirect=${encodeURIComponent(route.query.redirect)}` : ''
    }`
  )
} else if (route.query.code) {
  await loginHandler()
}

if (auth.value.user) {
  await navigateTo('/dashboard')
}

const data = useNuxtApp()

const turnstile = ref()

const email = ref('')
const password = ref('')
const token = ref('')

const flow = ref(route.query.flow)

async function loginPassword() {
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
      await loginHandler(res.session)
    }
  } catch (err) {
    data.$notify({
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
async function loginTwoFactor() {
  startLoading()
  try {
    const res = await useBaseFetch('auth/login/2fa', {
      method: 'POST',
      body: {
        flow: flow.value,
        code: twoFactorCode.value ? twoFactorCode.value.toString() : twoFactorCode.value,
      },
    })

    await loginHandler(res.session)
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    turnstile.value?.reset()
  }
  stopLoading()
}
async function loginHandler(token) {
  if (token) {
    await useAuth(token)
    await useUser()
  }

  if (route.query.redirect) {
    await navigateTo(route.query.redirect)
  } else {
    await navigateTo('/dashboard')
  }
}
</script>
<style lang="scss" scoped>
.totp {
  justify-content: center;
}

.totp-codes {
  justify-content: center;
  display: grid;
  gap: var(--gap-md);
  grid-template-columns: repeat(2, 1fr);
  width: 100%;
}

.account-options {
  display: flex;
  width: 100%;
  margin-block-start: 0 !important;
}
.account-options a {
  margin-left: auto;
}
</style>
