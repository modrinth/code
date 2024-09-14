<template>
  <div class="contents">
    <PyroLoading v-if="loading" class="h-full" />
    <div v-else-if="serverData" class="p-8 w-full min-h-full flex flex-col gap-4">
      <div class="flex flex-row items-center gap-6">
        <Avatar
          no-shadow
          size="lg"
          alt="Server Icon"
          style="width: 96px; height: 96px; min-width: 96px; min-height: 96px"
          :src="serverData?.project?.icon_url ?? undefined"
        />
        <div class="flex flex-col gap-4">
          <div class="-mb-2 flex shrink-0 flex-row items-center gap-1"></div>
          <div class="flex flex-row items-center gap-4">
            <h1 class="m-0 text-4xl font-bold text-[var(--color-contrast)]">
              {{ serverData.name }}
            </h1>
          </div>
          <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
            <ServerGameLabel
              v-if="showGameLabel"
              :game="serverData.game!"
              :mc-version="serverData.mc_version ?? ''"
            />
            <ServerLoaderLabel
              v-if="showLoaderLabel"
              :loader="serverData.loader!"
              :loader-version="serverData.loader_version ?? ''"
            />
            <ServerModLabel v-if="showModLabel" :mods="serverData.mods" />
          </div>
        </div>
      </div>

      <div class="flex w-full flex-col justify-between gap-4 md:flex-row md:items-center">
        <NavTabs :links="navLinks" />
      </div>

      <div data-pyro-mount class="h-full w-full">
        <RouterView :server="serverData" />
      </div>

      <PoweredByPyro />
    </div>

    <PyroError v-if="error" :title="errorTitle" :message="errorMessage" />
  </div>
</template>

<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { useBreadcrumbs } from '../../../store/breadcrumbs'
import { useServerStore } from '@/store/servers'
import { ref, onMounted, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { handleError } from '@/store/state'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { get_user } from '@/helpers/cache.js'
import PyroLoading from '@/components/ui/servers/PyroLoading.vue'
import { Avatar } from '@modrinth/ui'
import ServerGameLabel from '@/components/ui/servers/ServerGameLabel.vue'
import ServerLoaderLabel from '@/components/ui/servers/ServerLoaderLabel.vue'
import ServerModLabel from '@/components/ui/servers/ServerModLabel.vue'
import PoweredByPyro from '@/components/ui/servers/PoweredByPyro.vue'
import PyroError from '@/components/ui/servers/PyroError.vue'
import { PyroFetchError } from '@/helpers/pyroFetch'
import NavTabs from '@/components/ui/NavTabs.vue'

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

const route = useRoute()
const serverStore = useServerStore()
const breadcrumbs = useBreadcrumbs()

const serverId = computed(() => route.params.id as string)
const loading = ref(true)
const error = ref<Error | null>(null)
const errorTitle = ref('Error')
const errorMessage = ref('An unexpected error occurred.')
const credentials = ref<Credentials | null>(null)

const { serverData: storeServerData } = storeToRefs(serverStore)
const serverData = computed(() => storeServerData.value[serverId.value] || null)

const showGameLabel = computed(() => !!serverData.value?.game)
const showLoaderLabel = computed(() => !!serverData.value?.loader)
const showModLabel = computed(() => (serverData.value?.mods?.length ?? 0) > 0)

const navLinks = [
  { label: 'Overview', href: `/servers/manage/${serverId.value}` },
  { label: 'Content', href: `/servers/manage/${serverId.value}/content` },
  { label: 'Files', href: `/servers/manage/${serverId.value}/files` },
  { label: 'Backups', href: `/servers/manage/${serverId.value}/backups` },
  { label: 'Options', href: `/servers/manage/${serverId.value}/options` },
]

async function fetchCredentials() {
  try {
    const creds = await getCreds()
    if (creds && creds.user_id) {
      creds.user = await get_user(creds.user_id)
    }
    credentials.value = creds
  } catch (err) {
    handleError(err)
    error.value = err as Error
    errorTitle.value = 'Authentication Error'
    errorMessage.value = 'Failed to fetch Modrinth session. Are you logged in?'
  }
}

async function fetchServerDataOnMount() {
  if (!credentials.value) {
    error.value = new Error('No credentials')
    errorTitle.value = 'Authentication Error'
    errorMessage.value = 'Could not retrieve Modrinth session. Try logging in again.'
    loading.value = false
    return
  }

  const session = credentials.value.session || null

  if (!session) {
    error.value = new Error('No session')
    errorTitle.value = 'Authentication Error'
    errorMessage.value = 'No session found in credentials.'
    loading.value = false
    return
  }

  try {
    await serverStore.fetchServerData(session, serverId.value)
    if (serverData.value) {
      await breadcrumbs.setName('Server', serverData.value.name)
    }
  } catch (err) {
    console.error('Error fetching server data:', err)
    error.value = err as Error
    if (err instanceof PyroFetchError) {
      switch (err.statusCode) {
        case 400:
          errorTitle.value = 'Invalid Request'
          errorMessage.value = 'Request was malformed.'
          break
        case 401:
        case 404:
          errorTitle.value = 'Server Not Found'
          errorMessage.value = 'The server you are looking for does not exist.'
          break
        default:
          errorTitle.value = 'Error'
          errorMessage.value = `An error occurred: ${err.message}`
      }
    } else {
      errorTitle.value = 'Unexpected Error'
      errorMessage.value = 'An unexpected error occurred while fetching server data.'
    }
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await fetchCredentials()
  if (!error.value) {
    await fetchServerDataOnMount()
  }
})
</script>
