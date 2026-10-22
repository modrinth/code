<template>
	<div class="flex h-full min-h-0 min-w-0 flex-col">
		<slot />
		<ReviewOutcomeButtons v-if="hostsOutcome" class="mt-2.5" />
	</div>
</template>

<script setup lang="ts">
import { watch } from 'vue'

import { injectReviewPanelVisible } from './layout/context'
import ReviewOutcomeButtons from './review-outcome-buttons/index.vue'

/**
 * One of the review's conversation panels (re-review, thread, issues), which reports whether it's
 * shown and, when it's the one picked to, hosts the review's outcome buttons below its content.
 */
defineProps<{
	hostsOutcome: boolean
}>()

const emit = defineEmits<{
	shown: [shown: boolean]
}>()

const visible = injectReviewPanelVisible()
watch(visible, (shown) => emit('shown', shown), { immediate: true })
</script>
