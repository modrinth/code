<template>
	<div>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(messages.reportsTitle) }}</h2>
			<ReportsList :auth="auth" />
		</section>
	</div>
</template>
<script setup>
import { defineMessages, useVIntl } from '@modrinth/ui'

import ReportsList from '~/components/ui/report/ReportsList.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	reportsTitle: {
		id: 'dashboard.reports.title',
		defaultMessage: 'Reports',
	},
	activeReportsTitle: {
		id: 'dashboard.reports.active-title',
		defaultMessage: 'Active reports',
	},
})

const auth = await useAuth()
useHead({
	title: () => `${formatMessage(messages.activeReportsTitle)} - Modrinth`,
})
</script>
