<script setup>
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ClipboardCopyIcon, EditIcon,
  FolderOpenIcon, PlayIcon,
  PlusIcon,
  TrashIcon,
  DownloadIcon,
  GlobeIcon
} from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import { onMounted, onUnmounted, ref } from 'vue'
import ContextMenu from "@/components/ui/ContextMenu.vue";

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
const projectOptions = ref(null)

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

const handleInstanceRightClick = (event, e) => {
  console.log(event, e)
  instanceOptions.value.showMenu(event, e)
}

const handleProjectRightClick = (event, e) => {
  console.log(event, e)
  projectOptions.value.showMenu(event, e)
}

const handleOptionsClick = (args) => {
  console.log(args)
}
</script>

<template>
  <div v-if="props.instances.length > 0" class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr aria-hidden="true" />
      <div v-if="allowPagination" class="pagination">
        <ChevronLeftIcon role="button" @click="modsRow.value.scrollLeft -= 170" />
        <ChevronRightIcon role="button" @click="modsRow.value.scrollLeft += 170" />
      </div>
      <section ref="modsRow" class="instances">
        <Instance
          v-for="instance in row.instances"
          :key="instance?.project_id || instance?.id"
          :instance="instance"
          @contextmenu.prevent.stop="event => row.downloaded ? handleInstanceRightClick(event, instance) : handleProjectRightClick(event, instance)"
        />
      </section>
    </div>
  </div>
  <ContextMenu
    ref="instanceOptions"
    :element-id="`instance-options`"
    :options="[
        'play',
        'install',
        'divider',
        'edit',
        'open',
        'copy',
        'divider',
        'delete',
      ]"
    @option-clicked="handleOptionsClick"
  >
    <template #play>
      <PlayIcon /> Play
    </template>
    <template #install>
      <PlusIcon /> Add content
    </template>
    <template #edit>
      <EditIcon /> Edit
    </template>
    <template #delete>
      <TrashIcon /> Delete
    </template>
    <template #open>
      <FolderOpenIcon /> Open folder
    </template>
    <template #copy>
      <ClipboardCopyIcon /> Copy path
    </template>
  </ContextMenu>
  <ContextMenu
    ref="projectOptions"
    :element-id="`project-options`"
    :options="[
        'install',
        'add',
        'divider',
        'open',
        'copy',
      ]"
    @option-clicked="handleOptionsClick"
  >
    <template #install>
      <DownloadIcon /> Install
    </template>
    <template #add>
      <PlusIcon /> Add to Instance
    </template>
    <template #open>
      <GlobeIcon /> Open in Modrinth
    </template>
    <template #copy>
      <ClipboardCopyIcon /> Copy link
    </template>
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
    &:nth-child(odd) {
      background-color: rgb(30, 31, 34);
    }
  }
}
</style>
