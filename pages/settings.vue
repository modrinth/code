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
            <NavStackItem link="/settings/monetization" label="Monetization">
              <CurrencyIcon />
            </NavStackItem>
          </template>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>
<script setup>
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import PaintbrushIcon from '~/assets/images/utils/paintbrush.svg'
import UserIcon from '~/assets/images/utils/user.svg'
import CurrencyIcon from '~/assets/images/utils/currency.svg'

const route = useRoute()
</script>

<style lang="scss" scoped></style>
