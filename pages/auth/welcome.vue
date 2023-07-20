<template>
  <div class="auth-page-container">
    <h1>Welcome to Modrinth!</h1>
    <p>
      Thank you for creating an account. You can now follow and create projects, receive updates
      about your favorite projects, and more!
    </p>
    <Checkbox v-model="subscribe" class="subscribe-btn" label="Subscribe updates about Modrinth" />
    <button class="btn btn-primary continue-btn" @click="continueSignUp">Continue</button>
    <p>
      By creating an account, you agree to Modrinth's
      <nuxt-link to="/legal/terms" class="text-link">terms</nuxt-link> and
      <nuxt-link to="/legal/privacy" class="text-link">privacy policy</nuxt-link>.
    </p>
  </div>
</template>
<script setup>
import { Checkbox } from 'omorphia'

useHead({
  title: 'Welcome - Modrinth',
})

const subscribe = ref(true)

async function continueSignUp() {
  const route = useRoute()

  await useAuth(route.query.authToken)
  await useUser()

  if (subscribe.value) {
    try {
      await useBaseFetch('auth/email/subscribe', {
        method: 'POST',
      })
    } catch {}
  }

  if (route.query.redirect) {
    await navigateTo(route.query.redirect)
  } else {
    await navigateTo('/dashboard')
  }
}
</script>
