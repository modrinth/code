<script setup lang="ts">
import {
  VersionIcon,
  ImageIcon,
  BookmarkIcon,
  DownloadIcon,
  HeartIcon,
  MoreVerticalIcon,
  ExternalIcon,
  LinkIcon,
  ReportIcon,
  SpinnerIcon,
  CheckIcon,
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, OverflowMenu } from '@modrinth/ui'
import type { GameInstance } from '@/helpers/types'
import { computed, ref, type Ref } from 'vue'
import { install as installVersion } from '@/store/install'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { isSearchResult, type Project, type SearchResult } from '@modrinth/utils'
import type { InstanceContentMap } from '@/composables/instance-context.ts'

const { formatMessage } = useVIntl()

const props = defineProps<{
  project: Project | SearchResult
  instance?: GameInstance
  instanceContent?: InstanceContentMap
}>()

const installing = ref(false)

const installed: Ref<boolean> = ref(false)

checkInstallStatus()

function checkInstallStatus() {
  if (props.instanceContent) {
    installed.value = Object.values(props.instanceContent).some((content) => {
      if (content.metadata?.project_id === projectId.value) {
        return true
      }
    })
  }
}

async function install(toInstance: boolean) {
  if (toInstance) {
    installing.value = true
  }
  await installVersion(
    projectId.value,
    null,
    props.instance && toInstance ? props.instance.path : null,
    'SearchCard',
    () => {
      if (toInstance) {
        installing.value = false
        installed.value = true
      }
    },
  )
}

const modpack = computed(() => props.project.project_type === 'modpack')

const projectWebUrl = computed(() => `https://modrinth.com/${props.project.project_type}/${props.project.slug}`)

const tooltip = defineMessages({
  installing: {
    id: 'project.card.actions.installing.tooltip',
    defaultMessage: 'This project is being installed',
  },
  installed: {
    id: 'project.card.actions.installed.tooltip',
    defaultMessage: 'This project is already installed',
  },
})

const messages = defineMessages({
  viewVersions: {
    id: 'project.card.actions.view-versions',
    defaultMessage: 'View versions',
  },
  viewGallery: {
    id: 'project.card.actions.view-gallery',
    defaultMessage: 'View gallery',
  },
})

const projectId = computed(() => isSearchResult(props.project) ? props.project.project_id : props.project.id)

const copyText = (text: string) => {
  navigator.clipboard.writeText(text);
}
</script>

<template>
  <ButtonStyled color="brand">
    <button v-tooltip="installing ? formatMessage(tooltip.installing) : installed ? formatMessage(tooltip.installed) : null" :disabled="installing || installed" @click="() => install(true)">
      <SpinnerIcon v-if="installing" />
      <CheckIcon v-else-if="installed" />
      <DownloadIcon v-else />
      {{ formatMessage(
        installing ? commonMessages.installingButton :
        installed ? commonMessages.installedButton :
        commonMessages.installButton)
      }}
    </button>
  </ButtonStyled>
  <!-- TODO: Add in later -->
  <ButtonStyled v-if="false" circular>
    <button v-tooltip="'Follow'">
      <HeartIcon />
    </button>
  </ButtonStyled>
  <ButtonStyled v-if="false" circular>
    <button v-tooltip="'Save'">
      <BookmarkIcon />
    </button>
  </ButtonStyled>
  <ButtonStyled circular type="transparent">
    <OverflowMenu
      :options="[
        {
          id: 'install-elsewhere',
          color: 'primary',
          action: () => install(false),
          shown: !!instance && !modpack,
        },
        {
          divider: true,
          shown: !!instance && !modpack,
        },
        {
          id: 'versions',
          link: `/project/${projectId}/versions`,
        },
        {
          id: 'gallery',
          link: `/project/${projectId}/gallery`,
          shown: (project.gallery?.length ?? 0) > 0,
        },
        {
          id: 'open-link',
          link: projectWebUrl,
        },
        {
          id: 'copy-link',
          action: () => copyText(projectWebUrl),
        },
        {
          divider: true,
        },
        {
          id: 'report',
          color: 'red',
          hoverFilled: true,
          action: () => {},
        },
      ]"
    >
      <MoreVerticalIcon />
      <template #install-elsewhere>
        <DownloadIcon /> {{ formatMessage(commonMessages.installToButton) }}
      </template>
      <template #versions>
        <VersionIcon /> {{ formatMessage(messages.viewVersions) }}
      </template>
      <template #gallery>
        <ImageIcon /> {{ formatMessage(messages.viewGallery) }}
      </template>
      <template #open-link>
        <ExternalIcon /> {{ formatMessage(commonMessages.openInBrowserButton) }}
      </template>
      <template #copy-link>
        <LinkIcon /> {{ formatMessage(commonMessages.copyLinkButton) }}
      </template>
      <template #report> <ReportIcon /> {{ formatMessage(commonMessages.reportButton) }} </template>
    </OverflowMenu>
  </ButtonStyled>
</template>
