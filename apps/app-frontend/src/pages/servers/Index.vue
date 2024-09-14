<template>
  <div class="w-full h-full p-8">
    <h1>Servers</h1>
    <div v-if="error">
      <p>Your servers couldn't be retrieved at this time. {{ error }}</p>
    </div>
    <ul v-else-if="servers.length">
      <li v-for="server in servers" :key="server.server_id">
        {{ server.name }}
      </li>
    </ul>
    <div v-else-if="loading">
      <PyroLoading />
    </div>
    <p v-else>No servers found.</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { handleError } from '@/store/state'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { get_user } from '@/helpers/cache.js'
import { useServerStore } from '@/store/servers'
import type { Server } from '@/types/servers'
import PyroLoading from '@/components/ui/servers/PyroLoading.vue'

type Credentials = {
  session: string
  expires: Date
  user_id: string
  active: boolean
  user: {
    id: string
    username: string
    avatar_url: string
    bio: string
    created: Date
    role: string
    badges: number
  }
}

const credentials = ref<Credentials | null>(null)
const servers = ref<Server[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

const serverStore = useServerStore()

async function fetchCredentials() {
  try {
    const creds = await getCreds()
    if (creds && creds.user_id) {
      creds.user = await get_user(creds.user_id)
    }
    credentials.value = creds
  } catch (err) {
    handleError(err)
    error.value = 'Failed to fetch Modrinth session. Are you logged in???'
  }
}

async function fetchServers() {
  if (!credentials.value) {
    error.value = 'Could not retrieve Modrinth session. Try logging in again.'
    loading.value = false
    return
  }

  const session = credentials.value.session || null

  if (!session) {
    error.value = 'No session found in credentials. Shit is FUCKED'
    loading.value = false
    return
  }

  try {
    const result = await serverStore.listServers(session)
    if (Array.isArray(result.servers)) {
      servers.value = result.servers
    } else {
      console.error('Server store must return array type:', result)
      servers.value = []
    }
  } catch (err) {
    console.error('Error fetching servers:', err)
    error.value = 'Failed to fetch servers'
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await fetchCredentials()
  if (!error.value) {
    await fetchServers()
  }
})
</script>
