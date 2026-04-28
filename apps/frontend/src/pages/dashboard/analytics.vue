<template>
	<AnalyticsDashboard />
</template>

<script setup>
import { commonProjectSettingsMessages, useVIntl } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics/AnalyticsDashboard.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonProjectSettingsMessages.analytics)} - Modrinth`,
})
</script>
