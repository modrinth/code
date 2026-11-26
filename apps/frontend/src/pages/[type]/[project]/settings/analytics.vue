<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import AnalyticsSettings from '~/components/project-settings/analytics.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.analytics)
</script>

<template>
	<AnalyticsSettings />
</template>
