<template>
	<p v-if="membersLoading" role="status">
		{{ formatMessage(messages.loading) }}
	</p>
	<p v-else-if="membersError" role="alert">
		{{ formatMessage(messages.loadError) }}
	</p>
	<ul v-else class="m-0 flex list-none flex-col gap-4 p-0">
		<MemberItem
			v-for="member in members"
			:key="member.user.id"
			:member="member"
			:stats="memberStats[member.user.id] ?? []"
		/>
	</ul>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import MemberItem from './member-item.vue'

const { members, memberStats, membersLoading, membersError } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
