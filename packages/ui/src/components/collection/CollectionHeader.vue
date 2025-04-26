<template>
  <ContentPageHeader>
    <template #icon>
      <Avatar :src="collection.icon_url" :alt="collection.name" size="64px" />
    </template>
    <template #title>
      {{ collection.name }}
    </template>
    <template #stats>
      <div
        v-if="collection.projects.length > 0"
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
      >
        <CollectionStatusBadge :status="collection.status" />
      </div>
      <div
        v-if="collection.projects.length > 0"
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
      >
        <BoxIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.projectsStat, { count: formatNumber(collection.projects.length, false) }) }}

      </div>
      <div
        v-tooltip="
          formatMessage(commonMessages.dateAtTimeTooltip, {
            date: new Date(collection.updated),
            time: new Date(collection.updated),
          })
        "
        class="flex items-center gap-2 font-semibold"
      >
        <HistoryIcon class="h-5 w-5 text-secondary" />
        {{ formatMessage(commonMessages.updatedDate, { date: dayjs(collection.updated).fromNow() }) }}
      </div>
    </template>
    <template #actions>
      <slot name="actions" />
    </template>
  </ContentPageHeader>
</template>
<script setup lang="ts">
import { GlobeIcon, BoxIcon, HistoryIcon } from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import { type Collection, formatNumber } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'
import dayjs from 'dayjs'
import { useVIntl } from '@vintl/vintl'
import ProjectStatusBadge from '../project/ProjectStatusBadge.vue'
import CollectionStatusBadge from './CollectionStatusBadge.vue'

const { formatMessage } = useVIntl()

defineProps<{
  collection: Collection
}>()
</script>
