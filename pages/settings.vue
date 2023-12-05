<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Settings</h1>
        <NavStack>
          <NavStackItem link="/settings" label="Appearance">
            <PaintbrushIcon />
          </NavStackItem>
          <NavStackItem v-if="isStaging" link="/settings/language" label="Language">
            <LanguagesIcon />
          </NavStackItem>
          <template v-if="auth.user">
            <h3>User settings</h3>
            <NavStackItem link="/settings/account" label="Account">
              <UserIcon />
            </NavStackItem>
            <NavStackItem link="/settings/authorizations" label="Authorizations">
              <UsersIcon />
            </NavStackItem>
            <NavStackItem link="/settings/sessions" :label="formatMessage(messages.sessionsTitle)">
              <ShieldIcon />
            </NavStackItem>
          </template>
          <template v-if="auth.user">
            <h3>Developer Settings</h3>
            <NavStackItem link="/settings/pats" label="PATs">
              <KeyIcon />
            </NavStackItem>
            <NavStackItem link="/settings/applications" label="Applications">
              <ServerIcon />
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
import { UsersIcon, ServerIcon } from 'omorphia'
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import PaintbrushIcon from '~/assets/images/utils/paintbrush.svg'
import UserIcon from '~/assets/images/utils/user.svg'
import ShieldIcon from '~/assets/images/utils/shield.svg'
import KeyIcon from '~/assets/images/utils/key.svg'
import LanguagesIcon from '~/assets/images/utils/languages.svg'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  sessionsTitle: {
    id: 'settings.sessions.title',
    defaultMessage: 'Sessions',
  },
})

const route = useRoute()
const auth = await useAuth()
const isStaging = useRuntimeConfig().public.siteUrl !== 'https://modrinth.com'
</script>
