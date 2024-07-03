<template>
  <div>
    <h1>{{ formatMessage(messages.welcomeLongTitle) }}</h1>

    <section class="auth-form">
      <p>
        {{ formatMessage(messages.welcomeDescription) }}
      </p>

      <Checkbox
        v-model="subscribe"
        class="subscribe-btn"
        :label="formatMessage(messages.subscribeCheckbox)"
        :description="formatMessage(messages.subscribeCheckbox)"
      />

      <button class="btn btn-primary continue-btn centered-btn" @click="continueSignUp">
        {{ formatMessage(commonMessages.continueButton) }} <RightArrowIcon />
      </button>

      <p>
        <IntlFormatted :message-id="messages.tosLabel">
          <template #terms-link="{ children }">
            <NuxtLink to="/legal/terms" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
          <template #privacy-policy-link="{ children }">
            <NuxtLink to="/legal/privacy" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
        </IntlFormatted>
      </p>
    </section>
  </div>
</template>
<script setup>
import { Checkbox, RightArrowIcon } from 'omorphia'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  subscribeCheckbox: {
    id: 'auth.welcome.checkbox.subscribe',
    defaultMessage: 'Subscribe to updates about Modrinth',
  },
  tosLabel: {
    id: 'auth.welcome.label.tos',
    defaultMessage:
      "By creating an account, you have agreed to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
  },
  welcomeDescription: {
    id: 'auth.welcome.description',
    defaultMessage:
      'Thank you for creating an account. You can now follow and create projects, receive updates about your favorite projects, and more!',
  },
  welcomeLongTitle: {
    id: 'auth.welcome.long-title',
    defaultMessage: 'Welcome to Modrinth!',
  },
  welcomeTitle: {
    id: 'auth.welcome.title',
    defaultMessage: 'Welcome',
  },
})

useHead({
  title: () => `${formatMessage(messages.welcomeTitle)} - Modrinth`,
})

const subscribe = ref(true)

async function continueSignUp() {
  const route = useNativeRoute()

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
