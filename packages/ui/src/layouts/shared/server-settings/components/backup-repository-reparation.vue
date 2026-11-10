<template>
	<div class="flex flex-col gap-4">
		<Teleport to="body">
			<div class="relative z-[100]">
				<ConfirmModal
					ref="confirmOperationModal"
					:title="confirmTitle"
					:description="confirmDescription"
					:proceed-label="confirmTitle"
					:proceed-icon="pendingOperation ? operationIcons[pendingOperation] : undefined"
					:markdown="false"
					@proceed="confirmOperation"
				/>
			</div>
		</Teleport>

		<div class="flex flex-col gap-1">
			<span class="text-lg font-semibold text-contrast">Backup repository reparation</span>
			<span class="text-primary">
				Check a backup repository for damage and repair it. Backups of the repository are paused
				while an operation runs.
			</span>
		</div>

		<span v-if="serverQuery.isPending.value" class="text-secondary">
			Loading backup repositories…
		</span>
		<span v-else-if="repositoryIds.length === 0" class="text-secondary">
			This server has no backup repositories.
		</span>
		<template v-else>
			<div v-if="repositoryIds.length > 1" class="max-w-xs">
				<Combobox
					v-model="selectedRepositoryId"
					name="backup-repository"
					:options="repositoryOptions"
					:display-value="repositoryLabel(selectedRepositoryId)"
				/>
			</div>
			<span v-else class="text-secondary">{{ repositoryLabel(selectedRepositoryId) }}</span>

			<Admonition
				v-if="activeOperation"
				type="info"
				waiting
				:header="`${operationLabels[activeOperation.operation_type]} is ${statusLabels[activeOperation.status]}`"
				:timestamp="activeOperation.started_at ?? activeOperation.created_at"
			>
				Other operations can start once it finishes. This page refreshes automatically.
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">Check</span>
				<Checkbox
					v-model="readData"
					label="Read all pack data (very slow)"
					description="Not recommended. Downloads and verifies every pack, which can take hours on large repositories. Only use it when a regular check passes but restores still fail."
					:disabled="!!activeOperation"
				/>
				<div>
					<Button
						v-tooltip="activeOperation ? operationRunningMessage : undefined"
						type="colored"
						color="brand"
						:disabled="!!activeOperation || isStarting"
						:loading="isStarting && pendingOperation === null"
						@click="startCheck"
					>
						<SearchIcon class="size-5" />
						{{ latestRun ? 'Run new check' : 'Run check' }}
					</Button>
				</div>
			</div>

			<div v-if="checkOutput" class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
				<span class="font-semibold text-contrast">Check results</span>
				<div class="flex flex-wrap gap-6">
					<div class="flex flex-col">
						<span class="text-secondary">Errors</span>
						<span class="text-2xl font-semibold text-contrast">{{ checkOutput.num_errors }}</span>
					</div>
					<div class="flex flex-col">
						<span class="text-secondary">Broken packs</span>
						<span class="text-2xl font-semibold text-contrast">{{
							checkOutput.broken_packs.length
						}}</span>
					</div>
					<div class="flex flex-col">
						<span class="text-secondary">Missing pack errors</span>
						<span class="text-2xl font-semibold text-contrast">{{ missingPackErrors }}</span>
					</div>
				</div>
				<div class="flex flex-col gap-1">
					<span class="text-secondary">Recommended action</span>
					<ul class="m-0 flex flex-col gap-1 pl-5 text-primary">
						<li v-for="recommendation in recommendations" :key="recommendation">
							{{ recommendation }}
						</li>
					</ul>
				</div>
				<details v-if="checkOutput.errors.length > 0">
					<summary class="cursor-pointer text-secondary">
						{{ checkOutput.errors_truncated ? 'Show the first' : 'Show' }}
						{{ pluralize(checkOutput.errors.length, 'error') }}
					</summary>
					<pre
						class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-all rounded-xl bg-surface-3 p-3 font-mono text-sm"
						>{{ checkOutput.errors.join('\n') }}</pre
					>
				</details>
			</div>

			<div v-if="latestRun" class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">Repairs</span>
				<div class="flex flex-wrap gap-2">
					<Button
						v-for="operationType in runnableOperations"
						:key="operationType"
						v-tooltip="operationBlockers[operationType] ?? undefined"
						:disabled="!!operationBlockers[operationType] || isStarting"
						:loading="isStarting && pendingOperation === operationType"
						@click="askOperation(operationType)"
					>
						<component :is="operationIcons[operationType]" class="size-5" />
						{{ operationLabels[operationType] }}
					</Button>
				</div>
			</div>

			<div v-if="latestRun" class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					Run {{ latestRun.id }}, started {{ formatRelativeTime(latestRun.created_at) }}
				</span>
				<div
					v-for="operation in latestRun.operations"
					:key="operation.id"
					class="flex flex-col gap-1.5 rounded-2xl bg-surface-2 p-4"
				>
					<div class="flex flex-wrap items-center gap-2">
						<component :is="operationIcons[operation.operation_type]" class="size-5" />
						<span class="font-semibold text-contrast">{{
							operationLabels[operation.operation_type]
						}}</span>
						<SimpleBadge
							:formatted-name="statusLabels[operation.status]"
							:color="statusColors[operation.status]"
						/>
						<span class="text-secondary">{{ operationTiming(operation) }}</span>
					</div>
					<span v-if="operationSummary(operation)" class="text-primary">{{
						operationSummary(operation)
					}}</span>
					<span v-if="operation.error" class="break-all font-mono text-sm text-red">{{
						operation.error
					}}</span>
					<details v-if="operation.results">
						<summary class="cursor-pointer text-secondary">Raw results</summary>
						<pre
							class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-all rounded-xl bg-surface-3 p-3 font-mono text-sm"
							>{{ JSON.stringify(operation.results, null, 2) }}</pre
						>
					</details>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	BrushCleaningIcon,
	DatabaseIcon,
	HistoryIcon,
	PackageIcon,
	SearchIcon,
	UndoIcon,
} from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Component, computed, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import Combobox from '#ui/components/base/Combobox.vue'
import SimpleBadge from '#ui/components/base/SimpleBadge.vue'
import { ConfirmModal } from '#ui/components/modal'
import { useRelativeTime } from '#ui/composables/how-ago'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'

type OperationType = Archon.BackupRepositories.Internal.OperationType
type RunnableOperationType = Archon.BackupRepositories.Internal.RunnableOperationType
type OperationStatus = Archon.BackupRepositories.Internal.OperationStatus
type ReparationOperation = Archon.BackupRepositories.Internal.ReparationOperation
type ReparationRun = Archon.BackupRepositories.Internal.ReparationRun
type RepairOutput = Archon.BackupRepositories.Internal.RepairOutput
type RepairSnapshotsResults = Archon.BackupRepositories.Internal.RepairSnapshotsResults
type RestoreHiddenVersionsResults = Archon.BackupRepositories.Internal.RestoreHiddenVersionsResults

const props = defineProps<{
	serverId: string
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()

const operationRunningMessage = 'Another operation is running on this repository'

const operationLabels: Record<OperationType, string> = {
	check: 'Check',
	'check-read-data': 'Check with data',
	'repair-index': 'Repair index',
	'repair-packs': 'Repair packs',
	'repair-snapshots': 'Repair snapshots',
	prune: 'Prune',
	'restore-hidden-versions': 'Restore hidden versions',
}

const operationIcons: Record<OperationType, Component> = {
	check: SearchIcon,
	'check-read-data': SearchIcon,
	'repair-index': DatabaseIcon,
	'repair-packs': PackageIcon,
	'repair-snapshots': HistoryIcon,
	prune: BrushCleaningIcon,
	'restore-hidden-versions': UndoIcon,
}

const statusLabels: Record<OperationStatus, string> = {
	pending: 'pending',
	ongoing: 'running',
	completed: 'completed',
	failed: 'failed',
}

const statusColors: Record<OperationStatus, 'blue' | 'orange' | 'green' | 'red'> = {
	pending: 'blue',
	ongoing: 'orange',
	completed: 'green',
	failed: 'red',
}

const runnableOperations: RunnableOperationType[] = [
	'repair-index',
	'repair-packs',
	'repair-snapshots',
	'prune',
	'restore-hidden-versions',
]

function pluralize(count: number, noun: string) {
	return `${count} ${noun}${count === 1 ? '' : 's'}`
}

const serverQuery = useQuery({
	queryKey: computed(() => ['servers', 'v1', 'detail', props.serverId]),
	queryFn: () => client.archon.servers_v1.get(props.serverId),
})

const repositoryIds = computed(() =>
	[...(serverQuery.data.value?.repository_ids ?? [])].sort((a, b) => b - a),
)
const selectedRepositoryId = ref<number | null>(null)

watch(
	repositoryIds,
	(ids) => {
		if (selectedRepositoryId.value === null || !ids.includes(selectedRepositoryId.value)) {
			selectedRepositoryId.value = ids[0] ?? null
		}
	},
	{ immediate: true },
)

const repositoryOptions = computed(() =>
	repositoryIds.value.map((id) => ({ value: id, label: repositoryLabel(id) })),
)

function repositoryLabel(id: number | null) {
	return id === null ? '' : `Repository ${id}`
}

const reparationsQueryKey = computed(() => [
	'servers',
	'backup-repositories',
	'reparations',
	selectedRepositoryId.value,
])

const reparationsQuery = useQuery({
	queryKey: reparationsQueryKey,
	queryFn: () =>
		client.archon.backup_repositories_internal.getReparations(selectedRepositoryId.value!),
	enabled: computed(() => selectedRepositoryId.value !== null),
	refetchInterval: (query) => (query.state.data?.some(hasActiveOperation) ? 5000 : false),
})

function hasActiveOperation(run: ReparationRun) {
	return run.operations.some(isActive)
}

function isActive(operation: ReparationOperation) {
	return operation.status === 'pending' || operation.status === 'ongoing'
}

const latestRun = computed(() => reparationsQuery.data.value?.[0] ?? null)

const activeOperation = computed(
	() => reparationsQuery.data.value?.flatMap((run) => run.operations).find(isActive) ?? null,
)

const checkOutput = computed(() => {
	const check = latestRun.value?.operations.find(
		(operation) =>
			(operation.operation_type === 'check' || operation.operation_type === 'check-read-data') &&
			operation.status === 'completed',
	)
	const results = check?.results
	return results && 'type' in results && results.type === 'check' ? results : null
})

const missingPackErrors = computed(
	() =>
		checkOutput.value?.errors.filter((error) =>
			/does not exist|not found|no such file/i.test(error),
		).length ?? 0,
)

const recommendations = computed(() => {
	const output = checkOutput.value
	if (!output) return []

	const result: string[] = []
	if (output.broken_packs.length > 0) {
		result.push('Repair packs, then repair snapshots.')
	}
	if (missingPackErrors.value > 0) {
		result.push(
			'Packs are missing. If the bucket keeps old object versions, restore hidden versions. Otherwise, repair the index, then repair snapshots.',
		)
	} else if (output.suggest_repair_index) {
		result.push('Repair the index, then repair snapshots.')
	}
	if (output.suggest_prune) {
		result.push('Prune to remove unused data.')
	}
	if (result.length === 0) {
		result.push(
			output.num_errors === 0
				? 'None, the repository is healthy.'
				: "The errors don't match a known repair. Review them below.",
		)
	}
	return result
})

function completedInRun(operationType: OperationType) {
	return !!latestRun.value?.operations.some(
		(operation) => operation.operation_type === operationType && operation.status === 'completed',
	)
}

const operationBlockers = computed(() => {
	const blockers = {} as Record<RunnableOperationType, string | null>
	for (const operationType of runnableOperations) {
		blockers[operationType] = operationBlocker(operationType)
	}
	return blockers
})

function operationBlocker(operationType: RunnableOperationType): string | null {
	if (activeOperation.value) return operationRunningMessage

	const output = checkOutput.value
	if (!output) return 'Requires a completed check'

	const existing = latestRun.value?.operations.find(
		(operation) => operation.operation_type === operationType,
	)
	if (existing && existing.status !== 'failed') {
		return 'Already started in this run. Run a new check to repeat it'
	}

	switch (operationType) {
		case 'repair-packs':
			return output.broken_packs.length > 0 ? null : 'The check found no broken packs'
		case 'repair-snapshots':
			return completedInRun('repair-index') || completedInRun('repair-packs')
				? null
				: 'Requires a completed index or pack repair in this run'
		case 'prune':
			return output.suggest_prune || completedInRun('repair-snapshots')
				? null
				: 'Requires the check to suggest pruning, or a completed snapshot repair in this run'
		default:
			return null
	}
}

function operationTiming(operation: ReparationOperation) {
	if (operation.completed_at) {
		return `finished ${formatRelativeTime(operation.completed_at)}`
	}
	if (operation.started_at) {
		return `started ${formatRelativeTime(operation.started_at)}`
	}
	return `queued ${formatRelativeTime(operation.created_at)}`
}

function operationSummary(operation: ReparationOperation): string | null {
	const results = operation.results
	if (!results) return null

	if (operation.operation_type === 'restore-hidden-versions') {
		const restore = results as RestoreHiddenVersionsResults
		if (restore.imported_backups === undefined) {
			return `Restoring into repository ${restore.destination_repository_id}`
		}
		return `${pluralize(restore.imported_backups, 'backup')} restored into repository ${restore.destination_repository_id} (${restore.matched_backups ?? 0} matched to existing backups). ${pluralize(restore.copied_objects ?? 0, 'object')} copied, ${pluralize(restore.missing_objects?.length ?? 0, 'object')} missing.`
	}

	if (operation.operation_type === 'repair-snapshots') {
		const repair = results as RepairSnapshotsResults
		return `${pluralize(repair.remapped_backups.length, 'backup')} moved to repaired snapshots, ${pluralize(repair.missing_snapshot_backups.length, 'backup')} missing their snapshot`
	}

	const output = results as RepairOutput
	if (output.type === 'check') {
		return `${pluralize(output.num_errors, 'error')}, ${pluralize(output.broken_packs.length, 'broken pack')}`
	}
	if (output.type === 'command') {
		return `Restic exited with code ${output.exit_code ?? '?'}`
	}
	return null
}

const readData = ref(false)
const isStarting = ref(false)
const pendingOperation = ref<RunnableOperationType | null>(null)
const confirmOperationModal = ref<InstanceType<typeof ConfirmModal>>()

const confirmTitle = computed(() =>
	pendingOperation.value ? operationLabels[pendingOperation.value] : '',
)

const confirmDescription = computed(() => {
	const operationType = pendingOperation.value
	if (!operationType) return ''

	const id = selectedRepositoryId.value
	if (operationType === 'restore-hidden-versions') {
		return `Copy every retained version of repository ${id}'s objects into a new repository and publish its snapshots as restored backups? The new repository becomes the world's backup target. This can take hours.`
	}
	if (operationType === 'prune') {
		return `Prune repository ${id}? Data that isn't referenced by a snapshot is permanently deleted.`
	}
	return `Run ${operationLabels[operationType]} on repository ${id}? Backups of the repository are paused until it finishes.`
})

function askOperation(operationType: RunnableOperationType) {
	if (operationBlockers.value[operationType]) return
	pendingOperation.value = operationType
	confirmOperationModal.value?.show()
}

async function startCheck() {
	pendingOperation.value = null
	await start(() =>
		client.archon.backup_repositories_internal.startReparation(selectedRepositoryId.value!, {
			read: readData.value,
		}),
	)
}

async function confirmOperation() {
	const operationType = pendingOperation.value
	if (!operationType) return

	await start(() =>
		client.archon.backup_repositories_internal.startOperation(
			selectedRepositoryId.value!,
			operationType,
		),
	)
	pendingOperation.value = null
}

async function start(request: () => Promise<unknown>) {
	if (selectedRepositoryId.value === null || isStarting.value) return

	try {
		isStarting.value = true
		await request()
	} catch (error) {
		addNotification({
			type: 'error',
			title: 'Failed to start the operation',
			text: error instanceof Error ? error.message : undefined,
		})
	} finally {
		isStarting.value = false
		await queryClient.invalidateQueries({ queryKey: reparationsQueryKey.value })
	}
}
</script>
