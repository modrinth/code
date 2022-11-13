<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Settings</h1>
        <NavStack>
          <NavStackItem link="/settings" label="Appearance">
            <PaintbrushIcon />
          </NavStackItem>
          <template v-if="$auth.user">
            <h3>User settings</h3>
            <NavStackItem link="/settings/account" label="Account">
              <UserIcon />
            </NavStackItem>
            <NavStackItem link="/settings/follows" label="Followed projects">
              <HeartIcon />
            </NavStackItem>
            <NavStackItem
              link="/settings/monetization"
              label="Monetization"
              beta
            >
              <CurrencyIcon />
            </NavStackItem>
          </template>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtChild />
    </div>
  </div>
</template>

<script>
import NavStack from '~/components/ui/NavStack'
import NavStackItem from '~/components/ui/NavStackItem'

import PaintbrushIcon from '~/assets/images/utils/paintbrush.svg?inline'
import UserIcon from '~/assets/images/utils/user.svg?inline'
import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import CurrencyIcon from '~/assets/images/utils/currency.svg?inline'

export default {
  name: 'Settings',
  components: {
    NavStack,
    NavStackItem,
    PaintbrushIcon,
    UserIcon,
    HeartIcon,
    CurrencyIcon,
  },
}
</script>

<style lang="scss" scoped></style>
