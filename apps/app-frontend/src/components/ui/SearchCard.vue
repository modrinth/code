<template>
  <div
    class="button-base p-4 bg-bg-raised rounded-xl flex gap-3 group"
    @click="
      () => {
        emits('open')
        $router.push({
          path: `/project/${project.project_id ?? project.id}`,
          query: { i: props.instance ? props.instance.path : undefined },
        })
      }
    "
  >
    <div class="icon">
      <Avatar :src="project.icon_url" size="96px" class="search-icon" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">{{
          project.title
        }}</span>
        <span v-if="project.author" class="text-secondary"> by {{ project.author }}</span>
      </div>
      <div class="m-0 line-clamp-2">
        {{ project.description }}
      </div>
      <div class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-for="tag in categories"
          :key="tag"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ formatCategory(tag.name) }}
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0">
      <div class="flex items-center gap-2">
        <DownloadIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.downloads) }}
          <span class="text-secondary">downloads</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <HeartIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.follows ?? project.followers) }}
          <span class="text-secondary">followers</span>
        </span>
      </div>
      <div class="mt-auto relative">
        <div
          class="flex items-center gap-2 group-hover:-translate-y-3 group-hover:opacity-0 group-focus-within:opacity-0 group-hover:scale-95 group-focus-within:scale-95 transition-all"
        >
          <HistoryIcon class="shrink-0" />
          <span>
            <span class="text-secondary">Updated</span>
            {{ dayjs(project.date_modified ?? project.updated).fromNow() }}
          </span>
        </div>
        <div
          class="opacity-0 scale-95 translate-y-3 group-hover:translate-y-0 group-hover:scale-100 group-hover:opacity-100 group-focus-within:opacity-100 group-focus-within:scale-100 absolute bottom-0 right-0 transition-all w-fit"
        >
          <ButtonStyled color="brand">
            <button
              :disabled="installed || installing"
              class="shrink-0 no-wrap"
              @click.stop="install()"
            >
              <template v-if="!installed">
                <DownloadIcon v-if="modpack || instance" />
                <PlusIcon v-else />
              </template>
              <CheckIcon v-else />
              {{
                installing
                  ? 'Installing'
                  : installed
                    ? 'Installed'
                    : modpack || instance
                      ? 'Install'
                      : 'Add to an instance'
              }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  TagsIcon,
  DownloadIcon,
  HeartIcon,
  PlusIcon,
  CheckIcon,
  HistoryIcon,
} from '@modrinth/assets'
import { ButtonStyled, Avatar } from '@modrinth/ui'
import { formatNumber, formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref, computed } from 'vue'
import { install as installVersion } from '@/store/install.js'
dayjs.extend(relativeTime)

const props = defineProps({
  backgroundImage: {
    type: String,
    default: null,
  },
  project: {
    type: Object,
    required: true,
  },
  categories: {
    type: Array,
    required: true,
  },
  instance: {
    type: Object,
    default: null,
  },
  featured: {
    type: Boolean,
    default: false,
  },
  installed: {
    type: Boolean,
    default: false,
  },
})

const emits = defineEmits(['open'])

const installing = ref(false)
const installed = ref(props.installed)

async function install() {
  installing.value = true
  await installVersion(
    props.project.project_id,
    null,
    props.instance ? props.instance.path : null,
    'SearchCard',
    (version) => {
      installing.value = false

      if (props.instance && version) {
        installed.value = true
      }
    },
  )
}

const modpack = computed(() => props.project.project_type === 'modpack')
</script>
