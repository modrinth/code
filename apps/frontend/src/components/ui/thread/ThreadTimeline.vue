<template>
	<template v-for="entry in entries" :key="entry.id">
		<ThreadMessage
			v-if="entry.type === 'message'"
			:message="entry.message"
			:review-issue-count="reviewIssueCounts.get(entry.message) ?? 0"
			:project-owner-id="projectOwnerId ?? undefined"
			:members="members"
			:auth="auth"
			:report="report ?? undefined"
			:raised="raised"
			:image-previews="imagePreviews"
			:class="messageClass"
			@update-thread="emit('update-thread')"
			@open-image="emit('open-image', $event)"
		/>
		<ThreadIssueHistory
			v-else
			:entry="entry"
			:raised="raised"
			:author="entry.authorId ? members[entry.authorId] : undefined"
			:class="messageClass"
			@update-thread="emit('update-thread')"
		/>
	</template>
</template>

<script setup lang="ts" generic="T extends { id?: string | null; created: string }">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'

import { isStaff } from '~/helpers/users.js'

import ThreadIssueHistory from './ThreadIssueHistory.vue'
import ThreadMessage from './ThreadMessage.vue'
import { buildThreadTimeline, countThreadReviewIssues } from './timeline'

const props = withDefaults(
	defineProps<{
		projectOwnerId?: string | null
		messages: readonly T[]
		issues?: readonly Labrinth.Threads.v3.ThreadIssue[]
		members: Record<string, { username: string; avatar_url?: string | null; role?: string }>
		auth: { user?: { role: string } | null }
		report?: object | null
		raised?: boolean
		imagePreviews?: boolean
		messageClass?: string
	}>(),
	{ issues: () => [], report: null, raised: false, imagePreviews: false, messageClass: '' },
)

const emit = defineEmits<{
	'update-thread': []
	'open-image': [image: { src: string; alt: string; element: HTMLImageElement }]
}>()

const entries = computed(() =>
	buildThreadTimeline(props.messages, props.issues, !!isStaff(props.auth.user)),
)
const reviewIssueCounts = computed(() => countThreadReviewIssues(props.messages, props.issues))
</script>
