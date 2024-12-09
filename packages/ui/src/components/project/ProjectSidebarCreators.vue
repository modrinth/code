<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-3 font-semibold">
      <template v-if="organization">
        <AutoLink
          class="flex gap-2 items-center w-fit text-primary leading-[1.2] group"
          :to="orgLink(organization.slug)"
          :target="linkTarget ?? null"
        >
          <Avatar :src="organization.icon_url" :alt="organization.name" size="32px" />
          <div class="flex flex-col flex-nowrap justify-center">
            <span class="group-hover:underline">
              {{ organization.name }}
            </span>
            <span class="text-secondary text-sm font-medium flex items-center gap-1"
              ><OrganizationIcon /> Organization</span
            >
          </div>
        </AutoLink>
        <hr v-if="sortedMembers.length > 0" class="w-full border-button-border my-0.5" />
      </template>
      <AutoLink
        v-for="member in sortedMembers"
        :key="`member-${member.id}`"
        class="flex gap-2 items-center w-fit text-primary leading-[1.2] group"
        :to="userLink(member.user.username)"
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
import { CrownIcon, ExternalIcon, OrganizationIcon } from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import Avatar from '../base/Avatar.vue'
import AutoLink from '../base/AutoLink.vue'
import { computed } from 'vue'

const { formatMessage } = useVIntl()

type TeamMember = {
  id: string
  role: string
  is_owner: boolean
  accepted: boolean
  user: {
    id: string
    username: string
    avatar_url: string
  }
}

const props = defineProps<{
  organization?: {
    id: string
    slug: string
    name: string
    icon_url: string
    avatar_url: string
    members: TeamMember[]
  } | null
  members: TeamMember[]
  orgLink: (slug: string) => string
  userLink: (username: string) => string
  linkTarget?: string
}>()

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const sortedMembers = computed(() => {
  const acceptedMembers = props.members.filter((x) => x.accepted === undefined || x.accepted)
  const owner = acceptedMembers.find((x) =>
    props.organization
      ? props.organization.members.some(
          (orgMember) => orgMember.user.id === x.user.id && orgMember.is_owner,
        )
      : x.is_owner,
  )

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
    id: 'project.about.creators.title',
    defaultMessage: 'Creators',
  },
  owner: {
    id: 'project.about.creators.owner',
    defaultMessage: 'Project owner',
  },
})
</script>
