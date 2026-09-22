<template>
	<h2 class="m-0 text-lg font-semibold text-contrast">
		{{ formatMessage(messages.history) }}
	</h2>
	<p class="text-secondary">{{ formatMessage(messages.historyStub) }}</p>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'

const { formatMessage } = useVIntl()
</script>
