<template>
	<AnalyticsDashboard class="lg:pl-4 lg:pt-1.5" />
</template>

<script setup>
import { commonProjectSettingsMessages, useVIntl } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonProjectSettingsMessages.analytics)} - Modrinth`,
})
</script>
