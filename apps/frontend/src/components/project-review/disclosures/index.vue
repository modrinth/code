<template>
	<ReviewPanel
		mode="anchored"
		:target="{ kind: 'disclosures' }"
		:label="formatMessage(messages.disclosures)"
	>
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.disclosures) }}
		</h2>
	</ReviewPanel>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'

const { formatMessage } = useVIntl()
</script>
