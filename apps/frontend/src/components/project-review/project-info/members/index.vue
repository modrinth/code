<template>
	<p v-if="membersLoading || organizationLoading" role="status" class="m-0">
		{{ formatMessage(messages.loading) }}
	</p>
	<p v-else-if="membersError || organizationError" role="alert" class="m-0">
		{{ formatMessage(messages.loadError) }}
	</p>
	<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
		<li v-if="organization" class="flex flex-col gap-2.5">
			<OrganizationItem :organization="organization" />
			<ul
				v-if="organizationMembers.length"
				class="flex list-none flex-col gap-2 border-0 border-t border-solid border-surface-4 p-0 pt-2.5"
			>
				<MemberItem
					v-for="member in organizationMembers"
					:key="member.user.id"
					:member="member"
					:stats="memberStats[member.user.id] ?? []"
				/>
			</ul>
		</li>
		<MemberItem
			v-for="member in directMembers"
			:key="member.user.id"
			:member="member"
			:stats="memberStats[member.user.id] ?? []"
		/>
	</ul>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import MemberItem from './member-item.vue'
import OrganizationItem from './organization-item.vue'

const {
	members,
	memberStats,
	membersLoading,
	membersError,
	organization,
	organizationMembers,
	organizationLoading,
	organizationError,
} = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const directMembers = computed(() => {
	if (!organization.value) return members.value
	const organizationMemberIds = new Set(organizationMembers.value.map((member) => member.user.id))
	return members.value.filter((member) => !organizationMemberIds.has(member.user.id))
})
</script>
