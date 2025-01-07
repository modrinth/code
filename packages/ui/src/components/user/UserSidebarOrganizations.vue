<template>
  <div v-if="organizations.length > 0" class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="grid grid-cols-5 gap-2 place-items-center">
      <AutoLink
        v-for="org in organizationsSorted"
        :key="`user-org-${org.id}`"
        v-tooltip="org.name"
        :to="link(org)"
      >
        <Avatar :src="org.icon_url" :alt="'Icon for ' + org.name" size="3rem" />
      </AutoLink>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useVIntl, defineMessages } from '@vintl/vintl'
import { computed } from 'vue'
import Avatar from '../base/Avatar.vue'
import AutoLink from '../base/AutoLink.vue'
import type { Organization } from '@modrinth/utils'
import type { Linkish } from '../../utils/link'

const { formatMessage } = useVIntl()

const props = defineProps<{
  organizations: Organization[],
  link: (org: Organization) => Linkish
}>()

const organizationsSorted = computed(() => props.organizations.slice().sort((a, b) => a.name.localeCompare(b.name)))

const messages = defineMessages({
  title: {
    id: 'user.organizations.title',
    defaultMessage: 'Organizations',
  },
})
</script>
