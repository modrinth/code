<template>
  <div class="auth-page-container">
    <h1>Reset your password</h1>
    <template v-if="step === 'choose_method'">
      <p>
        Enter your email below and we'll send a recovery link to allow you to recover your account.
        <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />
      </p>
      <label for="email" hidden>Email or username</label>
      <input id="email" v-model="email" type="text" placeholder="Email or username" />
      <button class="btn btn-primary continue-btn" @click="recovery">Send recovery email</button>
    </template>
    <template v-else-if="step === 'passed_challenge'">
      <p>Enter your new password below to gain access to your account.</p>
      <label for="password" hidden>Password</label>
      <input id="password" v-model="newPassword" type="password" placeholder="Password" />
      <label for="confirm-password" hi2dden>Password</label>
      <input
        id="confirm-password"
        v-model="confirmNewPassword"
        type="password"
        placeholder="Confirm password"
      />
      <button class="btn btn-primary continue-btn" @click="changePassword">Reset password</button>
    </template>
  </div>
</template>
<script setup>
useHead({
  title: 'Reset Password - Modrinth',
})

const auth = await useAuth()
if (auth.value.user) {
  await navigateTo('/dashboard')
}

const data = useNuxtApp()
const route = useRoute()

const step = ref('choose_method')

if (route.query.flow) {
  step.value = 'passed_challenge'
}

const turnstile = ref()

const email = ref('')
const token = ref('')

async function recovery() {
  startLoading()
  try {
    await useBaseFetch('auth/password/reset', {
      method: 'POST',
      body: {
        username: email.value,
        challenge: token.value,
      },
    })

    data.$notify({
      group: 'main',
      title: 'Email sent',
      text: 'An email with instructions has been sent to you if the email was previously saved on your account.',
      type: 'success',
    })
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

const newPassword = ref('')
const confirmNewPassword = ref('')

async function changePassword() {
  startLoading()
  try {
    await useBaseFetch('auth/password', {
      method: 'PATCH',
      body: {
        new_password: newPassword.value,
        flow: route.query.flow,
      },
    })

    data.$notify({
      group: 'main',
      title: 'Password successfully reset',
      text: 'You can now log-in into your account with your new password.',
      type: 'success',
    })
    await navigateTo('/auth/sign-in')
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
