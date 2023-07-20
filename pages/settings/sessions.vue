<template>
  <div class="universal-card">
    <h2>Sessions</h2>
    <p>
      Here are all the devices that are currently logged in with your Modrinth account. You can log
      out of each one individually.
      <br /><br />
      If you see an entry you don't recognize, log out of that device and change your Modrinth
      account password immediately.
    </p>
    <div v-for="session in sessions" :key="session.id" class="universal-card recessed session">
      <div>
        <div>
          <strong>
            {{ session.os ?? 'Unknown OS' }} ⋅ {{ session.platform ?? 'Unknown platform' }} ⋅
            {{ session.ip }}
          </strong>
        </div>
        <div>
          <template v-if="session.city">{{ session.city }}, {{ session.country }} ⋅</template>
          <span v-tooltip="$dayjs(session.last_login).format('MMMM D, YYYY [at] h:mm A')">
            Last accessed {{ fromNow(session.last_login) }}
          </span>
          ⋅
          <span v-tooltip="$dayjs(session.created).format('MMMM D, YYYY [at] h:mm A')">
            Created {{ fromNow(session.created) }}
          </span>
        </div>
      </div>
      <div class="input-group">
        <i v-if="session.current">Current session</i>
        <button v-else class="iconified-button raised-button" @click="revokeSession(session.id)">
          <XIcon /> Revoke session
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

useHead({
  title: 'Sessions - Modrinth',
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
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}
</script>
<style lang="scss">
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
