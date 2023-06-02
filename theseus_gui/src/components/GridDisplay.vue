<script setup>
import Instance from '@/components/ui/Instance.vue'
import { ref } from 'vue'
import {
  ClipboardCopyIcon,
  EditIcon,
  FolderOpenIcon,
  PlayIcon, PlusIcon,
  TrashIcon,
  StopCircleIcon
} from "omorphia";
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
})
const instanceOptions = ref(null)
const instanceComponents = ref(null)
const baseOptions = [
  { name: 'add_content' },
  { name: 'divider' },
  { name: 'edit' },
  { name: 'open' },
  { name: 'copy' },
  { name: 'divider' },
  {
    name: 'delete',
    color: 'danger',
  },
]

const playingOptions = ref([
  {
    name: 'play',
    color: 'primary',
  },
  ... baseOptions
])

const stoppingOptions = ref([
  {
    name: 'stop',
    color: 'danger',
  },
  ... baseOptions
])

const handleRightClick = (event, e, passedOptions) => {
  console.log(event, e)
  instanceOptions.value.showMenu(event, e, passedOptions)
}

const handleOptionsClick = (args) => {
  console.log(args)
  switch (args.option) {
    case 'play':
      args.item.play()
      break
    case 'stop':
      args.item.stop()
      break
    case 'edit':
      args.item.seeInstance()
      break
    case 'delete':
      args.item.deleteInstance()
      break
    case 'open':
      args.item.openFolder()
      break
    case 'copy':
      navigator.clipboard.writeText(args.item.instance.path)
      break
  }
}
</script>
<template>
  <div class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr />
    </div>
    <section class="instances">
      <Instance
        v-for="(instance, index) in props.instances"
        ref="instanceComponents"
        :key="instance.id"
        :instance="instance"
        @contextmenu.prevent.stop="event => handleRightClick(event, instanceComponents[index], !instanceComponents[index].playing ? playingOptions : stoppingOptions)"
      />
    </section>
  </div>
  <ContextMenu
    ref="instanceOptions"
    :element-id="`instance-options`"
    @option-clicked="handleOptionsClick"
  >
    <template #play>
      <PlayIcon /> Play
    </template>
    <template #stop>
      <StopCircleIcon /> Stop
    </template>
    <template #add_content>
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
</template>
<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 1rem;

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
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    width: 100%;
    gap: 1rem;
    margin-right: auto;
    scroll-behavior: smooth;
    overflow-y: auto;
    transition: all ease-in-out 0.3s;
  }
}

.dark-mode {
  .row {
    &:nth-child(even) {
      background-color: rgb(30, 31, 34);
    }
  }
}
</style>
