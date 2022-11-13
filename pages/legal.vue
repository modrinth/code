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
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtChild class="universal-card" />
    </div>
  </div>
</template>

<script>
import NavStack from '~/components/ui/NavStack'
import NavStackItem from '~/components/ui/NavStackItem'

import TermsIcon from '~/assets/images/utils/heart-handshake.svg?inline'
import PrivacyIcon from '~/assets/images/utils/lock.svg?inline'
import RulesIcon from '~/assets/images/sidebar/admin.svg?inline'

export default {
  name: 'Settings',
  components: {
    NavStack,
    NavStackItem,
    TermsIcon,
    PrivacyIcon,
    RulesIcon,
  },
}
</script>

<style lang="scss" scoped>
.normal-page__content ::v-deep a {
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
