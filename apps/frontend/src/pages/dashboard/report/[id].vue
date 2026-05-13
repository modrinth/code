<template>
	<ReportView
		:auth="auth"
		:report-id="route.params.id"
		:breadcrumbs-stack="[
			{ href: '/dashboard/reports', label: formatMessage(messages.activeReportsTitle) },
		]"
	/>
</template>
<script setup>
import { defineMessages, useVIntl } from '@modrinth/ui'

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

const route = useNativeRoute()
const auth = await useAuth()

useHead({
	title: () => `${formatMessage(messages.reportTitle, { id: route.params.id })} - Modrinth`,
})
</script>
