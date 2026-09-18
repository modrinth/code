<template>
	<section class="flex flex-col gap-3">
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.permissions) }}
		</h2>
		<p class="m-0 text-secondary">{{ formatMessage(messages.permissionsPlaceholder) }}</p>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'

const { formatMessage } = useVIntl()
</script>
