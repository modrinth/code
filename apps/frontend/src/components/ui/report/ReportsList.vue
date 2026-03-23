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
	<p v-if="filteredReports.length === 0">You don't have any active reports.</p>
</template>
<script setup>
import { Chips, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ReportInfo from '~/components/ui/report/ReportInfo.vue'
import { addReportMessage } from '~/helpers/threads.js'
import { fetchSegmentedWith } from '~/utils/fetch-helpers.ts'

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

const client = injectModrinthClient()
const viewMode = ref('open')
const reasonFilter = ref('All')

const MAX_REPORTS = 1500

const { data: rawReportsData } = useQuery({
	queryKey: ['reports', MAX_REPORTS],
	queryFn: () => client.labrinth.reports_v3.list({ count: MAX_REPORTS }),
	placeholderData: [],
})

const rawReports = computed(() => rawReportsData.value)

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
	queryFn: () =>
		fetchSegmentedWith(userIds.value, (ids) => client.labrinth.users_v2.getMultiple(ids)),
	enabled: computed(() => userIds.value.length > 0),
	placeholderData: [],
})

const { data: versions } = useQuery({
	queryKey: computed(() => ['versions', versionIds.value]),
	queryFn: () =>
		fetchSegmentedWith(versionIds.value, (ids) => client.labrinth.versions_v2.getVersions(ids)),
	enabled: computed(() => versionIds.value.length > 0),
	placeholderData: [],
})

const { data: threads } = useQuery({
	queryKey: computed(() => ['threads', threadIds.value]),
	queryFn: () =>
		fetchSegmentedWith(threadIds.value, (ids) => client.labrinth.threads_v3.getMultiple(ids)),
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
		fetchSegmentedWith(projectIds.value, (ids) => client.labrinth.projects_v2.getMultiple(ids)),
	enabled: computed(() => projectIds.value.length > 0),
	placeholderData: [],
})

const userMap = computed(() => new Map(users.value.map((u) => [u.id, u])))
const versionMap = computed(() => new Map(versions.value.map((v) => [v.id, v])))
const projectMap = computed(() => new Map(projects.value.map((p) => [p.id, p])))
const threadMap = computed(() => new Map(threads.value.map((t) => [t.id, t])))

const reports = computed(() =>
	rawReports.value.map((report) => {
		const enrichedReport = { ...report }
		enrichedReport.reporterUser = userMap.value.get(report.reporter)
		if (report.item_type === 'user') {
			enrichedReport.user = userMap.value.get(report.item_id)
		} else if (report.item_type === 'project') {
			enrichedReport.project = projectMap.value.get(report.item_id)
		} else if (report.item_type === 'version') {
			enrichedReport.version = versionMap.value.get(report.item_id)
			enrichedReport.project = projectMap.value.get(enrichedReport.version?.project_id)
		}
		if (report.thread_id) {
			const thread = threadMap.value.get(report.thread_id)
			enrichedReport.thread = thread ? addReportMessage(thread, enrichedReport) : null
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
