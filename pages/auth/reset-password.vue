<template>
  <div>
    <h1>{{ formatMessage(messages.longTitle) }}</h1>
    <section class="auth-form">
      <template v-if="step === 'choose_method'">
        <p>
          {{ formatMessage(methodChoiceMessages.description) }}
        </p>

        <div class="iconified-input">
          <label for="email" hidden>
            {{ formatMessage(methodChoiceMessages.emailUsernameLabel) }}
          </label>
          <MailIcon />
          <input
            id="email"
            v-model="email"
            type="text"
            autocomplete="username"
            class="auth-form__input"
            :placeholder="formatMessage(methodChoiceMessages.emailUsernamePlaceholder)"
          />
        </div>

        <NuxtTurnstile
          ref="turnstile"
          v-model="token"
          class="turnstile"
          :options="{ theme: $colorMode.value === 'light' ? 'light' : 'dark' }"
        />

        <button class="btn btn-primary centered-btn" :disabled="!token" @click="recovery">
          <SendIcon /> {{ formatMessage(methodChoiceMessages.action) }}
        </button>
      </template>
      <template v-else-if="step === 'passed_challenge'">
        <p>{{ formatMessage(postChallengeMessages.description) }}</p>

        <div class="iconified-input">
          <label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
          <KeyIcon />
          <input
            id="password"
            v-model="newPassword"
            type="password"
            autocomplete="new-password"
            class="auth-form__input"
            :placeholder="formatMessage(commonMessages.passwordLabel)"
          />
        </div>

        <div class="iconified-input">
          <label for="confirm-password" hidden>
            {{ formatMessage(commonMessages.passwordLabel) }}
          </label>
          <KeyIcon />
          <input
            id="confirm-password"
            v-model="confirmNewPassword"
            type="password"
            autocomplete="new-password"
            class="auth-form__input"
            :placeholder="formatMessage(postChallengeMessages.confirmPasswordLabel)"
          />
        </div>

        <button class="auth-form__input btn btn-primary continue-btn" @click="changePassword">
          {{ formatMessage(postChallengeMessages.action) }}
        </button>
      </template>
    </section>
  </div>
</template>
<script setup>
import { SendIcon, MailIcon, KeyIcon } from 'omorphia'

const { formatMessage } = useVIntl()

const methodChoiceMessages = defineMessages({
  description: {
    id: 'auth.reset-password.method-choice.description',
    defaultMessage:
      "Enter your email below and we'll send a recovery link to allow you to recover your account.",
  },
  emailUsernameLabel: {
    id: 'auth.reset-password.method-choice.email-username.label',
    defaultMessage: 'Email or username',
  },
  emailUsernamePlaceholder: {
    id: 'auth.reset-password.method-choice.email-username.placeholder',
    defaultMessage: 'Email',
  },
  action: {
    id: 'auth.reset-password.method-choice.action',
    defaultMessage: 'Send recovery email',
  },
})

const postChallengeMessages = defineMessages({
  description: {
    id: 'auth.reset-password.post-challenge.description',
    defaultMessage: 'Enter your new password below to gain access to your account.',
  },
  confirmPasswordLabel: {
    id: 'auth.reset-password.post-challenge.confirm-password.label',
    defaultMessage: 'Confirm password',
  },
  action: {
    id: 'auth.reset-password.post-challenge.action',
    defaultMessage: 'Reset password',
  },
})

// NOTE(Brawaru): Vite uses esbuild for minification so can't combine these
// because it'll keep the original prop names compared to consts, which names
// will be mangled.
const emailSentNotificationMessages = defineMessages({
  title: {
    id: 'auth.reset-password.notification.email-sent.title',
    defaultMessage: 'Email sent',
  },
  text: {
    id: 'auth.reset-password.notification.email-sent.text',
    defaultMessage:
      'An email with instructions has been sent to you if the email was previously saved on your account.',
  },
})

const passwordResetNotificationMessages = defineMessages({
  title: {
    id: 'auth.reset-password.notification.password-reset.title',
    defaultMessage: 'Password successfully reset',
  },
  text: {
    id: 'auth.reset-password.notification.password-reset.text',
    defaultMessage: 'You can now log-in into your account with your new password.',
  },
})

const messages = defineMessages({
  title: {
    id: 'auth.reset-password.title',
    defaultMessage: 'Reset Password',
  },
  longTitle: {
    id: 'auth.reset-password.title.long',
    defaultMessage: 'Reset your password',
  },
})

useHead({
  title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()
if (auth.value.user) {
  await navigateTo('/dashboard')
}

const route = useNativeRoute()

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
      title: formatMessage(emailSentNotificationMessages.title),
      text: formatMessage(emailSentNotificationMessages.text),
      type: 'success',
    })
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
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
      title: formatMessage(passwordResetNotificationMessages.title),
      text: formatMessage(passwordResetNotificationMessages.text),
      type: 'success',
    })
    await navigateTo('/auth/sign-in')
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    turnstile.value?.reset()
  }
  stopLoading()
}
</script>
