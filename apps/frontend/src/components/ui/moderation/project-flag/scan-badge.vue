<template>
	<TagItem v-tooltip="tooltip" tabindex="0">
		<template v-if="typeof run.status === 'object'">
			<CircleAlertIcon v-if="numWorkers === 0" class="text-orange" aria-hidden="true" />
			<LoaderCircleIcon v-else class="animate-spin" aria-hidden="true" />
			{{ progress }}% processed
		</template>
		<template v-else>{{ formatRunStatus(run.status) }}</template>
	</TagItem>
</template>

<script setup lang="ts">
import type { ProjectFlagRun } from '@modrinth/api-client'
import { CircleAlertIcon, LoaderCircleIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'

import { formatDuration, formatRunStatus } from './run-format'

const props = defineProps<{
	run: Pick<
		ProjectFlagRun,
		'status' | 'started_at' | 'completed_at' | 'total_projects' | 'num_scanned'
	>
	numWorkers?: number
}>()

const progress = computed(() =>
	props.run.total_projects === 0
		? 0
		: Math.min(100, Math.floor((props.run.num_scanned / props.run.total_projects) * 100)),
)
const tooltip = computed(() => {
	const { run, numWorkers } = props
	const scanned = run.num_scanned.toLocaleString('en-US')
	const total = run.total_projects.toLocaleString('en-US')
	const progress = `${scanned}/${total} projects processed`
	if (typeof run.status === 'string') {
		return run.completed_at
			? `${progress} in ${formatDuration(run.started_at, run.completed_at, true)}`
			: progress
	}
	const workers =
		numWorkers === undefined
			? 'Worker count unavailable'
			: `${numWorkers.toLocaleString('en-US')} ${numWorkers === 1 ? 'worker' : 'workers'} recently polled`
	return `${progress} · ${formatRunStatus(run.status)} · ${workers}`
})
</script>
