<template>
  <div class="instance-container">
    <div class="side-cards pb-4" @scroll="$refs.promo.scroll()">
      <Card class="instance-card" @contextmenu.prevent.stop="handleRightClick">
        <Avatar size="md" :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null" />
        <div class="instance-info">
          <h2 class="name">{{ instance.name }}</h2>
          <span class="metadata"> {{ instance.loader }} {{ instance.game_version }} </span>
        </div>
        <span class="button-group">
          <Button v-if="instance.install_stage !== 'installed'" disabled class="instance-button">
            Installing...
          </Button>
          <Button
            v-else-if="playing === true"
            color="danger"
            class="instance-button"
            @click="stopInstance('InstancePage')"
          >
            <StopCircleIcon />
            Stop
          </Button>
          <Button
            v-else-if="playing === false && loading === false"
            color="primary"
            class="instance-button"
            @click="startInstance('InstancePage')"
          >
            <PlayIcon />
            Play
          </Button>
          <Button
            v-else-if="loading === true && playing === false"
            disabled
            class="instance-button"
          >
            Loading...
          </Button>
          <Button
            v-tooltip="'Open instance folder'"
            class="instance-button"
            @click="showProfileInFolder(instance.path)"
          >
            <FolderOpenIcon />
            Folder
          </Button>
        </span>
        <hr class="card-divider" />
        <div class="pages-list">
          <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/`" class="btn">
            <BoxIcon />
            Content
          </RouterLink>
          <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/logs`" class="btn">
            <FileIcon />
            Logs
          </RouterLink>
          <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/options`" class="btn">
            <SettingsIcon />
            Options
          </RouterLink>
        </div>
      </Card>
      <PromotionWrapper ref="promo" class="mt-4" />
    </div>
    <div class="content">
      <RouterView v-slot="{ Component }">
        <template v-if="Component">
          <Suspense @pending="loadingBar.startLoading()" @resolve="loadingBar.stopLoading()">
            <component
              :is="Component"
              :instance="instance"
              :options="options"
              :offline="offline"
              :playing="playing"
              :versions="modrinthVersions"
              :installed="instance.install_stage !== 'installed'"
            ></component>
          </Suspense>
        </template>
      </RouterView>
    </div>
  </div>
  <ContextMenu ref="options" @option-clicked="handleOptionsClick">
    <template #play> <PlayIcon /> Play </template>
    <template #stop> <StopCircleIcon /> Stop </template>
    <template #add_content> <PlusIcon /> Add Content </template>
    <template #edit> <EditIcon /> Edit </template>
    <template #copy_path> <ClipboardCopyIcon /> Copy Path </template>
    <template #open_folder> <ClipboardCopyIcon /> Open Folder </template>
    <template #copy_link> <ClipboardCopyIcon /> Copy Link </template>
    <template #open_link> <ClipboardCopyIcon /> Open In Modrinth <ExternalIcon /> </template>
    <template #copy_names><EditIcon />Copy names</template>
    <template #copy_slugs><HashIcon />Copy slugs</template>
    <template #copy_links><GlobeIcon />Copy Links</template>
    <template #toggle><EditIcon />Toggle selected</template>
    <template #disable><XIcon />Disable selected</template>
    <template #enable><CheckCircleIcon />Enable selected</template>
    <template #hide_show><EyeIcon />Show/Hide unselected</template>
    <template #update_all
      ><UpdatedIcon />Update {{ selected.length > 0 ? 'selected' : 'all' }}</template
    >
    <template #filter_update><UpdatedIcon />Select Updatable</template>
  </ContextMenu>
</template>
<script setup>
import { Button, Avatar, Card } from '@modrinth/ui'
import {
  BoxIcon,
  SettingsIcon,
  FileIcon,
  PlayIcon,
  StopCircleIcon,
  EditIcon,
  FolderOpenIcon,
  ClipboardCopyIcon,
  PlusIcon,
  ExternalIcon,
  HashIcon,
  GlobeIcon,
  EyeIcon,
  XIcon,
  CheckCircleIcon,
  UpdatedIcon,
} from '@modrinth/assets'
import { get, kill, run } from '@/helpers/profile'
import { get_by_profile_path } from '@/helpers/process'
import { process_listener, profile_listener } from '@/helpers/events'
import { useRoute, useRouter } from 'vue-router'
import { ref, onUnmounted } from 'vue'
import { handleError, useBreadcrumbs, useLoading } from '@/store/state'
import { showProfileInFolder } from '@/helpers/utils.js'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { trackEvent } from '@/helpers/analytics'
import { convertFileSrc } from '@tauri-apps/api/core'
import { handleSevereError } from '@/store/error.js'
import { get_project, get_version_many } from '@/helpers/cache.js'
import dayjs from 'dayjs'
import PromotionWrapper from '@/components/ui/PromotionWrapper.vue'

const route = useRoute()

const router = useRouter()
const breadcrumbs = useBreadcrumbs()

const instance = ref(await get(route.params.id).catch(handleError))

breadcrumbs.setName(
  'Instance',
  instance.value.name.length > 40
    ? instance.value.name.substring(0, 40) + '...'
    : instance.value.name,
)

breadcrumbs.setContext({
  name: instance.value.name,
  link: route.path,
  query: route.query,
})

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const loadingBar = useLoading()

const playing = ref(false)
const loading = ref(false)
const options = ref(null)

const startInstance = async (context) => {
  loading.value = true
  try {
    await run(route.params.id)
    playing.value = true
  } catch (err) {
    handleSevereError(err, { profilePath: route.params.id })
  }
  loading.value = false

  trackEvent('InstanceStart', {
    loader: instance.value.loader,
    game_version: instance.value.game_version,
    source: context,
  })
}

const checkProcess = async () => {
  const runningProcesses = await get_by_profile_path(route.params.id).catch(handleError)

  playing.value = runningProcesses.length > 0
}

// Get information on associated modrinth versions, if any
const modrinthVersions = ref([])
if (!offline.value && instance.value.linked_data && instance.value.linked_data.project_id) {
  get_project(instance.value.linked_data.project_id, 'must_revalidate')
    .catch(handleError)
    .then((project) => {
      if (project && project.versions) {
        get_version_many(project.versions, 'must_revalidate')
          .catch(handleError)
          .then((versions) => {
            modrinthVersions.value = versions.sort(
              (a, b) => dayjs(b.date_published) - dayjs(a.date_published),
            )
          })
      }
    })
}

await checkProcess()

const stopInstance = async (context) => {
  playing.value = false
  await kill(route.params.id).catch(handleError)

  trackEvent('InstanceStop', {
    loader: instance.value.loader,
    game_version: instance.value.game_version,
    source: context,
  })
}

const handleRightClick = (event) => {
  const baseOptions = [
    { name: 'add_content' },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'open_folder' },
    { name: 'copy_path' },
  ]

  options.value.showMenu(
    event,
    instance.value,
    playing.value
      ? [
          {
            name: 'stop',
            color: 'danger',
          },
          ...baseOptions,
        ]
      : [
          {
            name: 'play',
            color: 'primary',
          },
          ...baseOptions,
        ],
  )
}

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'play':
      await startInstance('InstancePageContextMenu')
      break
    case 'stop':
      await stopInstance('InstancePageContextMenu')
      break
    case 'add_content':
      await router.push({
        path: `/browse/${instance.value.loader === 'vanilla' ? 'datapack' : 'mod'}`,
        query: { i: route.params.id },
      })
      break
    case 'edit':
      await router.push({
        path: `/instance/${encodeURIComponent(route.params.id)}/options`,
      })
      break
    case 'open_folder':
      await showProfileInFolder(instance.value.path)
      break
    case 'copy_path':
      await navigator.clipboard.writeText(instance.value.path)
      break
  }
}

const unlistenProfiles = await profile_listener(async (event) => {
  if (event.profile_path_id === route.params.id) {
    if (event.event === 'removed') {
      await router.push({
        path: '/',
      })
      return
    }
    instance.value = await get(route.params.id).catch(handleError)
  }
})

const unlistenProcesses = await process_listener((e) => {
  if (e.event === 'finished' && e.profile_path_id === route.params.id) playing.value = false
})

onUnmounted(() => {
  unlistenProcesses()
  unlistenProfiles()
})
</script>

<style scoped lang="scss">
.instance-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

Button {
  width: 100%;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.side-cards {
  position: fixed;
  width: 300px;
  display: flex;
  flex-direction: column;

  min-height: calc(100vh - 3.25rem);
  max-height: calc(100vh - 3.25rem);
  overflow-y: auto;
  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }

  .card {
    min-height: unset;
    margin-bottom: 0;
  }
}

.instance-nav {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 1rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  height: 100%;
}

.name {
  font-size: 1.25rem;
  color: var(--color-contrast);
  overflow: hidden;
  text-overflow: ellipsis;
}

.metadata {
  text-transform: capitalize;
}

.instance-container {
  display: flex;
  flex-direction: row;
  overflow: auto;
  gap: 1rem;
  min-height: 100%;
  padding: 1rem;
}

.instance-info {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.badge {
  display: flex;
  align-items: center;
  font-weight: bold;
  width: fit-content;
  color: var(--color-orange);
}

.pages-list {
  display: flex;
  flex-direction: column;
  gap: var(--gap-xs);

  .btn {
    font-size: 100%;
    font-weight: 400;
    background: inherit;
    transition: all ease-in-out 0.1s;
    width: 100%;
    color: var(--color-primary);
    box-shadow: none;

    &.router-link-exact-active {
      box-shadow: var(--shadow-inset-lg);
      background: var(--color-button-bg);
      color: var(--color-contrast);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: var(--shadow-inset-lg);
      text-decoration: none;
    }

    svg {
      width: 1.3rem;
      height: 1.3rem;
    }
  }
}

.instance-nav {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  justify-content: left;
  padding: 1rem;
  gap: 0.5rem;
  height: min-content;
  width: 100%;
}

.instance-button {
  width: fit-content;
}

.actions {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: 0.5rem;
}

.content {
  margin: 0 1rem 0.5rem 20rem;
  width: calc(100% - 20rem);
  display: flex;
  flex-direction: column;
  overflow: auto;
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }

  @media screen and (max-width: 750px) {
    flex-direction: row;
    column-gap: var(--gap-md);
    margin-top: var(--gap-xs);
  }

  @media screen and (max-width: 600px) {
    margin-top: 0;

    .stat-label {
      display: none;
    }
  }
}
</style>
