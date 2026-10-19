<template>
	<Section :heading="formatMessage(messages.members)">
		<p v-if="membersLoading || organizationLoading" role="status" class="m-0">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="membersError || organizationError" role="alert" class="m-0">
			{{ formatMessage(messages.loadError) }}
		</p>
		<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
			<li v-if="organization" class="flex flex-col gap-2.5">
				<OrganizationItem :organization="organization" :stats="organizationStats" />
				<ul
					v-if="organizationProjectMembers.length"
					class="m-0 ml-9 flex list-none flex-col gap-2 p-0"
				>
					<MemberItem
						v-for="member in organizationProjectMembers"
						:key="member.user.id"
						class="organization-member"
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
	</Section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import Section from '../section.vue'
import MemberItem from './member-item.vue'
import OrganizationItem from './organization-item.vue'

const {
	members,
	memberStats,
	membersLoading,
	membersError,
	organization,
	organizationStats,
	organizationMembers,
	organizationLoading,
	organizationError,
} = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const organizationMemberIds = computed(
	() => new Set(organizationMembers.value.map((member) => member.user.id)),
)
const organizationProjectMembers = computed(() =>
	organization.value
		? members.value.filter((member) => organizationMemberIds.value.has(member.user.id))
		: [],
)
const directMembers = computed(() => {
	if (!organization.value) return members.value
	return members.value.filter((member) => !organizationMemberIds.value.has(member.user.id))
})
</script>

<style scoped>
.organization-member {
	position: relative;
}

.organization-member::before,
.organization-member::after {
	position: absolute;
	left: -1.3rem;
	pointer-events: none;
	content: '';
	border-color: var(--color-divider);
	border-style: solid;
	border-width: 0;
}

.organization-member::before {
	top: -2rem;
	bottom: -0.5rem;
	border-left-width: 1px;
}

.organization-member:last-child::before {
	bottom: calc(100% - 1rem);
}

.organization-member::after {
	top: 1rem;
	width: 1.25rem;
	border-top-width: 1px;
}
</style>
