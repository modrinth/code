<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Legal</h1>
        <NavStack>
          <NavStackItem link="/legal/terms" label="Terms and Conditions">
            <TermsIcon />
          </NavStackItem>
          <NavStackItem link="/legal/privacy" label="Privacy Policy">
            <PrivacyIcon />
          </NavStackItem>
          <NavStackItem link="/legal/rules" label="Content Rules">
            <RulesIcon />
          </NavStackItem>
          <NavStackItem link="/legal/security" label="Security Notice">
            <ShieldIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtPage class="universal-card" :route="route" />
    </div>
  </div>
</template>

<script setup>
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import TermsIcon from '~/assets/images/utils/heart-handshake.svg'
import PrivacyIcon from '~/assets/images/utils/lock.svg'
import RulesIcon from '~/assets/images/sidebar/admin.svg'
import ShieldIcon from '~/assets/images/utils/shield.svg'

const route = useRoute()
</script>

<style lang="scss" scoped>
.normal-page__content :deep(a) {
  color: var(--color-link);
  text-decoration: underline;

  &:focus-visible,
  &:hover {
    color: var(--color-link-hover);
  }

  &:active {
    color: var(--color-link-active);
  }
}
</style>
