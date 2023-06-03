<script setup>
import {
  ChevronLeftIcon,
  ChevronRightIcon,
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
} from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import { onMounted, onUnmounted, ref } from 'vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'

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

const allowPagination = ref(Array.apply(false, Array(props.instances.length)))
const modsRow = ref(null)
const instanceOptions = ref(null)
const instanceComponents = ref(null)

const handlePaginationDisplay = () => {
  for (let i = 0; i < props.instances.length; i++) {
    let parentsRow = modsRow.value[i]

    // This is wrapped in a setTimeout because the HtmlCollection seems to struggle
    // with getting populated sometimes. It's a flaky error, but providing a bit of
    // wait-time for the below expressions has not failed thus-far.
    setTimeout(() => {
      const children = parentsRow.children
      const lastChild = children[children.length - 1]
      const childBox = lastChild?.getBoundingClientRect()

      if (childBox?.x + childBox?.width > window.innerWidth && props.canPaginate)
        allowPagination.value[i] = true
      else allowPagination.value[i] = false
    }, 300)
  }
}

onMounted(() => {
  if (props.canPaginate) window.addEventListener('resize', handlePaginationDisplay)
  handlePaginationDisplay()
})

onUnmounted(() => {
  if (props.canPaginate) window.removeEventListener('resize', handlePaginationDisplay)
})

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

  const options = !passedInstance.instance.path
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
</script>

<template>
  <div class="content">
    <div v-for="(row, rowIndex) in instances" :key="row.label" class="row">
      <div class="header">
        <p>{{ row.label }}</p>
        <hr aria-hidden="true" />
        <div v-if="allowPagination[rowIndex]" class="pagination">
          <ChevronLeftIcon role="button" @click="modsRow.value.scrollLeft -= 170" />
          <ChevronRightIcon role="button" @click="modsRow.value.scrollLeft += 170" />
        </div>
      </div>
      <section ref="modsRow" class="instances">
        <Instance
          v-for="(instance, instanceIndex) in row.instances"
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
}

.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 1rem;

  &:nth-child(even) {
    background: var(--color-bg);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 100%;
    margin-bottom: 1rem;
    gap: 1rem;

    p {
      margin: 0;
      font-size: 1rem;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }

    .pagination {
      display: inherit;
      align-items: inherit;

      svg {
        background: var(--color-raised-bg);
        border-radius: var(--radius-lg);
        width: 1.3rem;
        height: 1.2rem;
        cursor: pointer;
        margin-right: 0.5rem;
        transition: all ease-in-out 0.1s;

        &:hover {
          filter: brightness(150%);
        }
      }
    }
  }

  section {
    display: flex;
    align-items: inherit;
    transition: all ease-in-out 0.4s;
    gap: 1rem;
  }

  .instances {
    display: flex;
    flex-direction: row;
    width: 100%;
    gap: 1rem;
    margin-right: auto;
    scroll-behavior: smooth;
    overflow-x: scroll;
    overflow-y: hidden;

    :deep(.instance-card-item) {
      margin-bottom: 0.1rem;
    }

    &::-webkit-scrollbar {
      width: 0px;
      background: transparent;
    }

    :deep(.instance) {
      min-width: 10.5rem;
      max-width: 10.5rem;
    }
  }
}

.dark-mode {
  .row {
    background-color: rgb(30, 31, 34);
  }
}
</style>
