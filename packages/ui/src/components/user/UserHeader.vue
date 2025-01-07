<template>
  <ContentPageHeader>
    <template #icon>
      <Avatar :src="user.avatar_url" :alt="user.username" size="96px" circle />
    </template>
    <template #title>
      {{ user.username }}
    </template>
    <template #summary>
      {{
        user.bio
          ? user.bio
          : projectCount > 0
            ? formatMessage(messages.creatorBio)
            : formatMessage(messages.userBio)
      }}
    </template>
    <template #stats>
      <div
        v-if="projectCount > 0"
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
      >
        <BoxIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.projectsStat, { count: formatNumber(projectCount, false) }) }}

      </div>
      <div
        v-if="projectCount > 0"
        v-tooltip="formatNumber(downloadCount, false)"
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
      >
        <DownloadIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.downloadsStat, { count: formatNumber(downloadCount) }) }}
      </div>
      <div
        v-tooltip="
          formatMessage(commonMessages.dateAtTimeTooltip, {
            date: new Date(user.created),
            time: new Date(user.created),
          })
        "
        class="flex items-center gap-2 font-semibold"
      >
        <CalendarIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(messages.joinedStat, { date: dayjs(user.created).fromNow() }) }}
      </div>
    </template>
    <template #actions>
      <slot name="actions" />
    </template>
  </ContentPageHeader>
</template>
<script setup lang="ts">
import { BoxIcon, CalendarIcon, DownloadIcon } from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import { formatNumber, type User } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'
import dayjs from 'dayjs'
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

withDefaults(
  defineProps<{
    user: User
    currentUser?: boolean
    projectCount?: number
    downloadCount?: number
  }>(),
  {
    currentUser: false,
    projectCount: 0,
    downloadCount: 0,
  },
)

const messages = defineMessages({
  userBio: {
    id: 'user.bio.default',
    defaultMessage: 'A Modrinth user.',
  },
  creatorBio: {
    id: 'user.bio.creator',
    defaultMessage: 'A creator on Modrinth.',
  },
  joinedStat: {
    id: 'user.stat.joined',
    defaultMessage: 'Joined {date}',
  },
})
</script>
