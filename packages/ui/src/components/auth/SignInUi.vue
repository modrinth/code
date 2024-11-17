<template>
  <div class="flex flex-col gap-8">
    <span class="text-2xl font-bold leading-tight m-0 text-contrast -mb-4">{{
      formatMessage(messages.signInWithLabel)
    }}</span>

    <ThirdPartyAuthButtons :type="thirdPartyButtonHandlerType" :handler="thirdPartyButtonHandler" />

    <span class="text-2xl font-bold leading-tight m-0 text-contrast -mb-4">{{
      formatMessage(messages.usePasswordLabel)
    }}</span>

    <section class="flex flex-col gap-3">
      <div class="iconified-input">
        <label for="email" hidden>{{ formatMessage(messages.emailUsernameLabel) }}</label>
        <MailIcon />
        <input
          id="email"
          v-model="email"
          type="text"
          autocomplete="username"
          class="auth-form__input"
          :placeholder="formatMessage(messages.emailUsernameLabel)"
        />
      </div>

      <div class="iconified-input">
        <label for="password" hidden>{{ formatMessage(messages.passwordLabel) }}</label>
        <KeyIcon />
        <input
          id="password"
          v-model="password"
          type="password"
          autocomplete="current-password"
          class="w-full flex-basis-auto"
          :placeholder="formatMessage(messages.passwordLabel)"
        />
      </div>

      <div v-if="$slots.turnstile" class="turnstile-container">
        <slot name="turnstile" />
      </div>

      <ButtonStyled color="brand" size="large">
        <button
          :disabled="!completedTurnstile || !email || !password"
          @click="() => onSignIn(email, password)"
          class="mx-auto"
        >
          {{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
        </button>
      </ButtonStyled>

      <div class="flex items-center justify-center gap-3 flex-wrap">
        <IntlFormatted :message-id="messages.additionalOptionsLabel">
          <template #forgot-password-link="{ children }">
            <AutoLink class="text-link" :to="forgotPasswordLink">
              <component :is="() => children" />
            </AutoLink>
          </template>
          <template #create-account-link="{ children }">
            <AutoLink class="text-link" :to="createAccountLink">
              <component :is="() => children" />
            </AutoLink>
          </template>
        </IntlFormatted>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { KeyIcon, MailIcon, RightArrowIcon } from '@modrinth/assets'
import ThirdPartyAuthButtons from './ThirdPartyAuthButtons.vue'
import type { ActionProvider, HrefProvider } from './ThirdPartyAuthButtons.vue'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import AutoLink from '../base/AutoLink.vue'
import { ref } from 'vue'
import { commonMessages } from '../../utils/common-messages'
import ButtonStyled from '../base/ButtonStyled.vue'

const { formatMessage } = useVIntl()

const email = ref('')
const password = ref('')

defineProps<{
  completedTurnstile: boolean
  forgotPasswordLink: string
  createAccountLink: string
  thirdPartyButtonHandlerType: 'href' | 'action'
  thirdPartyButtonHandler: HrefProvider | ActionProvider
  onSignIn: (email: string, password: string) => void
}>()

const messages = defineMessages({
  additionalOptionsLabel: {
    id: 'auth.sign-in.additional-options',
    defaultMessage:
      '<forgot-password-link>Forgot password?</forgot-password-link> • <create-account-link>Create an account</create-account-link>',
  },
  emailUsernameLabel: {
    id: 'auth.sign-in.email-username.label',
    defaultMessage: 'Email or username',
  },
  passwordLabel: {
    id: 'auth.sign-in.password.label',
    defaultMessage: 'Password',
  },
  signInWithLabel: {
    id: 'auth.sign-in.sign-in-with',
    defaultMessage: 'Sign in with',
  },
  signInTitle: {
    id: 'auth.sign-in.title',
    defaultMessage: 'Sign In',
  },
  twoFactorCodeInputPlaceholder: {
    id: 'auth.sign-in.2fa.placeholder',
    defaultMessage: 'Enter code...',
  },
  twoFactorCodeLabel: {
    id: 'auth.sign-in.2fa.label',
    defaultMessage: 'Enter two-factor code',
  },
  twoFactorCodeLabelDescription: {
    id: 'auth.sign-in.2fa.description',
    defaultMessage: 'Please enter a two-factor code to proceed.',
  },
  usePasswordLabel: {
    id: 'auth.sign-in.use-password',
    defaultMessage: 'Or use a password',
  },
})
</script>
<style scoped lang="scss">
.turnstile-container {
  display: flex;
  justify-content: center;
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 2px solid var(--color-button-bg);
  height: 65px;
  width: 100%;

  :deep(> div) {
    position: relative;
    top: -2px;
    min-width: calc(100% + 4px);
  }
}
</style>
