<template>
	<AnalyticsDashboard />
</template>

<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.analytics)
</script>
