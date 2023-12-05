<template>
  <div class="universal-card">
    <h2>{{ formatMessage(messages.sessionsTitle) }}</h2>
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
              formatMessage(messages.sessionsLastAccessedAt, {
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
              formatMessage(messages.sessionsCreatedAt, {
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

definePageMeta({
  middleware: 'auth',
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const messages = defineMessages({
  currentSessionLabel: {
    id: 'settings.sessions.label.current-session',
    defaultMessage: 'Current session',
  },
  revokeSessionButton: {
    id: 'settings.sessions.button.revoke-session',
    defaultMessage: 'Revoke session',
  },
  sessionsCreatedAt: {
    id: 'settings.sessions.label.created-at',
    defaultMessage: 'Created {ago}',
  },
  sessionsDescription: {
    id: 'settings.sessions.description',
    defaultMessage:
      "Here are all the devices that are currently logged in with your Modrinth account. You can log out of each one individually.\n\nIf you see an entry you don't recognize, log out of that device and change your Modrinth account password immediately.",
  },
  sessionsLastAccessedAt: {
    id: 'settings.sessions.label.last-accessed-at',
    defaultMessage: 'Last accessed {ago}',
  },
  sessionsTitle: {
    id: 'settings.sessions.title',
    defaultMessage: 'Sessions',
  },
  unknownOsLabel: {
    id: 'settings.sessions.label.unknown-os',
    defaultMessage: 'Unknown OS',
  },
  unknownPlatformLabel: {
    id: 'settings.sessions.label.unknown-platform',
    defaultMessage: 'Unknown platform',
  },
})

useHead({
  title: () => `${formatMessage(messages.sessionsTitle)} - Modrinth`,
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
<style lang="scss" socped>
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
