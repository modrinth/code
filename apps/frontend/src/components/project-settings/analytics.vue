<template>
	<AnalyticsDashboard class="!gap-8">
		<template #title>
			<h2 class="m-0 text-xl font-semibold leading-7 text-contrast">
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
