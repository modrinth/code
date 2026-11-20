<template>
	<TagItem v-tooltip="tooltip" tabindex="0">
		<template v-if="run.status !== 'done'">
			<CircleAlertIcon v-if="numConsumers === 0" class="text-orange" aria-hidden="true" />
			<LoaderCircleIcon v-else class="animate-spin" aria-hidden="true" />
			{{ progress === undefined ? 'Scanning…' : `${progress}% scanned` }}
		</template>
		<template v-else>{{ duration ?? 'Done' }}</template>
	</TagItem>
</template>

<script setup lang="ts">
import type { ProjectFlagRun } from '@modrinth/api-client'
import { CircleAlertIcon, LoaderCircleIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'

import { formatDuration } from './run-format'

const props = defineProps<{
	run: Pick<ProjectFlagRun, 'status' | 'started_at' | 'completed_at' | 'total_projects'>
	scanned?: number
	numConsumers?: number
}>()

const duration = computed(() => {
	const { started_at, completed_at } = props.run
	return completed_at ? formatDuration(started_at, completed_at) : undefined
})
const progress = computed(() => {
	if (props.scanned === undefined) return
	if (props.run.total_projects === 0) return 0
	return Math.min(100, Math.floor((props.scanned / props.run.total_projects) * 100))
})
const tooltip = computed(() => {
	const { run, scanned, numConsumers } = props
	const scannedText = scanned?.toLocaleString('en-US') ?? '—'

	if (run.status === 'done') {
		if (scanned === undefined) return 'Scanned count unavailable'
		const projects = scanned === 1 ? 'project' : 'projects'
		const elapsed = run.completed_at
			? ` in ${formatDuration(run.started_at, run.completed_at, true)}`
			: ''
		return `${scannedText} ${projects} scanned${elapsed}`
	}

	const total = run.total_projects.toLocaleString('en-US')
	const consumers =
		numConsumers === undefined
			? 'Consumer count unavailable'
			: `${numConsumers.toLocaleString('en-US')} ${numConsumers === 1 ? 'consumer' : 'consumers'} scanning`
	return `${scannedText}/${total} projects scanned · ${consumers}`
})
</script>
