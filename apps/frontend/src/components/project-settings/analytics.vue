<template>
	<AnalyticsDashboard>
		<template #title>
			<h2 class="settings-section-title !mb-0">
				{{ formatMessage(commonProjectSettingsMessages.analytics) }}
			</h2>
		</template>
	</AnalyticsDashboard>
</template>

<script setup lang="ts">
import { commonProjectSettingsMessages, useVIntl } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

const { formatMessage } = useVIntl()
</script>
