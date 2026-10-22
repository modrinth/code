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
			<span class="text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
			<span class="text-primary">{{ formatMessage(messages.description) }}</span>
		</div>

		<span v-if="serverQuery.isPending.value" class="text-secondary">
			{{ formatMessage(messages.loading) }}
		</span>
		<span v-else-if="repositoryIds.length === 0" class="text-secondary">
			{{ formatMessage(messages.noRepositories) }}
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
				:header="
					formatMessage(messages.activeOperation, {
						operation: formatMessage(operationLabels[activeOperation.operation_type]),
						status: formatMessage(statusLabels[activeOperation.status]),
					})
				"
				:timestamp="activeOperation.started_at ?? activeOperation.created_at"
			>
				{{ formatMessage(messages.activeOperationDescription) }}
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.checkTitle) }}</span>
				<Checkbox
					v-model="readData"
					:label="formatMessage(messages.readDataLabel)"
					:description="formatMessage(messages.readDataDescription)"
					:disabled="!!activeOperation"
				/>
				<div>
					<Button
						v-tooltip="activeOperation ? formatMessage(messages.operationRunning) : undefined"
						type="colored"
						color="brand"
						:disabled="!!activeOperation || isStarting"
						:loading="isStarting && pendingOperation === null"
						@click="startCheck"
					>
						<SearchIcon class="size-5" />
						{{ formatMessage(latestRun ? messages.startNewCheck : messages.startCheck) }}
					</Button>
				</div>
			</div>

			<div v-if="checkOutput" class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.checkResults) }}</span>
				<div class="flex flex-wrap gap-6">
					<div class="flex flex-col">
						<span class="text-secondary">{{ formatMessage(messages.errorCount) }}</span>
						<span class="text-2xl font-semibold text-contrast">{{ checkOutput.num_errors }}</span>
					</div>
					<div class="flex flex-col">
						<span class="text-secondary">{{ formatMessage(messages.brokenPackCount) }}</span>
						<span class="text-2xl font-semibold text-contrast">{{
							checkOutput.broken_packs.length
						}}</span>
					</div>
					<div class="flex flex-col">
						<span class="text-secondary">{{ formatMessage(messages.missingPackCount) }}</span>
						<span class="text-2xl font-semibold text-contrast">{{ missingPackErrors }}</span>
					</div>
				</div>
				<div class="flex flex-col gap-1">
					<span class="text-secondary">{{ formatMessage(messages.recommendedAction) }}</span>
					<ul class="m-0 flex flex-col gap-1 pl-5 text-primary">
						<li v-for="recommendation in recommendations" :key="recommendation">
							{{ recommendation }}
						</li>
					</ul>
				</div>
				<details v-if="checkOutput.errors.length > 0">
					<summary class="cursor-pointer text-secondary">
						{{
							formatMessage(
								checkOutput.errors_truncated ? messages.showErrorsTruncated : messages.showErrors,
								{ count: checkOutput.errors.length },
							)
						}}
					</summary>
					<pre
						class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-all rounded-xl bg-surface-3 p-3 font-mono text-sm"
						>{{ checkOutput.errors.join('\n') }}</pre
					>
				</details>
			</div>

			<div v-if="latestRun" class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.actionsTitle) }}</span>
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
						{{ formatMessage(operationLabels[operationType]) }}
					</Button>
				</div>
			</div>

			<div v-if="latestRun" class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{
						formatMessage(messages.runTitle, {
							id: latestRun.id,
							time: formatRelativeTime(latestRun.created_at),
						})
					}}
				</span>
				<div
					v-for="operation in latestRun.operations"
					:key="operation.id"
					class="flex flex-col gap-1.5 rounded-2xl bg-surface-2 p-4"
				>
					<div class="flex flex-wrap items-center gap-2">
						<component :is="operationIcons[operation.operation_type]" class="size-5" />
						<span class="font-semibold text-contrast">{{
							formatMessage(operationLabels[operation.operation_type])
						}}</span>
						<SimpleBadge
							:formatted-name="formatMessage(statusLabels[operation.status])"
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
						<summary class="cursor-pointer text-secondary">
							{{ formatMessage(messages.rawResults) }}
						</summary>
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
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'
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
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const messages = defineMessages({
	title: {
		id: 'servers.settings.support.reparation.title',
		defaultMessage: 'Backup repository reparation',
	},
	description: {
		id: 'servers.settings.support.reparation.description',
		defaultMessage:
			'Check a backup repository for damage and repair it. Backups of the repository are paused while an operation runs.',
	},
	loading: {
		id: 'servers.settings.support.reparation.loading',
		defaultMessage: 'Loading backup repositories…',
	},
	noRepositories: {
		id: 'servers.settings.support.reparation.no-repositories',
		defaultMessage: 'This server has no backup repositories.',
	},
	repositoryLabel: {
		id: 'servers.settings.support.reparation.repository-label',
		defaultMessage: 'Repository {id}',
	},
	activeOperation: {
		id: 'servers.settings.support.reparation.active-operation',
		defaultMessage: '{operation} is {status}',
	},
	activeOperationDescription: {
		id: 'servers.settings.support.reparation.active-operation-description',
		defaultMessage:
			'Other operations can start once it finishes. This page refreshes automatically.',
	},
	checkTitle: {
		id: 'servers.settings.support.reparation.check-title',
		defaultMessage: 'Check',
	},
	readDataLabel: {
		id: 'servers.settings.support.reparation.read-data-label',
		defaultMessage: 'Read all pack data',
	},
	readDataDescription: {
		id: 'servers.settings.support.reparation.read-data-description',
		defaultMessage:
			'Not recommended. Downloads and verifies every pack, which can take hours on large repositories. Only use it when a regular check passes but restores still fail.',
	},
	startCheck: {
		id: 'servers.settings.support.reparation.start-check',
		defaultMessage: 'Run check',
	},
	startNewCheck: {
		id: 'servers.settings.support.reparation.start-new-check',
		defaultMessage: 'Run new check',
	},
	checkResults: {
		id: 'servers.settings.support.reparation.check-results',
		defaultMessage: 'Check results',
	},
	errorCount: {
		id: 'servers.settings.support.reparation.error-count',
		defaultMessage: 'Errors',
	},
	brokenPackCount: {
		id: 'servers.settings.support.reparation.broken-pack-count',
		defaultMessage: 'Broken packs',
	},
	missingPackCount: {
		id: 'servers.settings.support.reparation.missing-pack-count',
		defaultMessage: 'Missing pack errors',
	},
	recommendedAction: {
		id: 'servers.settings.support.reparation.recommended-action',
		defaultMessage: 'Recommended action',
	},
	recommendHealthy: {
		id: 'servers.settings.support.reparation.recommend-healthy',
		defaultMessage: 'None, the repository is healthy.',
	},
	recommendRepairPacks: {
		id: 'servers.settings.support.reparation.recommend-repair-packs',
		defaultMessage: 'Repair packs, then repair snapshots.',
	},
	recommendRepairIndex: {
		id: 'servers.settings.support.reparation.recommend-repair-index',
		defaultMessage: 'Repair the index, then repair snapshots.',
	},
	recommendRestoreHiddenVersions: {
		id: 'servers.settings.support.reparation.recommend-restore-hidden-versions',
		defaultMessage:
			'Packs are missing. If the bucket keeps old object versions, restore hidden versions. Otherwise, repair the index, then repair snapshots.',
	},
	recommendPrune: {
		id: 'servers.settings.support.reparation.recommend-prune',
		defaultMessage: 'Prune to remove unused data.',
	},
	recommendReviewErrors: {
		id: 'servers.settings.support.reparation.recommend-review-errors',
		defaultMessage: "The errors don't match a known repair. Review them below.",
	},
	showErrors: {
		id: 'servers.settings.support.reparation.show-errors',
		defaultMessage: 'Show {count, plural, one {# error} other {# errors}}',
	},
	showErrorsTruncated: {
		id: 'servers.settings.support.reparation.show-errors-truncated',
		defaultMessage: 'Show the first {count, plural, one {# error} other {# errors}}',
	},
	actionsTitle: {
		id: 'servers.settings.support.reparation.actions-title',
		defaultMessage: 'Repairs',
	},
	runTitle: {
		id: 'servers.settings.support.reparation.run-title',
		defaultMessage: 'Run {id}, started {time}',
	},
	rawResults: {
		id: 'servers.settings.support.reparation.raw-results',
		defaultMessage: 'Raw results',
	},
	operationRunning: {
		id: 'servers.settings.support.reparation.operation-running',
		defaultMessage: 'Another operation is running on this repository',
	},
	checkRequired: {
		id: 'servers.settings.support.reparation.check-required',
		defaultMessage: 'Requires a completed check',
	},
	alreadyRan: {
		id: 'servers.settings.support.reparation.already-ran',
		defaultMessage: 'Already started in this run. Run a new check to repeat it',
	},
	noBrokenPacks: {
		id: 'servers.settings.support.reparation.no-broken-packs',
		defaultMessage: 'The check found no broken packs',
	},
	repairSnapshotsRequirement: {
		id: 'servers.settings.support.reparation.repair-snapshots-requirement',
		defaultMessage: 'Requires a completed index or pack repair in this run',
	},
	pruneRequirement: {
		id: 'servers.settings.support.reparation.prune-requirement',
		defaultMessage:
			'Requires the check to suggest pruning, or a completed snapshot repair in this run',
	},
	timingStarted: {
		id: 'servers.settings.support.reparation.timing-started',
		defaultMessage: 'started {time}',
	},
	timingCreated: {
		id: 'servers.settings.support.reparation.timing-created',
		defaultMessage: 'queued {time}',
	},
	timingCompleted: {
		id: 'servers.settings.support.reparation.timing-completed',
		defaultMessage: 'finished {time}',
	},
	summaryCheck: {
		id: 'servers.settings.support.reparation.summary-check',
		defaultMessage:
			'{errors, plural, one {# error} other {# errors}}, {packs, plural, one {# broken pack} other {# broken packs}}',
	},
	summaryCommand: {
		id: 'servers.settings.support.reparation.summary-command',
		defaultMessage: 'Restic exited with code {code}',
	},
	summaryRepairSnapshots: {
		id: 'servers.settings.support.reparation.summary-repair-snapshots',
		defaultMessage:
			'{remapped, plural, one {# backup} other {# backups}} moved to repaired snapshots, {missing, plural, one {# backup} other {# backups}} missing their snapshot',
	},
	summaryRestoreRunning: {
		id: 'servers.settings.support.reparation.summary-restore-running',
		defaultMessage: 'Restoring into repository {id}',
	},
	summaryRestoreDone: {
		id: 'servers.settings.support.reparation.summary-restore-done',
		defaultMessage:
			'{imported, plural, one {# backup} other {# backups}} restored into repository {id} ({matched} matched to existing backups). {copied, plural, one {# object} other {# objects}} copied, {missing, plural, one {# object} other {# objects}} missing.',
	},
	confirmDescription: {
		id: 'servers.settings.support.reparation.confirm-description',
		defaultMessage:
			'Run {operation} on repository {id}? Backups of the repository are paused until it finishes.',
	},
	confirmRestoreDescription: {
		id: 'servers.settings.support.reparation.confirm-restore-description',
		defaultMessage:
			"Copy every retained version of repository {id}'s objects into a new repository and publish its snapshots as restored backups? The new repository becomes the world's backup target. This can take hours.",
	},
	confirmPruneDescription: {
		id: 'servers.settings.support.reparation.confirm-prune-description',
		defaultMessage:
			"Prune repository {id}? Data that isn't referenced by a snapshot is permanently deleted.",
	},
	startError: {
		id: 'servers.settings.support.reparation.start-error',
		defaultMessage: 'Failed to start the operation',
	},
	opCheck: {
		id: 'servers.settings.support.reparation.operation.check',
		defaultMessage: 'Check',
	},
	opCheckReadData: {
		id: 'servers.settings.support.reparation.operation.check-read-data',
		defaultMessage: 'Check with data',
	},
	opRepairIndex: {
		id: 'servers.settings.support.reparation.operation.repair-index',
		defaultMessage: 'Repair index',
	},
	opRepairPacks: {
		id: 'servers.settings.support.reparation.operation.repair-packs',
		defaultMessage: 'Repair packs',
	},
	opRepairSnapshots: {
		id: 'servers.settings.support.reparation.operation.repair-snapshots',
		defaultMessage: 'Repair snapshots',
	},
	opPrune: {
		id: 'servers.settings.support.reparation.operation.prune',
		defaultMessage: 'Prune',
	},
	opRestoreHiddenVersions: {
		id: 'servers.settings.support.reparation.operation.restore-hidden-versions',
		defaultMessage: 'Restore hidden versions',
	},
	statusPending: {
		id: 'servers.settings.support.reparation.status.pending',
		defaultMessage: 'pending',
	},
	statusOngoing: {
		id: 'servers.settings.support.reparation.status.ongoing',
		defaultMessage: 'running',
	},
	statusCompleted: {
		id: 'servers.settings.support.reparation.status.completed',
		defaultMessage: 'completed',
	},
	statusFailed: {
		id: 'servers.settings.support.reparation.status.failed',
		defaultMessage: 'failed',
	},
})

const operationLabels: Record<OperationType, MessageDescriptor> = {
	check: messages.opCheck,
	'check-read-data': messages.opCheckReadData,
	'repair-index': messages.opRepairIndex,
	'repair-packs': messages.opRepairPacks,
	'repair-snapshots': messages.opRepairSnapshots,
	prune: messages.opPrune,
	'restore-hidden-versions': messages.opRestoreHiddenVersions,
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

const statusLabels: Record<OperationStatus, MessageDescriptor> = {
	pending: messages.statusPending,
	ongoing: messages.statusOngoing,
	completed: messages.statusCompleted,
	failed: messages.statusFailed,
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
	return id === null ? '' : formatMessage(messages.repositoryLabel, { id })
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
		result.push(formatMessage(messages.recommendRepairPacks))
	}
	if (missingPackErrors.value > 0) {
		result.push(formatMessage(messages.recommendRestoreHiddenVersions))
	} else if (output.suggest_repair_index) {
		result.push(formatMessage(messages.recommendRepairIndex))
	}
	if (output.suggest_prune) {
		result.push(formatMessage(messages.recommendPrune))
	}
	if (result.length === 0) {
		result.push(
			formatMessage(
				output.num_errors === 0 ? messages.recommendHealthy : messages.recommendReviewErrors,
			),
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
	if (activeOperation.value) return formatMessage(messages.operationRunning)

	const output = checkOutput.value
	if (!output) return formatMessage(messages.checkRequired)

	const existing = latestRun.value?.operations.find(
		(operation) => operation.operation_type === operationType,
	)
	if (existing && existing.status !== 'failed') return formatMessage(messages.alreadyRan)

	switch (operationType) {
		case 'repair-packs':
			return output.broken_packs.length > 0 ? null : formatMessage(messages.noBrokenPacks)
		case 'repair-snapshots':
			return completedInRun('repair-index') || completedInRun('repair-packs')
				? null
				: formatMessage(messages.repairSnapshotsRequirement)
		case 'prune':
			return output.suggest_prune || completedInRun('repair-snapshots')
				? null
				: formatMessage(messages.pruneRequirement)
		default:
			return null
	}
}

function operationTiming(operation: ReparationOperation) {
	if (operation.completed_at) {
		return formatMessage(messages.timingCompleted, {
			time: formatRelativeTime(operation.completed_at),
		})
	}
	if (operation.started_at) {
		return formatMessage(messages.timingStarted, { time: formatRelativeTime(operation.started_at) })
	}
	return formatMessage(messages.timingCreated, { time: formatRelativeTime(operation.created_at) })
}

function operationSummary(operation: ReparationOperation): string | null {
	const results = operation.results
	if (!results) return null

	if (operation.operation_type === 'restore-hidden-versions') {
		const restore = results as RestoreHiddenVersionsResults
		if (restore.imported_backups === undefined) {
			return formatMessage(messages.summaryRestoreRunning, {
				id: restore.destination_repository_id,
			})
		}
		return formatMessage(messages.summaryRestoreDone, {
			imported: restore.imported_backups,
			matched: restore.matched_backups ?? 0,
			id: restore.destination_repository_id,
			copied: restore.copied_objects ?? 0,
			missing: restore.missing_objects?.length ?? 0,
		})
	}

	if (operation.operation_type === 'repair-snapshots') {
		const repair = results as RepairSnapshotsResults
		return formatMessage(messages.summaryRepairSnapshots, {
			remapped: repair.remapped_backups.length,
			missing: repair.missing_snapshot_backups.length,
		})
	}

	const output = results as RepairOutput
	if (output.type === 'check') {
		return formatMessage(messages.summaryCheck, {
			errors: output.num_errors,
			packs: output.broken_packs.length,
		})
	}
	if (output.type === 'command') {
		return formatMessage(messages.summaryCommand, { code: output.exit_code ?? '?' })
	}
	return null
}

const readData = ref(false)
const isStarting = ref(false)
const pendingOperation = ref<RunnableOperationType | null>(null)
const confirmOperationModal = ref<InstanceType<typeof ConfirmModal>>()

const confirmTitle = computed(() =>
	pendingOperation.value ? formatMessage(operationLabels[pendingOperation.value]) : '',
)

const confirmDescription = computed(() => {
	const operationType = pendingOperation.value
	if (!operationType) return ''

	const id = selectedRepositoryId.value
	if (operationType === 'restore-hidden-versions') {
		return formatMessage(messages.confirmRestoreDescription, { id })
	}
	if (operationType === 'prune') {
		return formatMessage(messages.confirmPruneDescription, { id })
	}
	return formatMessage(messages.confirmDescription, {
		operation: formatMessage(operationLabels[operationType]),
		id,
	})
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
			title: formatMessage(messages.startError),
			text: error instanceof Error ? error.message : undefined,
		})
	} finally {
		isStarting.value = false
		await queryClient.invalidateQueries({ queryKey: reparationsQueryKey.value })
	}
}
</script>
