<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-3 font-semibold [&>div]:flex [&>div]:gap-2 [&>div]:items-center">
      <div
           v-tooltip="
          formatMessage(commonMessages.dateAtTimeTooltip, {
            date: new Date(collection.created),
            time: new Date(collection.created),
          })
        ">
        <CalendarIcon aria-hidden="true" />
        <div>
          {{ formatMessage(commonMessages.createdAgoLabel, { ago: dayjs(collection.created).fromNow() }) }}
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { CalendarIcon } from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import dayjs from 'dayjs'
import type { Collection } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'

const { formatMessage } = useVIntl()

defineProps<{
  collection: Collection
}>()

const messages = defineMessages({
  title: {
    id: 'collection.details.title',
    defaultMessage: 'Details',
  },
})
</script>
