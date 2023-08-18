<template>
  <div>
    <template v-if="auth.user && auth.user.email_verified && !success">
      <h1>Email already verified</h1>

      <section class="auth-form">
        <p>Your email is already verified!</p>

        <NuxtLink class="btn" to="/settings/account"> <SettingsIcon /> Account settings </NuxtLink>
      </section>
    </template>

    <template v-else-if="success">
      <h1>Email verification</h1>

      <section class="auth-form">
        <p>Your email address has been successfully verified!</p>

        <NuxtLink v-if="auth.user" class="btn" link="/settings/account">
          <SettingsIcon /> Account settings
        </NuxtLink>
        <NuxtLink v-else to="/auth/sign-in" class="btn btn-primary continue-btn centered-btn">
          Sign in <RightArrowIcon />
        </NuxtLink>
      </section>
    </template>

    <template v-else>
      <h1>Email verification failed</h1>

      <section class="auth-form">
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

        <NuxtLink v-else to="/auth/sign-in" class="btn btn-primary continue-btn centered-btn">
          Sign in <RightArrowIcon />
        </NuxtLink>
      </section>
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
