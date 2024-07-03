<template>
  <div class="universal-card">
    <h2>{{ formatMessage(commonSettingsMessages.sessions) }}</h2>
    <p class="preserve-lines">
      {{ formatMessage(messages.sessionsDescription) }}
    </p>
    <div v-for="session in sessions" :key="session.id" class="universal-card recessed session">
      <div>
        <div>
          <strong>
            {{ session.os ?? formatMessage(messages.unknownOsLabel) }} ⋅
            {{ session.platform ?? formatMessage(messages.unknownPlatformLabel) }} ⋅
            {{ session.ip }}
          </strong>
        </div>
        <div>
          <template v-if="session.city">{{ session.city }}, {{ session.country }} ⋅ </template>
          <span
            v-tooltip="
              formatMessage(commonMessages.dateAtTimeTooltip, {
                date: new Date(session.last_login),
                time: new Date(session.last_login),
              })
            "
          >
            {{
              formatMessage(messages.lastAccessedAgoLabel, {
                ago: formatRelativeTime(session.last_login),
              })
            }}
          </span>
          ⋅
          <span
            v-tooltip="
              formatMessage(commonMessages.dateAtTimeTooltip, {
                date: new Date(session.created),
                time: new Date(session.created),
              })
            "
          >
            {{
              formatMessage(messages.createdAgoLabel, {
                ago: formatRelativeTime(session.created),
              })
            }}
          </span>
        </div>
      </div>
      <div class="input-group">
        <i v-if="session.current">{{ formatMessage(messages.currentSessionLabel) }}</i>
        <button v-else class="iconified-button raised-button" @click="revokeSession(session.id)">
          <XIcon /> {{ formatMessage(messages.revokeSessionButton) }}
        </button>
      </div>
    </div>
  </div>
</template>
<script setup>
import { XIcon } from 'omorphia'
import { commonSettingsMessages } from '~/utils/common-messages.ts'

definePageMeta({
  middleware: 'auth',
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const messages = defineMessages({
  currentSessionLabel: {
    id: 'settings.sessions.current-session',
    defaultMessage: 'Current session',
  },
  revokeSessionButton: {
    id: 'settings.sessions.action.revoke-session',
    defaultMessage: 'Revoke session',
  },
  createdAgoLabel: {
    id: 'settings.sessions.created-ago',
    defaultMessage: 'Created {ago}',
  },
  sessionsDescription: {
    id: 'settings.sessions.description',
    defaultMessage:
      "Here are all the devices that are currently logged in with your Modrinth account. You can log out of each one individually.\n\nIf you see an entry you don't recognize, log out of that device and change your Modrinth account password immediately.",
  },
  lastAccessedAgoLabel: {
    id: 'settings.sessions.last-accessed-ago',
    defaultMessage: 'Last accessed {ago}',
  },
  unknownOsLabel: {
    id: 'settings.sessions.unknown-os',
    defaultMessage: 'Unknown OS',
  },
  unknownPlatformLabel: {
    id: 'settings.sessions.unknown-platform',
    defaultMessage: 'Unknown platform',
  },
})

useHead({
  title: () => `${formatMessage(commonSettingsMessages.sessions)} - Modrinth`,
})

const data = useNuxtApp()
const { data: sessions, refresh } = await useAsyncData('session/list', () =>
  useBaseFetch('session/list')
)

async function revokeSession(id) {
  startLoading()
  try {
    sessions.value = sessions.value.filter((x) => x.id !== id)
    await useBaseFetch(`session/${id}`, {
      method: 'DELETE',
    })
    await refresh()
  } catch (err) {
    data.$notify({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}
</script>
<style lang="scss" scoped>
.session {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  @media screen and (min-width: 800px) {
    flex-direction: row;
    align-items: center;

    .input-group {
      margin-left: auto;
    }
  }
}
</style>
