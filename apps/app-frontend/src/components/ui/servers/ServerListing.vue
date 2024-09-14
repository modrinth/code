<template>
  <RouterLink
    role="link"
    :aria-disabled="status.isInstalling || status.isFailed"
    :class="status.isInstalling || status.isFailed ? 'pointer-events-none cursor-not-allowed' : ''"
    class="flex flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4"
    data-pyro-server-listing
    :data-pyro-server-listing-id="server_id"
    :to="`/servers/manage/${server_id}`"
  >
    <Avatar :src="undefined" no-shadow size="md" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-2.5">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold text-[var(--color-contrast)]">{{ name }}</h2>
        <ServerInstallStatusPill v-if="status.state" :state="status.state" />
        <ChevronRightIcon v-if="!status.isInstalling && !status.isFailed" />
      </div>

      <div
        v-if="projectData?.title"
        class="m-0 flex flex-row items-center gap-1 text-sm font-medium text-[var(--color-text-secondary)]"
      >
        <Avatar
          :src="iconUrl"
          no-shadow
          style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
          alt="Server Icon"
        />
        Using {{ projectData?.title || 'Unknown' }}
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <ServerGameLabel v-if="showGameLabel" :game="game!" :mc-version="mc_version ?? ''" />
        <ServerLoaderLabel
          v-if="showLoaderLabel"
          :loader="loader!"
          :loader-version="loader_version ?? ''"
        />
        <ServerModLabel v-if="showModLabel" :mods="mods || []" />
      </div>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ChevronRightIcon } from '@modrinth/assets'
import type { StatusState } from '@/components/ui/servers/ServerInstallStatusPill.vue'
// i fucking hate whoever set up typescript in this god forsaken project
import type { Project, Server } from '../../../types/servers'
import { RouterLink } from 'vue-router'
import ServerModLabel from './ServerModLabel.vue'
import ServerLoaderLabel from './ServerLoaderLabel.vue'
import ServerGameLabel from './ServerGameLabel.vue'
import { Avatar } from '@modrinth/ui'
import { get_project, get_version } from '@/helpers/cache'
import ServerInstallStatusPill from '@/components/ui/servers/ServerInstallStatusPill.vue'
import { handleError } from '@/store/notifications'

const props = defineProps<Partial<Server>>()

const status = computed(() => ({
  state: props.state as StatusState | undefined,
  isFailed: props.state === 'Failed',
  isInstalling: props.state === 'Installing',
}))

const showGameLabel = computed(() => !!props.game)
const showLoaderLabel = computed(() => !!props.loader)
const showModLabel = computed(() => (props.mods?.length ?? 0) > 0)

const projectData = ref<Project | undefined>(undefined)

const fetchProjectData = async () => {
  if (props.modpack) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const versionData: any = await get_version(props.modpack, 'must_revalidate').catch(handleError)
    if (versionData && versionData.project_id) {
      projectData.value = await get_project(versionData.project_id, 'must_revalidate').catch(
        handleError,
      )
    }
  }
}

const iconUrl = computed(() => projectData.value?.icon_url || undefined)

onMounted(async () => {
  await fetchProjectData()
})
</script>
