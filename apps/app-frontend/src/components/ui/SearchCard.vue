<template>
  <div
    class="card-shadow p-4 bg-bg-raised rounded-xl flex gap-3 group cursor-pointer hover:brightness-90 transition-all"
    @click="
      () => {
        emit('open')
        $router.push({
          path: `/project/${project.project_id ?? project.id}`,
          query: { i: props.instance ? props.instance.path : undefined },
        })
      }
    "
  >
    <div class="icon w-[96px] h-[96px] relative">
      <Avatar :src="project.icon_url" size="96px" class="search-icon origin-top transition-all" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">
          {{ project.title }}
        </span>
        <span v-if="project.author" class="text-secondary"> by {{ project.author }}</span>
      </div>
      <div class="m-0 line-clamp-2">
        {{ project.description }}
      </div>
      <div v-if="categories.length > 0" class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-if="project.project_type === 'mod' || project.project_type === 'modpack'"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          <template v-if="project.client_side === 'optional' && project.server_side === 'optional'">
            Client or server
          </template>
          <template
            v-else-if="
              (project.client_side === 'optional' || project.client_side === 'required') &&
              (project.server_side === 'optional' || project.server_side === 'unsupported')
            "
          >
            Client
          </template>
          <template
            v-else-if="
              (project.server_side === 'optional' || project.server_side === 'required') &&
              (project.client_side === 'optional' || project.client_side === 'unsupported')
            "
          >
            Server
          </template>
          <template
            v-else-if="
              project.client_side === 'unsupported' && project.server_side === 'unsupported'
            "
          >
            Unsupported
          </template>
          <template
            v-else-if="project.client_side === 'required' && project.server_side === 'required'"
          >
            Client and server
          </template>
        </div>
        <div
          v-for="tag in categories"
          :key="tag"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ formatCategory(tag.name) }}
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
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
        <div class="absolute bottom-0 right-0 w-fit">
          <ButtonStyled color="brand" type="outlined">
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
import { TagsIcon, DownloadIcon, HeartIcon, PlusIcon, CheckIcon } from '@modrinth/assets'
import { ButtonStyled, Avatar } from '@modrinth/ui'
import { formatNumber, formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref, computed } from 'vue'
import { install as installVersion } from '@/store/install.js'
import { useRouter } from 'vue-router'
dayjs.extend(relativeTime)

const router = useRouter()

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

const emit = defineEmits(['open', 'install'])

const installing = ref(false)

async function install() {
  installing.value = true
  await installVersion(
    props.project.project_id ?? props.project.id,
    null,
    props.instance ? props.instance.path : null,
    'SearchCard',
    () => {
      installing.value = false
      emit('install', props.project.project_id ?? props.project.id)
    },
    (profile) => {
      router.push(`/instance/${profile}`)
    },
  )
}

const modpack = computed(() => props.project.project_type === 'modpack')
</script>
