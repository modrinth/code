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
  ChevronRightIcon
} from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import {onMounted, onUnmounted, ref} from 'vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import ProjectCard from "@/components/ui/ProjectCard.vue";

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

const modsRow = ref(null)
const instanceOptions = ref(null)
const instanceComponents = ref(null)
const rows = ref(null)

const handleInstanceRightClick = (event, passedInstance) => {
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

  const options = !passedInstance?.instance?.path
    ? [
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
      ]
    : passedInstance.playing
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

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'play':
      await args.item.play()
      break
    case 'stop':
      await args.item.stop()
      break
    case 'add_content':
      await args.item.addContent()
      break
    case 'edit':
      await args.item.seeInstance()
      break
    case 'delete':
      await args.item.deleteInstance()
      break
    case 'open_folder':
      await args.item.openFolder()
      break
    case 'copy_path':
      await navigator.clipboard.writeText(args.item.instance.path)
      break
    case 'install':
      args.item.install()
      break
    case 'open_link':
      window.__TAURI_INVOKE__('tauri', {
        __tauriModule: 'Shell',
        message: {
          cmd: 'open',
          path: `https://modrinth.com/${args.item.instance.project_type}/${args.item.instance.slug}`,
        },
      })
      break
    case 'copy_link':
      await navigator.clipboard.writeText(
        `https://modrinth.com/${args.item.instance.project_type}/${args.item.instance.slug}`
      )
      break
  }
}

const getInstanceIndex = (rowIndex, index) => {
  let instanceIndex = 0
  for (let i = 0; i < rowIndex; i++) {
    instanceIndex += props.instances[i].instances.length
  }
  instanceIndex += index
  return instanceIndex
}

const maxInstancesPerRow = ref(0);
const maxProjectsPerRow = ref(0);

const calculateCardsPerRow = () => {
  // Calculate how many cards fit in one row
  const containerWidth = rows.value[0].clientWidth;
  // Convert container width from pixels to rem
  const containerWidthInRem = containerWidth / parseFloat(getComputedStyle(document.documentElement).fontSize);
  maxInstancesPerRow.value = Math.floor((containerWidthInRem - 10) / 10);
  maxProjectsPerRow.value = Math.floor((containerWidthInRem - 5) / 18);
}

onMounted(() => {
  calculateCardsPerRow();
  window.addEventListener('resize', calculateCardsPerRow);
});

onUnmounted(() => {
  window.removeEventListener('resize', calculateCardsPerRow);
});
</script>

<template>
  <div class="content">
    <div v-for="(row, rowIndex) in instances" ref="rows" :key="row.label" class="row">
      <div class="header">
        <router-link :to="row.route">{{row.label}}</router-link>
        <ChevronRightIcon/>
      </div>
      <section v-if="row.instances[0].metadata" ref="modsRow" class="instances">
        <Instance
          v-for="(instance, instanceIndex) in row.instances.slice(0, maxInstancesPerRow)"
          ref="instanceComponents"
          :key="instance?.project_id || instance?.id"
          :instance="instance"
          @contextmenu.prevent.stop="
            (event) =>
              handleInstanceRightClick(
                event,
                instanceComponents[getInstanceIndex(rowIndex, instanceIndex)]
              )
          "
        />
      </section>
      <section v-else ref="modsRow" class="projects">
        <ProjectCard
          v-for="(project, projectIndex) in row.instances.slice(0, maxProjectsPerRow)"
          :key="project?.project_id"
          ref="instanceComponents"
          class="item"
          :project="project"
          @contextmenu.prevent.stop="
            (event) =>
              handleInstanceRightClick(
                event,
                instanceComponents[getInstanceIndex(rowIndex, projectIndex)]
              )
          "
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
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    grid-gap: 1rem;
    width: 100%;
  }

  .projects {
    display: grid;
    width: 100%;
    grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
    grid-gap: 1rem;

    .item {
      width: 100%;
      max-width: 100%;
    }
  }
}
</style>
