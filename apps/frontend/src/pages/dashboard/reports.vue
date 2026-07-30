<template>
	<div class="flex flex-col gap-4 pb-20 lg:pl-4 lg:pt-1.5">
		<h2 class="m-0 text-xl font-semibold text-contrast md:text-2xl">
			{{ formatMessage(messages.reportsTitle) }}
		</h2>
		<ReportsList :auth="auth" />
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
