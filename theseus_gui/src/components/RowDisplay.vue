<script setup>
import {
  ClipboardCopyIcon,
  FolderOpenIcon,
  PlayIcon,
  PlusIcon,
  TrashIcon,
  DownloadIcon,
  GlobeIcon,
  StopCircleIcon,
  ExternalIcon,
  EyeIcon,
  ChevronRightIcon,
} from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import { onMounted, onUnmounted, ref } from 'vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import ProjectCard from '@/components/ui/ProjectCard.vue'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import InstanceInstallModal from '@/components/ui/InstanceInstallModal.vue'
import {
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
  kill_by_uuid,
} from '@/helpers/process.js'
import { handleError } from '@/store/notifications.js'
import { remove, run } from '@/helpers/profile.js'
import { useRouter } from 'vue-router'
import { showInFolder } from '@/helpers/utils.js'
import { useFetch } from '@/helpers/fetch.js'
import { install as pack_install } from '@/helpers/pack.js'

const router = useRouter()

defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
  canPaginate: Boolean,
})

const modsRow = ref(null)
const instanceOptions = ref(null)
const instanceComponents = ref(null)
const rows = ref(null)
const confirmModal = ref(null)
const modInstallModal = ref(null)

const handleInstanceRightClick = async (event, passedInstance) => {
  const baseOptions = [
    { name: 'add_content' },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'open_folder' },
    { name: 'copy_path' },
    { type: 'divider' },
    {
      name: 'delete',
      color: 'danger',
    },
  ]

  const running = await get_all_running_profile_paths().catch(handleError)

  const options = running.includes(passedInstance.path)
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
      ]

  instanceOptions.value.showMenu(event, passedInstance, options)
}

const handleProjectClick = (event, passedInstance) => {
  instanceOptions.value.showMenu(event, passedInstance, [
    {
      name: 'install',
      color: 'primary',
    },
    { type: 'divider' },
    {
      name: 'open_link',
    },
    {
      name: 'copy_link',
    },
  ])
}

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'play':
      await run(args.item.path).catch(handleError)
      break
    case 'stop':
      for (const u of await get_uuids_by_profile_path(args.item.path).catch(handleError)) {
        await kill_by_uuid(u).catch(handleError)
      }
      break
    case 'add_content':
      await router.push({
        path: `/browse/${args.item.metadata.loader === 'vanilla' ? 'datapack' : 'mod'}`,
        query: { i: args.item.path },
      })
      break
    case 'edit':
      await router.push({
        path: `/instance/${encodeURIComponent(args.item.path)}/`,
      })
      break
    case 'delete':
      await remove(args.item.path).catch(handleError)
      break
    case 'open_folder':
      await showInFolder(args.item.path)
      break
    case 'copy_path':
      await navigator.clipboard.writeText(args.item.path)
      break
    case 'install': {
      const versions = await useFetch(
        `https://api.modrinth.com/v2/project/${args.item.project_id}/version`,
        'project versions'
      )

      if (args.item.project_type === 'modpack') {
        await pack_install(
          args.item.project_id,
          versions[0].id,
          args.item.title,
          args.item.icon_url
        )
      } else {
        modInstallModal.value.show(args.item.project_id, versions)
      }
      break
    }
    case 'open_link':
      window.__TAURI_INVOKE__('tauri', {
        __tauriModule: 'Shell',
        message: {
          cmd: 'open',
          path: `https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
        },
      })
      break
    case 'copy_link':
      await navigator.clipboard.writeText(
        `https://modrinth.com/${args.item.project_type}/${args.item.slug}`
      )
      break
  }
}

const maxInstancesPerRow = ref(0)
const maxProjectsPerRow = ref(0)

const calculateCardsPerRow = () => {
  // Calculate how many cards fit in one row
  const containerWidth = rows.value[0].clientWidth
  // Convert container width from pixels to rem
  const containerWidthInRem =
    containerWidth / parseFloat(getComputedStyle(document.documentElement).fontSize)
  maxInstancesPerRow.value = Math.floor((containerWidthInRem + 1) / 11)
  maxProjectsPerRow.value = Math.floor((containerWidthInRem + 1) / 19)
}

onMounted(() => {
  calculateCardsPerRow()
  window.addEventListener('resize', calculateCardsPerRow)
})

onUnmounted(() => {
  window.removeEventListener('resize', calculateCardsPerRow)
})
</script>

<template>
  <div class="content">
    <div v-for="row in instances" ref="rows" :key="row.label" class="row">
      <div class="header">
        <router-link :to="row.route">{{ row.label }}</router-link>
        <ChevronRightIcon />
      </div>
      <section v-if="row.instances[0].metadata" ref="modsRow" class="instances">
        <Instance
          v-for="instance in row.instances.slice(0, maxInstancesPerRow)"
          :key="instance?.project_id || instance?.id"
          :instance="instance"
          @contextmenu.prevent.stop="(event) => handleInstanceRightClick(event, instance)"
        />
      </section>
      <section v-else ref="modsRow" class="projects">
        <ProjectCard
          v-for="project in row.instances.slice(0, maxProjectsPerRow)"
          :key="project?.project_id"
          ref="instanceComponents"
          class="item"
          :project="project"
          :confirm-modal="confirmModal"
          :mod-install-modal="modInstallModal"
          @contextmenu.prevent.stop="(event) => handleProjectClick(event, project)"
        />
      </section>
    </div>
  </div>
  <ContextMenu ref="instanceOptions" @option-clicked="handleOptionsClick">
    <template #play> <PlayIcon /> Play </template>
    <template #stop> <StopCircleIcon /> Stop </template>
    <template #add_content> <PlusIcon /> Add content </template>
    <template #edit> <EyeIcon /> View instance </template>
    <template #delete> <TrashIcon /> Delete </template>
    <template #open_folder> <FolderOpenIcon /> Open folder </template>
    <template #copy_path> <ClipboardCopyIcon /> Copy path </template>
    <template #install> <DownloadIcon /> Install </template>
    <template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
    <template #copy_link> <ClipboardCopyIcon /> Copy link </template>
  </ContextMenu>
  <InstallConfirmModal ref="confirmModal" />
  <InstanceInstallModal ref="modInstallModal" />
</template>
<style lang="scss" scoped>
.content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 1rem;
  gap: 1rem;

  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }
}

.row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  overflow: hidden;
  width: 100%;
  min-width: 100%;

  &:nth-child(even) {
    background: var(--color-bg);
  }

  .header {
    width: 100%;
    margin-bottom: 1rem;
    gap: var(--gap-xs);
    display: flex;
    flex-direction: row;
    align-items: center;

    a {
      margin: 0;
      font-size: var(--font-size-lg);
      font-weight: bolder;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    svg {
      height: 1.5rem;
      width: 1.5rem;
      color: var(--color-contrast);
    }
  }

  .instances {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    grid-gap: 1rem;
    width: 100%;
  }

  .projects {
    display: grid;
    width: 100%;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    grid-gap: 1rem;

    .item {
      width: 100%;
      max-width: 100%;
    }
  }
}
</style>
