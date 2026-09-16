<template>
	<h2 class="m-0 text-lg font-semibold text-contrast">
		{{ formatMessage(messages.disclosures) }}
	</h2>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'

const { formatMessage } = useVIntl()
</script>
