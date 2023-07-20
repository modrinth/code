<template>
  <div class="auth-page-container">
    <h1>Create your account</h1>
    <div class="third-party">
      <a class="btn discord-btn" :href="getAuthUrl('discord')">
        <DiscordIcon /> <span>Discord</span>
      </a>
      <a class="btn github-btn" :href="getAuthUrl('github')"><GitHubIcon /> <span>GitHub</span></a>
      <a class="btn microsoft-btn" :href="getAuthUrl('microsoft')">
        <MicrosoftIcon /> <span>Microsoft</span>
      </a>
      <a class="btn google-btn" :href="getAuthUrl('google')">
        <GoogleIcon /> <span>Google</span>
      </a>
      <a class="btn apple-btn" :href="getAuthUrl('steam')"><SteamIcon /> <span>Steam</span></a>
      <a class="btn gitlab-btn" :href="getAuthUrl('gitlab')"> <GitLabIcon /> <span>GitLab</span></a>
    </div>
    <div class="text-divider">
      <div></div>
      <span>or</span>
      <div></div>
    </div>
    <label for="email" hidden>Email</label>
    <input id="email" v-model="email" type="text" placeholder="Email" />
    <label for="username" hidden>Username</label>
    <input id="username" v-model="username" type="text" placeholder="Username" />
    <label for="password" hidden>Password</label>
    <input id="password" v-model="password" type="password" placeholder="Password" />
    <label for="confirm-password" hidden>Password</label>
    <input
      id="confirm-password"
      v-model="confirmPassword"
      type="password"
      placeholder="Confirm password"
    />
    <Checkbox v-model="subscribe" class="subscribe-btn" label="Subscribe updates about Modrinth" />
    <p>
      By creating an account, you agree to Modrinth's
      <nuxt-link to="/legal/terms" class="text-link">terms</nuxt-link> and
      <nuxt-link to="/legal/privacy" class="text-link">privacy policy</nuxt-link>.
    </p>
    <button class="btn btn-primary continue-btn" @click="createAccount">
      Create account <RightArrowIcon />
    </button>
    <p>
      Already have an account yet?
      <nuxt-link
        class="text-link"
        :to="`/auth/sign-in${route.query.redirect ? `?redirect=${route.query.redirect}` : ''}`"
      >
        Sign in.
      </nuxt-link>
      <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />
    </p>
  </div>
</template>

<script setup>
import { GitHubIcon, RightArrowIcon, Checkbox } from 'omorphia'
import DiscordIcon from 'assets/images/utils/discord.svg'
import GoogleIcon from 'assets/images/utils/google.svg'
import SteamIcon from 'assets/images/utils/steam.svg'
import MicrosoftIcon from 'assets/images/utils/microsoft.svg'
import GitLabIcon from 'assets/images/utils/gitlab.svg'

useHead({
  title: 'Sign Up - Modrinth',
})

const auth = await useAuth()
const route = useRoute()

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

const data = useNuxtApp()

const turnstile = ref()

const email = ref('')
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const token = ref('')
const subscribe = ref(true)

async function createAccount() {
  startLoading()
  try {
    if (confirmPassword.value !== password.value) {
      data.$notify({
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
</script>
<style lang="scss" scoped>
.subscribe-btn {
  margin-block-start: 0 !important;
}
</style>
