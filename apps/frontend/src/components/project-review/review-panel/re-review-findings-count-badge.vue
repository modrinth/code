<template>
	<span
		v-if="count > 0"
		class="pointer-events-none flex h-[19px] min-w-[18px] shrink-0 items-center justify-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-3 px-1.5 text-xs font-medium leading-none text-primary"
	>
		<TagCategoryRefreshCcwIcon class="size-3 shrink-0 text-orange" aria-hidden="true" />
		<span aria-hidden="true">{{ count }}</span>
		<span class="sr-only">{{ formatMessage(messages.reReviewFindings, { count }) }}</span>
	</span>
</template>

<script setup lang="ts">
import { TagCategoryRefreshCcwIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'

defineProps<{ count: number }>()
const { formatMessage } = useVIntl()
</script>
