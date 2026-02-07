<template>
	<template v-if="moderation">
		<Chips v-model="reasonFilter" :items="reasons" />
		<p v-if="reports.length === MAX_REPORTS" class="text-red">
			There are at least {{ MAX_REPORTS }} open reports. This page is at its max reports and will
			not show any more recent ones.
		</p>
		<p v-else-if="reasonFilter === 'All'">There are {{ filteredReports.length }} open reports.</p>
		<p v-else>
			There are {{ filteredReports.length }}/{{ reports.length }} open '{{ reasonFilter }}' reports.
		</p>
	</template>
	<ReportInfo
		v-for="report in filteredReports"
		:key="report.id"
		:report="report"
		:thread="report.thread"
		:show-message="false"
		:moderation="moderation"
		raised
		:auth="auth"
		class="universal-card recessed"
	/>
	<p v-if="reports.length === 0">You don't have any active reports.</p>
</template>
<script setup>
import { Chips } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ReportInfo from '~/components/ui/report/ReportInfo.vue'
import { useBaseFetch } from '~/composables/fetch.js'
import { addReportMessage } from '~/helpers/threads.js'
import { asEncodedJsonArray, fetchSegmented } from '~/utils/fetch-helpers.ts'

const props = defineProps({
	moderation: {
		type: Boolean,
		default: false,
	},
	auth: {
		type: Object,
		required: true,
	},
})

const viewMode = ref('open')
const reasonFilter = ref('All')

const MAX_REPORTS = 1500

const { data: rawReportsData } = useQuery({
	queryKey: ['reports', MAX_REPORTS],
	queryFn: () => useBaseFetch(`report?count=${MAX_REPORTS}`),
	placeholderData: [],
})

const rawReports = computed(() =>
	rawReportsData.value.map((report) => ({
		...report,
		item_id: report.item_id.replace(/"/g, ''),
	})),
)

const reporterUsers = computed(() => rawReports.value.map((report) => report.reporter))
const reportedUsers = computed(() =>
	rawReports.value.filter((report) => report.item_type === 'user').map((report) => report.item_id),
)
const versionReports = computed(() =>
	rawReports.value.filter((report) => report.item_type === 'version'),
)
const versionIds = computed(() => [
	...new Set(versionReports.value.map((report) => report.item_id)),
])
const userIds = computed(() => [...new Set(reporterUsers.value.concat(reportedUsers.value))])
const threadIds = computed(() => [
	...new Set(
		rawReports.value.filter((report) => report.thread_id).map((report) => report.thread_id),
	),
])
const reasons = computed(() => [
	'All',
	...new Set(rawReports.value.map((report) => report.report_type)),
])

const { data: users } = useQuery({
	queryKey: computed(() => ['users', userIds.value]),
	queryFn: () => fetchSegmented(userIds.value, (ids) => `users?ids=${asEncodedJsonArray(ids)}`),
	enabled: computed(() => userIds.value.length > 0),
	placeholderData: [],
})

const { data: versions } = useQuery({
	queryKey: computed(() => ['versions', versionIds.value]),
	queryFn: () =>
		fetchSegmented(versionIds.value, (ids) => `versions?ids=${asEncodedJsonArray(ids)}`),
	enabled: computed(() => versionIds.value.length > 0),
	placeholderData: [],
})

const { data: threads } = useQuery({
	queryKey: computed(() => ['threads', threadIds.value]),
	queryFn: () => fetchSegmented(threadIds.value, (ids) => `threads?ids=${asEncodedJsonArray(ids)}`),
	enabled: computed(() => threadIds.value.length > 0),
	placeholderData: [],
})

const reportedProjects = computed(() =>
	rawReports.value
		.filter((report) => report.item_type === 'project')
		.map((report) => report.item_id),
)
const versionProjects = computed(() => versions.value.map((version) => version.project_id))
const projectIds = computed(() => [
	...new Set(reportedProjects.value.concat(versionProjects.value)),
])

const { data: projects } = useQuery({
	queryKey: computed(() => ['projects', projectIds.value]),
	queryFn: () =>
		fetchSegmented(projectIds.value, (ids) => `projects?ids=${asEncodedJsonArray(ids)}`),
	enabled: computed(() => projectIds.value.length > 0),
	placeholderData: [],
})

const reports = computed(() =>
	rawReports.value.map((report) => {
		const enrichedReport = { ...report }
		enrichedReport.reporterUser = users.value.find((user) => user.id === report.reporter)
		if (report.item_type === 'user') {
			enrichedReport.user = users.value.find((user) => user.id === report.item_id)
		} else if (report.item_type === 'project') {
			enrichedReport.project = projects.value.find((project) => project.id === report.item_id)
		} else if (report.item_type === 'version') {
			enrichedReport.version = versions.value.find((version) => version.id === report.item_id)
			enrichedReport.project = projects.value.find(
				(project) => project.id === enrichedReport.version?.project_id,
			)
		}
		if (report.thread_id) {
			const thread = threads.value.find((thread) => report.thread_id === thread.id)
			enrichedReport.thread = thread ? addReportMessage(thread, report) : null
		}
		enrichedReport.open = true
		return enrichedReport
	}),
)

const filteredReports = computed(() =>
	reports.value?.filter(
		(x) =>
			(props.moderation || x.reporterUser?.id === props.auth.user.id) &&
			(viewMode.value === 'open' ? x.open : !x.open) &&
			(reasonFilter.value === 'All' || reasonFilter.value === x.report_type),
	),
)
</script>
