<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-3 font-semibold">
      <AutoLink
        v-for="member in sortedMembers"
        :key="`member-${member.user.id}`"
        class="flex gap-2 items-center w-fit text-primary leading-[1.2] group"
        :to="userLink(member.user)"
        :target="linkTarget ?? null"
      >
        <Avatar :src="member.user.avatar_url" :alt="member.user.username" size="32px" circle />
        <div class="flex flex-col">
          <span class="flex flex-row flex-nowrap items-center gap-1 group-hover:underline">
            {{ member.user.username }}
            <CrownIcon
              v-if="member.is_owner"
              v-tooltip="formatMessage(messages.owner)"
              class="text-brand-orange"
            />
            <ExternalIcon v-if="linkTarget === '_blank'" />
          </span>
          <span class="text-secondary text-sm font-medium">{{ member.role }}</span>
        </div>
      </AutoLink>
    </div>
  </div>
</template>
<script setup lang="ts">
import { CrownIcon, ExternalIcon } from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import Avatar from '../base/Avatar.vue'
import AutoLink from '../base/AutoLink.vue'
import { computed } from 'vue'
import type { OrganizationMember, User } from '@modrinth/utils'

const { formatMessage } = useVIntl()

const props = defineProps<{
  members: OrganizationMember[]
  userLink: (user: User) => string
  linkTarget?: string
}>()

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const sortedMembers = computed(() => {
  const acceptedMembers = props.members.filter((x) => x.accepted === undefined || x.accepted)
  const owner = acceptedMembers.find((x) => x.is_owner)
  const rest = acceptedMembers.filter((x) => !owner || x.user.id !== owner.user.id) || []

  rest.sort((a, b) => {
    if (a.role === b.role) {
      return a.user.username.localeCompare(b.user.username)
    } else {
      return a.role.localeCompare(b.role)
    }
  })

  return owner ? [owner, ...rest] : rest
})

const messages = defineMessages({
  title: {
    id: 'organization.members.title',
    defaultMessage: 'Members',
  },
  owner: {
    id: 'organization.members.owner',
    defaultMessage: 'Organization owner',
  },
})
</script>
