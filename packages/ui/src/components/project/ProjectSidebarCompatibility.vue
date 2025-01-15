<template>
  <div v-if="project.versions.length > 0" class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <section class="flex flex-col gap-2">
      <h3 class="text-primary text-base m-0">{{ formatMessage(messages.minecraftJava) }}</h3>
      <div class="flex flex-wrap gap-1">
        <TagItem
          v-for="version in getVersionsToDisplay(project, tags.gameVersions)"
          :key="`version-tag-${version}`"
        >
          {{ version }}
        </TagItem>
      </div>
    </section>
    <section v-if="project.project_type !== 'resourcepack'" class="flex flex-col gap-2">
      <h3 class="text-primary text-base m-0">{{ formatMessage(messages.platforms) }}</h3>
      <div class="flex flex-wrap gap-1">
        <TagItem
          v-for="platform in project.loaders"
          :key="`platform-tag-${platform}`"
          :action="() => router.push(`/${project.project_type}s?g=categories:${platform}`)"
          :style="`--_color: var(--color-platform-${platform})`"
        >
          <svg v-html="tags.loaders.find((x) => x.name === platform).icon"></svg>
          {{ formatCategory(platform) }}
        </TagItem>
      </div>
    </section>
    <section
      v-if="
        (project.project_type === 'mod' || project.project_type === 'modpack') &&
        !(project.client_side === 'unsupported' && project.server_side === 'unsupported') &&
        !(project.client_side === 'unknown' && project.server_side === 'unknown')
      "
      class="flex flex-col gap-2"
    >
      <h3 class="text-primary text-base m-0">{{ formatMessage(messages.environments) }}</h3>
      <div class="flex flex-wrap gap-1">
        <TagItem
          v-if="
            (project.client_side === 'required' && project.server_side !== 'required') ||
            (project.client_side === 'optional' && project.server_side === 'optional')
          "
        >
          <ClientIcon aria-hidden="true" />
          Client-side
        </TagItem>
        <TagItem
          v-if="
            (project.server_side === 'required' && project.client_side !== 'required') ||
            (project.client_side === 'optional' && project.server_side === 'optional')
          "
        >
          <ServerIcon aria-hidden="true" />
          Server-side
        </TagItem>
        <TagItem v-if="false">
          <UserIcon aria-hidden="true" />
          Singleplayer
        </TagItem>
        <TagItem
          v-if="
            project.project_type !== 'datapack' &&
            project.client_side !== 'unsupported' &&
            project.server_side !== 'unsupported' &&
            project.client_side !== 'unknown' &&
            project.server_side !== 'unknown'
          "
        >
          <MonitorSmartphoneIcon aria-hidden="true" />
          Client and server
        </TagItem>
      </div>
    </section>
  </div>
</template>
<script setup lang="ts">
import { ClientIcon, MonitorSmartphoneIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import { formatCategory, getVersionsToDisplay } from '@modrinth/utils'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { useRouter } from 'vue-router'
import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()
const router = useRouter()

type EnvironmentValue = 'optional' | 'required' | 'unsupported' | 'unknown'

defineProps<{
  project: {
    actualProjectType: string
    project_type: string
    loaders: string[]
    client_side: EnvironmentValue
    server_side: EnvironmentValue
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    versions: any[]
  }
  tags: {
    gameVersions: GameVersionTag[]
    loaders: PlatformTag[]
  }
}>()

const messages = defineMessages({
  title: {
    id: 'project.about.compatibility.title',
    defaultMessage: 'Compatibility',
  },
  minecraftJava: {
    id: 'project.about.compatibility.game.minecraftJava',
    defaultMessage: 'Minecraft: Java Edition',
  },
  platforms: {
    id: 'project.about.compatibility.platforms',
    defaultMessage: 'Platforms',
  },
  environments: {
    id: 'project.about.compatibility.environments',
    defaultMessage: 'Supported environments',
  },
})
</script>
