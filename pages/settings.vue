<template>
  <div>
    <div class="normal-page no-sidebar">
      <h1>{{ formatMessage(commonMessages.settingsLabel) }}</h1>
    </div>
    <div class="normal-page">
      <div class="normal-page__sidebar">
        <aside class="universal-card">
          <NavStack>
            <h3>Display</h3>
            <NavStackItem link="/settings" :label="formatMessage(messages.appearanceTitle)">
              <PaintBrushIcon />
            </NavStackItem>
            <NavStackItem
              v-if="isStaging"
              link="/settings/language"
              :label="formatMessage(messages.languageTitle)"
            >
              <LanguagesIcon />
            </NavStackItem>
            <template v-if="auth.user">
              <h3>Account</h3>
              <NavStackItem link="/settings/profile" :label="formatMessage(messages.profileTitle)">
                <UserIcon />
              </NavStackItem>
              <NavStackItem link="/settings/account" :label="formatMessage(messages.accountTitle)">
                <ShieldIcon />
              </NavStackItem>
              <NavStackItem
                link="/settings/authorizations"
                :label="formatMessage(messages.authorizedAppsTitle)"
              >
                <GridIcon />
              </NavStackItem>
              <NavStackItem
                link="/settings/sessions"
                :label="formatMessage(messages.sessionsTitle)"
              >
                <MonitorSmartphoneIcon />
              </NavStackItem>
            </template>
            <template v-if="auth.user">
              <h3>Developer</h3>
              <NavStackItem link="/settings/pats" :label="formatMessage(messages.patsTitle)">
                <KeyIcon />
              </NavStackItem>
              <NavStackItem
                link="/settings/applications"
                :label="formatMessage(messages.applicationsTitle)"
              >
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
  </div>
</template>
<script setup>
import {
  UsersIcon,
  ServerIcon,
  GridIcon,
  PaintBrushIcon,
  UserIcon,
  ShieldIcon,
  KeyIcon,
  LanguagesIcon,
} from 'omorphia'
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'
import MonitorSmartphoneIcon from '~/assets/images/utils/monitor-smartphone.svg'

import { commonMessages } from '~/utils/common-messages.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  appearanceTitle: {
    id: 'settings.appearance.title',
    defaultMessage: 'Appearance',
  },
  languageTitle: {
    id: 'settings.language.title',
    defaultMessage: 'Language',
  },
  profileTitle: {
    id: 'settings.profile.title',
    defaultMessage: 'Public profile',
  },
  accountTitle: {
    id: 'settings.account.title',
    defaultMessage: 'Account and security',
  },
  authorizedAppsTitle: {
    id: 'settings.authorized-apps.title',
    defaultMessage: 'Authorized apps',
  },
  sessionsTitle: {
    id: 'settings.sessions.title',
    defaultMessage: 'Sessions',
  },
  patsTitle: {
    id: 'settings.pats.title',
    defaultMessage: 'Personal access tokens',
  },
  applicationsTitle: {
    id: 'settings.applications.title',
    defaultMessage: 'Your applications',
  },
})

const route = useRoute()
const auth = await useAuth()
const isStaging = useRuntimeConfig().public.siteUrl !== 'https://modrinth.com'
</script>
