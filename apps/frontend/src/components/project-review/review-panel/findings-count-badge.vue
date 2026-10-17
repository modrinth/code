<template>
	<span
		v-if="count > 0"
		class="pointer-events-none flex h-[18px] min-w-[18px] shrink-0 items-center justify-center rounded-full border border-solid border-highlight-orange bg-highlight-orange px-1 text-xs font-medium leading-none text-orange"
	>
		<span aria-hidden="true">{{ count }}</span>
		<span class="sr-only">{{ formatMessage(messages.selectedFindings, { count }) }}</span>
	</span>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'

defineProps<{ count: number }>()
const { formatMessage } = useVIntl()
</script>
