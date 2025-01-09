<template>
  <div v-if="collections.length > 0" class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-2 place-items-center">
      <Tooltip
        v-for="collection in collectionsSorted"
        :key="`user-collection-${collection.id}`"
        class="w-full"
        placement="right"
      >
        <AutoLink
          :to="link(collection)"
          class="flex gap-2 group overflow-hidden"
        >
          <Avatar :src="collection.icon_url" :alt="'Icon for ' + collection.name" size="3rem" />
          <div class="flex flex-col gap-2">
            <span class="items-center text-primary font-bold gap-2 line-clamp-1 text-ellipsis group-hover:underline">
              {{ collection.name }}
            </span>
            <span class="text-secondary font-medium flex items-center gap-1">
              <BoxIcon /> {{ formatMessage(commonMessages.projectsStat, { count: formatNumber(collection.projects.length, false) }) }}
            </span>
          </div>
        </AutoLink>
        <template #popper>
          <div class="flex flex-col p-1 dark">
            <p class="text-sm font-bold text-contrast m-0 max-w-80">{{ collection.name }}</p>
            <p class="text-sm text-primary font-semibold m-0 max-w-80">{{ collection.description }}</p>
          </div>
        </template>
      </Tooltip>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useVIntl, defineMessages } from '@vintl/vintl'
import { computed } from 'vue'
import Avatar from '../base/Avatar.vue'
import AutoLink from '../base/AutoLink.vue'
import type { Collection } from '@modrinth/utils'
import type { Linkish } from '../../utils/link'
import { commonMessages } from '../../utils/common-messages'
import { formatNumber } from '@modrinth/utils'
import { BoxIcon } from '@modrinth/assets'
import { Tooltip } from 'floating-vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
  collections: Collection[],
  link: (collection: Collection) => Linkish
}>()

const collectionsSorted = computed(() => props.collections.slice().sort((a, b) => a.name.localeCompare(b.name)))

const messages = defineMessages({
  title: {
    id: 'user.collections.title',
    defaultMessage: 'Collections',
  },
})
</script>
