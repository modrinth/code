<template>
  <div>
    <template v-if="auth.user && auth.user.email_verified && !success">
      <h1>{{ formatMessage(alreadyVerifiedMessages.title) }}</h1>

      <section class="auth-form">
        <p>{{ formatMessage(alreadyVerifiedMessages.description) }}</p>

        <NuxtLink class="btn" to="/settings/account">
          <SettingsIcon /> {{ formatMessage(messages.accountSettings) }}
        </NuxtLink>
      </section>
    </template>

    <template v-else-if="success">
      <h1>{{ formatMessage(postVerificationMessages.title) }}</h1>

      <section class="auth-form">
        <p>{{ formatMessage(postVerificationMessages.description) }}</p>

        <NuxtLink v-if="auth.user" class="btn" link="/settings/account">
          <SettingsIcon /> {{ formatMessage(messages.accountSettings) }}
        </NuxtLink>
        <NuxtLink v-else to="/auth/sign-in" class="btn btn-primary continue-btn centered-btn">
          {{ formatMessage(messages.signIn) }} <RightArrowIcon />
        </NuxtLink>
      </section>
    </template>

    <template v-else>
      <h1>{{ formatMessage(failedVerificationMessages.title) }}</h1>

      <section class="auth-form">
        <p>
          <template v-if="auth.user">
            {{ formatMessage(failedVerificationMessages.loggedInDescription) }}
          </template>
          <template v-else>
            {{ formatMessage(failedVerificationMessages.description) }}
          </template>
        </p>

        <button v-if="auth.user" class="btn btn-primary continue-btn" @click="resendVerifyEmail">
          {{ formatMessage(failedVerificationMessages.action) }} <RightArrowIcon />
        </button>

        <NuxtLink v-else to="/auth/sign-in" class="btn btn-primary continue-btn centered-btn">
          {{ formatMessage(messages.signIn) }} <RightArrowIcon />
        </NuxtLink>
      </section>
    </template>
  </div>
</template>
<script setup>
import { SettingsIcon, RightArrowIcon } from 'omorphia'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  title: {
    id: 'auth.verify-email.title',
    defaultMessage: 'Verify Email',
  },
  accountSettings: {
    id: 'auth.verify-email.action.account-settings',
    defaultMessage: 'Account settings',
  },
  signIn: {
    id: 'auth.verify-email.action.sign-in',
    defaultMessage: 'Sign in',
  },
})

const alreadyVerifiedMessages = defineMessages({
  title: {
    id: 'auth.verify-email.already-verified.title',
    defaultMessage: 'Email already verified',
  },
  description: {
    id: 'auth.verify-email.already-verified.description',
    defaultMessage: 'Your email is already verified!',
  },
})

const postVerificationMessages = defineMessages({
  title: {
    id: 'auth.verify-email.post-verification.title',
    defaultMessage: 'Email verification',
  },
  description: {
    id: 'auth.verify-email.post-verification.description',
    defaultMessage: 'Your email address has been successfully verified!',
  },
})

const failedVerificationMessages = defineMessages({
  title: {
    id: 'auth.verify-email.failed-verification.title',
    defaultMessage: 'Email verification failed',
  },
  description: {
    id: 'auth.verify-email.failed-verification.description',
    defaultMessage:
      'We were unable to verify your email. Try re-sending the verification email through your dashboard by signing in.',
  },
  loggedInDescription: {
    id: 'auth.verify-email.failed-verification.description.logged-in',
    defaultMessage:
      'We were unable to verify your email. Try re-sending the verification email through the button below.',
  },
  action: {
    id: 'auth.verify-email.failed-verification.action',
    defaultMessage: 'Resend verification email',
  },
})

useHead({
  title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()

const success = ref(false)
const route = useNativeRoute()

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
