<template>
	<footer class="shrink-0 border-0 border-t border-solid border-divider">
		<div v-if="navigationError" role="alert" class="px-3 py-2 text-xs text-red">
			{{ formatMessage(messages.queueError) }}
		</div>
		<nav
			class="flex min-h-10 flex-wrap items-center justify-between gap-x-4 gap-y-1 px-3 py-1.5 text-xs text-secondary"
			:aria-label="formatMessage(messages.queueNavigation)"
			:aria-busy="busy"
		>
			<div
				class="flex items-center gap-3 text-[0.6875rem] font-bold uppercase tracking-widest"
				aria-live="polite"
			>
				<span
					>{{ formatMessage(messages.complete) }}
					<span class="ml-1 tabular-nums text-primary">{{
						queue.currentQueue.completed.length
					}}</span></span
				>
				<span class="h-3 w-px bg-divider" aria-hidden="true" />
				<span
					>{{ formatMessage(messages.total) }}
					<span class="ml-1 tabular-nums text-primary">{{ queue.currentQueue.total }}</span></span
				>
			</div>
			<div class="flex items-center gap-3">
				<button type="button" class="queue-action" :disabled="disabled || !canGoBack" @click="back">
					<LeftArrowIcon />{{ formatMessage(messages.back) }}
				</button>
				<button
					type="button"
					class="queue-action"
					:disabled="disabled || (!inQueue && !completed)"
					@click="next"
				>
					<RightArrowIcon />{{
						formatMessage(
							completed ? (remaining.length ? messages.next : messages.finish) : messages.skip,
						)
					}}
				</button>
				<span class="h-3 w-px bg-divider" aria-hidden="true" />
				<button type="button" class="queue-action" disabled>
					<UndoIcon />{{ formatMessage(messages.reset) }}
				</button>
				<button type="button" class="queue-action" :disabled="busy" @click="exit">
					<LogOutIcon />{{ formatMessage(messages.exit) }}
				</button>
			</div>
		</nav>
	</footer>
</template>

<script setup lang="ts">
import { LeftArrowIcon, LogOutIcon, RightArrowIcon, UndoIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from './messages'

const { queue, isLoading, navigation } = injectProjectReviewPageContext()
const {
	busy,
	error: navigationError,
	canGoBack,
	inQueue,
	completed,
	remaining,
	back,
	next,
	exit,
} = navigation
const { formatMessage } = useVIntl()
const disabled = computed(() => busy.value || isLoading.value || !queue.hydrated)
</script>

<style scoped>
.queue-action {
	display: inline-flex;
	align-items: center;
	gap: 0.375rem;
	padding: 0.25rem 0;
	background: transparent;
	color: inherit;
	font: inherit;
	font-weight: 600;
}
.queue-action:hover:not(:disabled) {
	color: var(--color-contrast);
}
.queue-action:disabled {
	cursor: default;
	opacity: 0.4;
}
.queue-action :deep(svg) {
	width: 0.875rem;
	height: 0.875rem;
}
</style>
