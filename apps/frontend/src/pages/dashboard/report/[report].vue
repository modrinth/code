<template>
	<ReportView
		:auth="auth"
		:report-id="reportId"
		:breadcrumbs-stack="[
			{ href: '/dashboard/reports', label: formatMessage(messages.activeReportsTitle) },
		]"
	/>
</template>
<script setup>
import { defineMessages, useVIntl } from '@icarus/ui'

import ReportView from '~/components/ui/report/ReportView.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	activeReportsTitle: {
		id: 'dashboard.reports.active-title',
		defaultMessage: 'Active reports',
	},
	reportTitle: {
		id: 'dashboard.report.title',
		defaultMessage: 'Report {id}',
	},
})

const auth = await useAuth()
const reportId = useRouteId('report')

useHead({
	title: () => `${formatMessage(messages.reportTitle, { id: reportId })} - Modrinth`,
})
</script>
