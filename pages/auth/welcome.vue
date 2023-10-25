<template>
  <div>
    <h1>Welcome to Modrinth!</h1>

    <section class="auth-form">
      <p>
        Thank you for creating an account. You can now follow and create projects, receive updates
        about your favorite projects, and more!
      </p>

      <Checkbox
        v-model="subscribe"
        class="subscribe-btn"
        label="Subscribe to updates about Modrinth"
        description="Subscribe to updates about Modrinth"
      />

      <button class="btn btn-primary continue-btn centered-btn" @click="continueSignUp">
        Continue <RightArrowIcon />
      </button>

      <p>
        By creating an account, you have agreed to Modrinth's
        <NuxtLink to="/legal/terms" class="text-link">Terms</NuxtLink> and
        <NuxtLink to="/legal/privacy" class="text-link">Privacy Policy</NuxtLink>.
      </p>
    </section>
  </div>
</template>
<script setup>
import { Checkbox, RightArrowIcon } from 'omorphia'

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
