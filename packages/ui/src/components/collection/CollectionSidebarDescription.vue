<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div>
      {{ collection.description }}
    </div>
  </div>
</template>
<script setup lang="ts">
import { useVIntl, defineMessages } from '@vintl/vintl'
import type { Collection } from '@modrinth/utils'

const { formatMessage } = useVIntl()

defineProps<{
  collection: Collection
}>()

const messages = defineMessages({
  title: {
    id: 'collection.description.title',
    defaultMessage: 'Description',
  },
})
</script>
