<template>
	<div>
		<section class="universal-card">
			<Breadcrumbs
				v-if="breadcrumbsStack"
				:current-title="`Report ${reportId}`"
				:link-stack="breadcrumbsStack"
			/>
			<h2>Report details</h2>
			<ReportInfo :report="report" :show-thread="false" :show-message="false" :auth="auth" />
		</section>
		<section v-if="report && thread" class="universal-card">
			<h2>Messages</h2>
			<ConversationThread
				:thread="thread"
				:report="report"
				:auth="auth"
				@update-thread="updateThread"
			/>
		</section>
	</div>
</template>
<script setup>
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import ReportInfo from '~/components/ui/report/ReportInfo.vue'
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { addReportMessage } from '~/helpers/threads.js'

const props = defineProps({
	reportId: {
		type: String,
		required: true,
	},
	breadcrumbsStack: {
		type: Array,
		default: null,
	},
	auth: {
		type: Object,
		required: true,
	},
})

const client = injectModrinthClient()
const queryClient = useQueryClient()

// Fetch raw report
const { data: rawReport } = useQuery({
	queryKey: computed(() => ['report', props.reportId]),
	queryFn: () => client.labrinth.reports_v3.get(props.reportId),
})

// Compute user IDs needed
const userIds = computed(() => {
	if (!rawReport.value) return []
	const ids = [rawReport.value.reporter]
	if (rawReport.value.item_type === 'user') {
		ids.push(rawReport.value.item_id)
	}
	return ids
})

// Fetch users
const { data: users } = useQuery({
	queryKey: computed(() => ['users', userIds.value]),
	queryFn: () => client.labrinth.users_v2.getMultiple(userIds.value),
	enabled: computed(() => userIds.value.length > 0),
})

// Version ID if applicable
const versionId = computed(() =>
	rawReport.value?.item_type === 'version' ? rawReport.value.item_id : null,
)

// Fetch version
const { data: version } = useQuery({
	queryKey: computed(() => ['version', versionId.value]),
	queryFn: () => client.labrinth.versions_v2.getVersion(versionId.value),
	enabled: computed(() => !!versionId.value),
})

// Project ID
const projectId = computed(() => {
	if (version.value) return version.value.project_id
	if (rawReport.value?.item_type === 'project') return rawReport.value.item_id
	return null
})

// Fetch project
const { data: project } = useQuery({
	queryKey: computed(() => ['project', projectId.value]),
	queryFn: () => client.labrinth.projects_v2.get(projectId.value),
	enabled: computed(() => !!projectId.value),
})

// Assemble the full report object
const report = computed(() => {
	if (!rawReport.value) return null
	return {
		...rawReport.value,
		project: project.value ?? null,
		version: version.value ?? null,
		reporterUser: (users.value || []).find((user) => user.id === rawReport.value.reporter),
		user:
			rawReport.value.item_type === 'user'
				? (users.value || []).find((user) => user.id === rawReport.value.item_id)
				: undefined,
	}
})

// Fetch thread
const { data: rawThread } = useQuery({
	queryKey: computed(() => ['thread', report.value?.thread_id]),
	queryFn: () => client.labrinth.threads_v3.getThread(report.value.thread_id),
	enabled: computed(() => !!report.value?.thread_id),
})

const thread = computed(() =>
	rawThread.value && report.value ? addReportMessage(rawThread.value, report.value) : null,
)

async function updateThread(newThread) {
	queryClient.setQueryData(['thread', report.value?.thread_id], newThread)
	await queryClient.invalidateQueries({ queryKey: ['report', props.reportId] })
}
</script>
<style lang="scss" scoped>
.stacked {
	display: flex;
	flex-direction: column;
}
</style>
