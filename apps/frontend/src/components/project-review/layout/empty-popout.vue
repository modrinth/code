<template>
	<div class="flex h-full items-center justify-center p-3 text-secondary">
		<span v-if="params.group?.api.location.type === 'popout'" role="status">
			{{ formatMessage(projectReviewMessages.emptyPopout) }}
		</span>
	</div>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import type { WatermarkRendererInitParameters } from 'dockview-vue'

import { projectReviewMessages } from '../messages'

defineProps<{ params: WatermarkRendererInitParameters }>()
const { formatMessage } = useVIntl()
</script>
