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
} from '@modrinth/assets'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import Instance from '@/components/ui/Instance.vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import ProjectCard from '@/components/ui/ProjectCard.vue'
import { get_by_profile_path } from '@/helpers/process.js'
import { handleError } from '@/store/notifications.js'
import { duplicate, kill, remove, run } from '@/helpers/profile.js'
import { useRouter } from 'vue-router'
import { showProfileInFolder } from '@/helpers/utils.js'
import { trackEvent } from '@/helpers/analytics'
import { handleSevereError } from '@/store/error.js'
import { install as installVersion } from '@/store/install.js'
import { openUrl } from '@tauri-apps/plugin-opener'
import { HeadingLink } from '@modrinth/ui'

const router = useRouter()

const props = defineProps({
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

const actualInstances = computed(() =>
  props.instances.filter(
    (x) => (x && x.instances && x.instances[0] && x.show === undefined) || x.show,
  ),
)

const modsRow = ref(null)
const instanceOptions = ref(null)
const instanceComponents = ref(null)
const rows = ref(null)
const deleteConfirmModal = ref(null)

const currentDeleteInstance = ref(null)

async function deleteProfile() {
  if (currentDeleteInstance.value) {
    await remove(currentDeleteInstance.value).catch(handleError)
  }
}

async function duplicateProfile(p) {
  await duplicate(p).catch(handleError)
}

const handleInstanceRightClick = async (event, passedInstance) => {
  const baseOptions = [
    { name: 'add_content' },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'duplicate' },
    { name: 'open_folder' },
    { name: 'copy_path' },
    { type: 'divider' },
    {
      name: 'delete',
      color: 'danger',
    },
  ]

  const runningProcesses = await get_by_profile_path(passedInstance.path).catch(handleError)

  const options =
    runningProcesses.length > 0
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
      await run(args.item.path).catch((err) =>
        handleSevereError(err, { profilePath: args.item.path }),
      )
      trackEvent('InstanceStart', {
        loader: args.item.loader,
        game_version: args.item.game_version,
      })
      break
    case 'stop':
      await kill(args.item.path).catch(handleError)
      trackEvent('InstanceStop', {
        loader: args.item.loader,
        game_version: args.item.game_version,
      })
      break
    case 'add_content':
      await router.push({
        path: `/browse/${args.item.loader === 'vanilla' ? 'datapack' : 'mod'}`,
        query: { i: args.item.path },
      })
      break
    case 'edit':
      await router.push({
        path: `/instance/${encodeURIComponent(args.item.path)}/`,
      })
      break
    case 'duplicate':
      if (args.item.install_stage == 'installed') await duplicateProfile(args.item.path)
      break
    case 'delete':
      currentDeleteInstance.value = args.item.path
      deleteConfirmModal.value.show()
      break
    case 'open_folder':
      await showProfileInFolder(args.item.path)
      break
    case 'copy_path':
      await navigator.clipboard.writeText(args.item.path)
      break
    case 'install': {
      await installVersion(args.item.project_id, null, null, 'ProjectCardContextMenu')

      break
    }
    case 'open_link':
      openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
      break
    case 'copy_link':
      await navigator.clipboard.writeText(
        `https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
      )
      break
  }
}

const maxInstancesPerCompactRow = ref(1)
const maxInstancesPerRow = ref(1)
const maxProjectsPerRow = ref(1)

const calculateCardsPerRow = () => {
  if (rows.value.length === 0) {
    return
  }

  // Calculate how many cards fit in one row
  const containerWidth = rows.value[0].clientWidth
  // Convert container width from pixels to rem
  const containerWidthInRem =
    containerWidth / parseFloat(getComputedStyle(document.documentElement).fontSize)

  maxInstancesPerCompactRow.value = Math.floor((containerWidthInRem + 0.75) / 18.75)
  maxInstancesPerRow.value = Math.floor((containerWidthInRem + 0.75) / 20.75)
  maxProjectsPerRow.value = Math.floor((containerWidthInRem + 0.75) / 18.75)

  if (maxInstancesPerRow.value < 5) {
    maxInstancesPerRow.value *= 2
  }
  if (maxInstancesPerCompactRow.value < 5) {
    maxInstancesPerCompactRow.value *= 2
  }
  if (maxProjectsPerRow.value < 3) {
    maxProjectsPerRow.value *= 2
  }
}

const rowContainer = ref(null)
const resizeObserver = ref(null)

onMounted(() => {
  calculateCardsPerRow()
  resizeObserver.value = new ResizeObserver(calculateCardsPerRow)
  if (rowContainer.value) {
    resizeObserver.value.observe(rowContainer.value)
  }
  window.addEventListener('resize', calculateCardsPerRow)
})

onUnmounted(() => {
  window.removeEventListener('resize', calculateCardsPerRow)
  if (rowContainer.value) {
    resizeObserver.value.unobserve(rowContainer.value)
  }
})
</script>

<template>
  <ConfirmModalWrapper
    ref="deleteConfirmModal"
    title="Are you sure you want to delete this instance?"
    description="If you proceed, all data for your instance will be removed. You will not be able to recover it."
    :has-to-type="false"
    proceed-label="Delete"
    @proceed="deleteProfile"
  />
  <div ref="rowContainer" class="flex flex-col gap-4">
    <div v-for="row in actualInstances" ref="rows" :key="row.label" class="row">
      <HeadingLink class="mt-1" :to="row.route">
        {{ row.label }}
      </HeadingLink>
      <section
        v-if="row.instance"
        ref="modsRow"
        class="instances"
        :class="{ compact: row.compact }"
      >
        <Instance
          v-for="(instance, instanceIndex) in row.instances.slice(
            0,
            row.compact ? maxInstancesPerCompactRow : maxInstancesPerRow,
          )"
          :key="row.label + instance.path"
          :instance="instance"
          :compact="row.compact"
          :first="instanceIndex === 0"
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
    <template #duplicate> <ClipboardCopyIcon /> Duplicate instance</template>
    <template #copy_path> <ClipboardCopyIcon /> Copy path </template>
    <template #install> <DownloadIcon /> Install </template>
    <template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
    <template #copy_link> <ClipboardCopyIcon /> Copy link </template>
  </ContextMenu>
</template>
<style lang="scss" scoped>
.content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
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
      font-size: var(--font-size-md);
      font-weight: bolder;
      white-space: nowrap;
      color: var(--color-base);
    }

    svg {
      height: 1.25rem;
      width: 1.25rem;
      color: var(--color-base);
    }
  }

  .instances {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
    grid-gap: 0.75rem;
    width: 100%;

    &.compact {
      grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
      gap: 0.75rem;
    }
  }

  .projects {
    display: grid;
    width: 100%;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    grid-gap: 0.75rem;

    .item {
      width: 100%;
      max-width: 100%;
    }
  }
}
</style>
