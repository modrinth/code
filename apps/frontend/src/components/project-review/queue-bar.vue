<template>
	<div class="flex min-w-0 flex-1 flex-col text-sm">
		<div v-if="navigationError" role="alert" class="px-1 py-1 text-red">
			Could not change projects. Please try again
		</div>
		<nav
			class="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 text-secondary"
			aria-label="Review queue navigation"
			:aria-busy="busy"
		>
			<div class="flex items-center gap-3 font-medium" aria-live="polite">
				<button
					v-tooltip="{ text: 'Settings', delay: 500 }"
					type="button"
					class="queue-action"
					aria-label="Settings"
					@click="toggleSettings"
				>
					<SettingsIcon aria-hidden="true" />
				</button>

				<div class="flex items-center gap-2">
					<template v-for="(column, index) in queueColumns" :key="column.label">
						<span v-if="index" class="h-5 w-px bg-divider" aria-hidden="true" />
						<FloatingMenu
							trigger="hover"
							placement="top-start"
							:flip="false"
							theme="tooltip"
							:distance="4"
						>
							<span tabindex="0" class="cursor-default pb-1 pt-1.5 text-xs">
								{{ column.label }}
								<span class="ml-1 tabular-nums text-primary">{{ column.count }}</span>
							</span>
							<template #popper>
								<div class="flex max-w-80 flex-col gap-2 py-1">
									<span class="font-semibold">{{ column.heading }}</span>
									<QueueProjectList :project-ids="column.projects" :empty-message="column.empty" />
								</div>
							</template>
						</FloatingMenu>
					</template>
				</div>
			</div>
			<div class="flex items-center gap-1">
				<button type="button" class="queue-action" :disabled="busy" @click="exit">Exit</button>
				<span class="mx-1.5 h-5 w-px bg-divider" aria-hidden="true" />
				<button type="button" class="queue-action" :disabled="disabled || !canGoBack" @click="back">
					<LeftArrowIcon class="mb-px" />
					Back
				</button>
				<button
					type="button"
					class="queue-action"
					:disabled="disabled || (!inQueue && !completed)"
					@click="next"
				>
					{{ completed ? (remaining.length ? 'Next' : 'Finish') : 'Skip' }}
					<RightArrowIcon class="mb-px" />
				</button>
			</div>
		</nav>
	</div>
</template>

<script setup lang="ts">
import { LeftArrowIcon, RightArrowIcon, SettingsIcon } from '@modrinth/assets'
import { FloatingMenu } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { injectProjectReviewContext } from './layout/context'
import QueueProjectList from './queue-project-list.vue'
import { useReviewShortcut } from './shortcuts'

const { toggleSettings } = injectProjectReviewContext()

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

const disabled = computed(() => busy.value || isLoading.value || !queue.hydrated)
const queueColumns = computed(() => {
	const current = queue.currentQueue
	const completedIds = new Set(current.completed)
	const skippedIds = new Set(current.skipped)
	return [
		{
			label: 'Complete',
			heading: 'Completed projects',
			empty: 'No completed projects yet',
			count: current.completed.length,
			projects: current.projectOrder.filter((id) => completedIds.has(id)),
		},
		{
			label: 'Skipped',
			heading: 'Skipped projects',
			empty: 'No skipped projects',
			count: current.skipped.length,
			projects: current.projectOrder.filter((id) => skippedIds.has(id)),
		},
		{
			label: 'Remaining',
			heading: 'Next 5 projects',
			empty: 'No projects in the queue',
			count: current.items.length,
			projects: remaining.value.slice(0, 5),
		},
	]
})

useReviewShortcut(
	'back',
	() => {
		void back()
	},
	() => !disabled.value && canGoBack.value,
)
useReviewShortcut(
	'next',
	() => {
		void next()
	},
	() => !disabled.value && (inQueue.value || completed.value),
)
useReviewShortcut(
	'exit',
	() => {
		void exit()
	},
	() => !busy.value,
)
</script>

<style scoped>
.queue-action {
	display: inline-flex;
	align-items: center;
	gap: 0.25rem;
	padding: 0.375rem;
	background: transparent;
	color: inherit;
	font: inherit;
	font-size: inherit;
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
	width: 1rem;
	height: 1rem;
}
</style>
