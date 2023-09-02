<template>
  <div>
    <h1>Reset your password</h1>
    <section class="auth-form">
      <template v-if="step === 'choose_method'">
        <p>
          Enter your email below and we'll send a recovery link to allow you to recover your
          account.
          <NuxtTurnstile ref="turnstile" v-model="token" class="turnstile" />
        </p>

        <div class="iconified-input">
          <label for="email" hidden>Email or username</label>
          <MailIcon />
          <input
            id="email"
            v-model="email"
            type="text"
            autocomplete="username"
            class="auth-form__input"
            placeholder="Email"
          />
        </div>

        <button class="btn btn-primary centered-btn" @click="recovery">
          <SendIcon /> Send recovery email
        </button>
      </template>
      <template v-else-if="step === 'passed_challenge'">
        <p>Enter your new password below to gain access to your account.</p>

        <div class="iconified-input">
          <label for="password" hidden>Password</label>
          <KeyIcon />
          <input
            id="password"
            v-model="newPassword"
            type="password"
            autocomplete="new-password"
            class="auth-form__input"
            placeholder="Password"
          />
        </div>

        <div class="iconified-input">
          <label for="confirm-password" hidden>Password</label>
          <KeyIcon />
          <input
            id="confirm-password"
            v-model="confirmNewPassword"
            type="password"
            autocomplete="new-password"
            class="auth-form__input"
            placeholder="Confirm password"
          />
        </div>

        <button class="auth-form__input btn btn-primary continue-btn" @click="changePassword">
          Reset password
        </button>
      </template>
    </section>
  </div>
</template>
<script setup>
import { SendIcon } from 'omorphia'
import MailIcon from 'assets/icons/auth/mail.svg'
import KeyIcon from 'assets/icons/auth/key.svg'

useHead({
  title: 'Reset Password - Modrinth',
})

const auth = await useAuth()
if (auth.value.user) {
  await navigateTo('/dashboard')
}

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

    addNotification({
      group: 'main',
      title: 'Email sent',
      text: 'An email with instructions has been sent to you if the email was previously saved on your account.',
      type: 'success',
    })
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

    addNotification({
      group: 'main',
      title: 'Password successfully reset',
      text: 'You can now log-in into your account with your new password.',
      type: 'success',
    })
    await navigateTo('/auth/sign-in')
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
