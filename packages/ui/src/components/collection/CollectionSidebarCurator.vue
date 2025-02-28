<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-3 font-semibold">
      <AutoLink
        class="flex gap-2 items-center w-fit text-primary leading-[1.2] group"
        :to="linkifiedLink"
      >
        <Avatar :src="user.avatar_url" :alt="user.username" size="32px" circle />
        <div class="flex flex-col">
          <span class="flex flex-row flex-nowrap items-center gap-1 group-hover:underline">
            {{ user.username }}
            <ExternalIcon v-if="linkifiedLink.type === 'external'" />
          </span>
        </div>
      </AutoLink>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import Avatar from '../base/Avatar.vue'
import AutoLink from '../base/AutoLink.vue'
import type { User } from '@modrinth/utils'
import { asLink, type Linkish } from '../../utils/link'
import { computed } from 'vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
  user: User
  link: Linkish
}>()

const messages = defineMessages({
  title: {
    id: 'collection.curator.title',
    defaultMessage: 'Curator',
  },
})

const linkifiedLink = computed(() => asLink(props.link))
</script>
