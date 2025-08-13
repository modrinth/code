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

import ReportInfo from '~/components/ui/report/ReportInfo.vue'
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
const reports = ref([])

const MAX_REPORTS = 1500

let { data: rawReports } = await useAsyncData('report', () =>
	useBaseFetch(`report?count=${MAX_REPORTS}`),
)

rawReports = rawReports.value.map((report) => {
	report.item_id = report.item_id.replace(/"/g, '')
	return report
})

const reporterUsers = rawReports.map((report) => report.reporter)
const reportedUsers = rawReports
	.filter((report) => report.item_type === 'user')
	.map((report) => report.item_id)
const versionReports = rawReports.filter((report) => report.item_type === 'version')
const versionIds = [...new Set(versionReports.map((report) => report.item_id))]
const userIds = [...new Set(reporterUsers.concat(reportedUsers))]
const threadIds = [
	...new Set(rawReports.filter((report) => report.thread_id).map((report) => report.thread_id)),
]
const reasons = ['All', ...new Set(rawReports.map((report) => report.report_type))]

const [{ data: users }, { data: versions }, { data: threads }] = await Promise.all([
	await useAsyncData(`users?ids=${JSON.stringify(userIds)}`, () =>
		fetchSegmented(userIds, (ids) => `users?ids=${asEncodedJsonArray(ids)}`),
	),
	await useAsyncData(`versions?ids=${JSON.stringify(versionIds)}`, () =>
		fetchSegmented(versionIds, (ids) => `versions?ids=${asEncodedJsonArray(ids)}`),
	),
	await useAsyncData(`threads?ids=${JSON.stringify(threadIds)}`, () =>
		fetchSegmented(threadIds, (ids) => `threads?ids=${asEncodedJsonArray(ids)}`),
	),
])

const reportedProjects = rawReports
	.filter((report) => report.item_type === 'project')
	.map((report) => report.item_id)
const versionProjects = versions.value.map((version) => version.project_id)
const projectIds = [...new Set(reportedProjects.concat(versionProjects))]

const { data: projects } = await useAsyncData(`projects?ids=${JSON.stringify(projectIds)}`, () =>
	fetchSegmented(projectIds, (ids) => `projects?ids=${asEncodedJsonArray(ids)}`),
)

reports.value = rawReports.map((report) => {
	report.reporterUser = users.value.find((user) => user.id === report.reporter)
	if (report.item_type === 'user') {
		report.user = users.value.find((user) => user.id === report.item_id)
	} else if (report.item_type === 'project') {
		report.project = projects.value.find((project) => project.id === report.item_id)
	} else if (report.item_type === 'version') {
		report.version = versions.value.find((version) => version.id === report.item_id)
		report.project = projects.value.find((project) => project.id === report.version.project_id)
	}
	if (report.thread_id) {
		report.thread = addReportMessage(
			threads.value.find((thread) => report.thread_id === thread.id),
			report,
		)
	}
	report.open = true
	return report
})

const filteredReports = computed(() =>
	reports.value?.filter(
		(x) =>
			(props.moderation || x.reporterUser.id === props.auth.user.id) &&
			(viewMode.value === 'open' ? x.open : !x.open) &&
			(reasonFilter.value === 'All' || reasonFilter.value === x.report_type),
	),
)
</script>
