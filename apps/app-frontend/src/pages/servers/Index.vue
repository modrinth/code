<template>
  <div data-pyro-mount class="w-full min-h-full p-8">
    <div class="flex justify-between items-center mb-4">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search servers..."
        class="px-3 py-2 w-full border rounded-md"
      />
    </div>
    <template v-if="!loading">
      <ul v-if="filteredServers.length > 0" class="m-0 flex flex-col gap-4 p-0">
        <ServerListing
          v-for="server in filteredServers"
          :key="server.server_id"
          :server_id="server.server_id"
          :name="server.name"
          :state="server.state"
          :game="server.game"
          :loader="server.loader"
          :loader_version="server.loader_version"
          :mc_version="server.mc_version"
          :modpack="server.modpack"
        />
      </ul>
      <div
        v-else-if="searchQuery && servers.length > 0"
        class="flex h-full items-center justify-center"
      >
        <p class="text-contrast">No servers found.</p>
      </div>
      <div
        v-else-if="servers.length === 0"
        class="flex h-full min-h-[128px] items-center justify-center"
      >
        <p class="text-contrast">You don't have any servers yet.</p>
      </div>
    </template>
    <div v-else-if="error">
      <img
        alt=""
        class="h-full w-full max-w-24 rounded-2xl object-cover align-middle"
        height="256"
        src="@/assets/images/servers/this-is-fine.gif"
        width="256"
        loading="lazy"
        decoding="async"
      />
      <div class="leading-[165%]">
        <h1 class="m-0 mb-2 text-2xl font-semibold">Unable to load servers</h1>
        <p class="m-0 max-w-2xl">
          Your servers are safe, but could not be loaded due to a technical issue on our end. Please
          try again later. If this issue persists, please contact
          <a
            class="text-[var(--color-link)]"
            href="https://support.modrinth.com/"
            rel="noopener noreferrer"
            target="_blank"
          >
            Modrinth support.
          </a>
        </p>
      </div>
    </div>
    <PyroLoading v-else class="h-full" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { handleError } from '@/store/state'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { get_user } from '@/helpers/cache.js'
import { useServerStore } from '@/store/servers'
import type { Server } from '@/types/servers'
import PyroLoading from '@/components/ui/servers/PyroLoading.vue'
import ServerListing from '@/components/ui/servers/ServerListing.vue'

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
const searchQuery = ref('')

const serverStore = useServerStore()

const filteredServers = computed(() => {
  if (!searchQuery.value) return servers.value
  const query = searchQuery.value.toLowerCase()
  return servers.value.filter((server) => server.name.toLowerCase().includes(query))
})

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
