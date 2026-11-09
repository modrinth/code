<template>
	<div>
		<h2 class="settings-section-title">
			{{ formatMessage(commonProjectSettingsMessages.analytics) }}
		</h2>
		<AnalyticsDashboard />
	</div>
</template>

<script setup lang="ts">
import { commonProjectSettingsMessages, useVIntl } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

const { formatMessage } = useVIntl()

useProjectSettingsHeadTitle(commonProjectSettingsMessages.analytics)
</script>
