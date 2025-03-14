<template>
  <ContentPageHeader>
    <template #icon>
      <Avatar :src="organization.icon_url" :alt="organization.name" size="96px" />
    </template>
    <template #title>
      {{ organization.name }}
    </template>
    <template #summary>
      {{
        organization.description ??formatMessage(messages.defaultDescription)
      }}
    </template>
    <template #stats>
      <div
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
      >
        <UsersIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.membersStat, { count: formatNumber(organization.members.length, false) }) }}
      </div>
      <div
        class="flex items-center gap-2 border-0 border-solid border-divider pr-4 font-semibold"
        :class="{ 'border-r': projectCount > 0 }"
      >
        <BoxIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.projectsStat, { count: formatNumber(projectCount, false) }) }}

      </div>
      <div
        v-if="projectCount > 0"
        v-tooltip="formatNumber(downloadCount, false)"
        class="flex items-center gap-2 font-semibold"
      >
        <DownloadIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.downloadsStat, { count: formatNumber(downloadCount) }) }}
      </div>
    </template>
    <template #actions>
      <slot name="actions" />
    </template>
  </ContentPageHeader>
</template>
<script setup lang="ts">
import { UsersIcon, BoxIcon, DownloadIcon } from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import { formatNumber, type Organization } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

withDefaults(
  defineProps<{
    organization: Organization
    projectCount?: number
    downloadCount?: number
  }>(),
  {
    projectCount: 0,
    downloadCount: 0,
  },
)

const messages = defineMessages({
  defaultDescription: {
    id: 'organization.description.default',
    defaultMessage: 'An organization on Modrinth.',
  },
})
</script>
