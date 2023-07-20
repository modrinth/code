<template>
  <div class="auth-page-container">
    <template v-if="auth.user && auth.user.email_verified && !success">
      <h1>Email already verified</h1>
      <p>Your email is already verified!</p>
      <nuxt-link class="btn" link="/settings/account">
        <SettingsIcon /> Account settings
      </nuxt-link>
    </template>
    <template v-else-if="success">
      <h1>Email verification</h1>
      <p>Your email address has been successfully verified!</p>
      <nuxt-link v-if="auth.user" class="btn" to="/settings/account">
        <SettingsIcon /> Account settings
      </nuxt-link>
      <nuxt-link v-else to="/auth/sign-in" class="btn btn-primary continue-btn">
        Sign in <RightArrowIcon />
      </nuxt-link>
    </template>
    <template v-else>
      <h1>Email verification failed</h1>
      <p>
        We were unable to verify your email.
        <template v-if="auth.user">
          Try re-sending the verification email through the button below.
        </template>
        <template v-else>
          Try re-sending the verification email through your dashboard by signing in.
        </template>
      </p>
      <button v-if="auth.user" class="btn btn-primary continue-btn" @click="resendVerifyEmail">
        Resend verification email <RightArrowIcon />
      </button>
      <nuxt-link v-else to="/auth/sign-in" class="btn btn-primary continue-btn">
        Sign in <RightArrowIcon />
      </nuxt-link>
    </template>
  </div>
</template>
<script setup>
import { SettingsIcon, RightArrowIcon } from 'omorphia'

useHead({
  title: 'Verify Email - Modrinth',
})

const auth = await useAuth()

const success = ref(false)
const route = useRoute()

if (route.query.flow) {
  try {
    const emailVerified = useState('emailVerified', () => null)

    if (emailVerified.value === null) {
      await useBaseFetch('auth/email/verify', {
        method: 'POST',
        body: {
          flow: route.query.flow,
        },
      })
      emailVerified.value = true
      success.value = true
    }

    if (emailVerified.value) {
      success.value = true

      if (auth.value.token) {
        await useAuth(auth.value.token)
      }
    }
  } catch (err) {
    success.value = false
  }
}
</script>
