<template>
  <NewProjectCard
    :project="project"
    :link="
      asLink(
        {
          path: `/project/${projectId}`,
          query: { i: props.instance ? props.instance.path : undefined },
        },
        () => emit('open'),
      )
    "
    :experimental-colors="themeStore.featureFlags.project_card_background"
    :creator-link=" creator ?
      asLink(
        {
          path: `/user/${creator}`,
          query: { i: props.instance ? props.instance.path : undefined },
        },
        () => emit('open'),
      ) : undefined
    "
  >
    <template #actions>
      <ButtonStyled color="brand">
        <button
          :disabled="installed || installing"
          class="shrink-0 no-wrap"
          @click.stop="install()"
        >
          <template v-if="!installed">
            <DownloadIcon />
          </template>
          <CheckIcon v-else />
          {{ installing ? 'Installing' : installed ? 'Installed' : 'Install' }}
        </button>
      </ButtonStyled>
    </template>
  </NewProjectCard>
</template>

<script setup lang="ts">
import { DownloadIcon, CheckIcon } from '@modrinth/assets'
import { ButtonStyled, NewProjectCard, asLink } from '@modrinth/ui'
import type {
  Project,
  SearchResult} from '@modrinth/utils';
import {
  isSearchResult
} from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref, computed } from 'vue'
import { install as installVersion } from '@/store/install.js'
import { useTheming } from '@/store/state.js'
import type { GameInstance } from '@/helpers/types'
dayjs.extend(relativeTime)

const themeStore = useTheming()

const props = withDefaults(defineProps<{
  project: Project | SearchResult
  instance?: GameInstance
  installed?: boolean
}>(), {
  instance: undefined,
  installed: false,
})

const emit = defineEmits(['open', 'install'])

const installing = ref(false)

async function install() {
  installing.value = true
  await installVersion(
    projectId.value,
    null,
    props.instance ? props.instance.path : null,
    'SearchCard',
    () => {
      installing.value = false
      emit('install', projectId.value)
    },
  )
}

const projectId = computed(() => isSearchResult(props.project) ? props.project.project_id : props.project.id)
const creator = computed(() => isSearchResult(props.project) ? props.project.author : undefined)
</script>
