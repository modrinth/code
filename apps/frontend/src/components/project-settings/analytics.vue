<template>
	<AnalyticsDashboard>
		<template #title>
			<SettingsSection :title="formatMessage(commonProjectSettingsMessages.analytics)" />
		</template>
	</AnalyticsDashboard>
</template>

<script setup lang="ts">
import { commonProjectSettingsMessages, useVIntl } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

import SettingsSection from './settings-section.vue'

const { formatMessage } = useVIntl()
</script>
