<template>
	<Transition name="fade" mode="out-in">
		<div
			v-if="error"
			key="error"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-orange p-4">
							<IssuesIcon class="size-12 text-orange" />
						</div>
						<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load backups</h1>
					</div>
					<p class="text-lg text-secondary">
						We couldn't load your server's backups. Here's what went wrong:
					</p>
					<p>
						<span class="break-all font-mono">{{ error.message }}</span>
					</p>
					<ButtonStyled size="large" color="brand" @click="refetch">
						<button class="mt-6 !w-full">Retry</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<div v-else key="content" class="contents">
			<ReadyTransition :pending="backupsReadyPending">
				<BackupCreateModal ref="createBackupModal" :backups="completedBackups" />
				<BackupRenameModal ref="renameBackupModal" :backups="completedBackups" />
				<BackupRestoreModal ref="restoreBackupModal" />
				<BackupDeleteModal
					ref="deleteBackupModal"
					@delete="deleteBackup"
					@bulk-delete="bulkDelete"
				/>

				<div
					v-if="completedBackups.length"
					class="mb-2 flex flex-wrap items-center justify-between gap-4"
				>
					<div class="flex min-w-0 flex-wrap items-center gap-4">
						<Checkbox
							:model-value="allSelected"
							:indeterminate="someSelected"
							:label="formatMessage(messages.selectAll)"
							class="shrink-0"
							label-class="text-secondary font-semibold"
							@update:model-value="toggleSelectAll"
						/>
						<div class="hidden h-6 w-px bg-surface-5 sm:block" />
						<FilterPills v-model="selectedFilters" :options="filterPillOptions">
							<template #all>{{ formatMessage(commonMessages.allProjectType) }}</template>
						</FilterPills>
					</div>
					<ButtonStyled color="brand">
						<button
							v-tooltip="backupCreationDisabled"
							:disabled="!!backupCreationDisabled"
							@click="showCreateModel"
						>
							<PlusIcon class="size-5" />
							{{ formatMessage(messages.createBackup) }}
						</button>
					</ButtonStyled>
				</div>

				<div class="flex w-full flex-col gap-1.5">
					<div
						v-if="groupedBackups.length === 0"
						class="mt-6 flex flex-col items-center justify-center gap-2 text-center text-secondary"
					>
						<EmptyState
							v-if="completedBackups.length === 0"
							type="empty-inbox"
							:heading="formatMessage(messages.emptyHeading)"
							:description="formatMessage(messages.emptyDescription)"
						>
							<template #actions>
								<ButtonStyled color="brand">
									<button
										v-tooltip="backupCreationDisabled"
										:disabled="!!backupCreationDisabled"
										class="mx-auto w-min"
										@click="showCreateModel"
									>
										<PlusIcon class="size-5" />
										{{ formatMessage(messages.createBackup) }}
									</button>
								</ButtonStyled>
							</template>
						</EmptyState>
						<EmptyState
							v-else
							type="empty-inbox"
							:heading="formatMessage(messages.filteredEmptyHeading)"
							:description="formatMessage(messages.filteredEmptyDescription)"
						>
							<template #actions>
								<ButtonStyled type="outlined">
									<button class="!border !border-surface-4" @click="clearBackupFilters">
										{{ formatMessage(messages.clearFilters) }}
									</button>
								</ButtonStyled>
							</template>
						</EmptyState>
					</div>

					<div v-else class="flex flex-col gap-3">
						<template v-for="group in groupedBackups" :key="group.label">
							<div class="flex items-center gap-2">
								<div class="flex w-5 shrink-0 items-center justify-center">
									<component :is="group.icon" v-if="group.icon" class="size-5" />
								</div>
								<span class="text-lg font-semibold leading-5 text-contrast">{{ group.label }}</span>
							</div>

							<TransitionGroup name="list" tag="div" class="flex flex-col">
								<div
									v-for="(backup, backupIndex) in group.backups"
									:key="`backup-${backup.id}`"
									class="flex gap-2"
								>
									<div class="flex w-5 flex-col items-center">
										<div
											class="w-px flex-1 bg-surface-5"
											:class="{ '-mt-1.5': backupIndex === 0 }"
										/>
										<Checkbox
											:model-value="selectedIds.has(backup.id)"
											:description="formatMessage(messages.selectBackupAria, { name: backup.name })"
											class="shrink-0"
											@update:model-value="toggleSelection(backup.id)"
										/>
										<div class="w-px flex-1 bg-surface-5" />
									</div>
									<BackupItem
										class="my-1.5 min-w-0 flex-1"
										:backup="backup"
										:selected="selectedIds.has(backup.id)"
										:restore-disabled="backupRestoreDisabled"
										:kyros-url="server.node?.instance"
										:jwt="server.node?.token"
										:show-copy-id-action="showCopyIdAction"
										:show-debug-info="showDebugInfo"
										@download="() => triggerDownloadAnimation()"
										@rename="() => renameBackupModal?.show(backup)"
										@restore="() => restoreBackupModal?.show(backup)"
										@delete="
											(skipConfirmation?: boolean) =>
												skipConfirmation ? deleteBackup(backup) : deleteBackupModal?.show(backup)
										"
									/>
								</div>
							</TransitionGroup>
						</template>
					</div>
				</div>

				<FloatingActionBar
					:shown="selectedIds.size > 0 || isBulkOperating"
					:aria-label="
						formatMessage(messages.bulkBarAriaLabel, {
							count: isBulkOperating ? bulkTotal : selectedIds.size,
						})
					"
				>
					<div class="flex items-center gap-0.5">
						<span class="px-4 py-2.5 text-base font-semibold tabular-nums text-contrast">
							{{
								formatMessage(messages.selectedCount, {
									count: isBulkOperating ? bulkTotal : selectedIds.size,
								})
							}}
						</span>
						<div class="mx-1 h-6 w-px bg-surface-5" />
						<ButtonStyled type="transparent">
							<button
								type="button"
								:disabled="isBulkOperating"
								:class="{ 'pointer-events-none opacity-60': isBulkOperating }"
								@click="deselectAll"
							>
								{{ formatMessage(commonMessages.clearButton) }}
							</button>
						</ButtonStyled>
					</div>

					<div v-if="!isBulkOperating" class="ml-auto flex items-center gap-0.5">
						<ButtonStyled type="transparent" color="red" hover-color-fill="background">
							<button type="button" @click="confirmBulkDelete">
								<TrashIcon />
								<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
							</button>
						</ButtonStyled>
					</div>

					<div v-else class="ml-auto flex items-center" aria-live="polite">
						<span class="px-4 py-2.5 text-base font-semibold tabular-nums text-secondary">
							{{ formatMessage(messages.bulkDeleting, { total: bulkTotal }) }}
						</span>
					</div>

					<div v-if="isBulkOperating" class="absolute bottom-0 left-0 right-0 h-1">
						<div
							class="animate-indeterminate h-full rounded-l-full bg-brand"
							role="progressbar"
							:aria-valuemin="0"
							:aria-valuemax="bulkTotal"
							style="box-shadow: 0px -2px 4px 0px rgba(27, 217, 106, 0.1)"
						/>
					</div>
				</FloatingActionBar>

				<div
					class="over-the-top-download-animation"
					:class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
				>
					<div>
						<div
							class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
						></div>
						<div
							class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
						></div>
						<div
							class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
						>
							<DownloadIcon class="h-20 w-20 text-contrast" />
						</div>
					</div>
				</div>
			</ReadyTransition>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { CalendarIcon, DownloadIcon, IssuesIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import type { Component } from 'vue'
import { computed, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import FilterPills, { type FilterPillOption } from '#ui/components/base/FilterPills.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import BackupCreateModal from '#ui/components/servers/backups/BackupCreateModal.vue'
import BackupDeleteModal from '#ui/components/servers/backups/BackupDeleteModal.vue'
import BackupItem from '#ui/components/servers/backups/BackupItem.vue'
import BackupRenameModal from '#ui/components/servers/backups/BackupRenameModal.vue'
import BackupRestoreModal from '#ui/components/servers/backups/BackupRestoreModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerBackupsQueue } from '#ui/composables/servers/server-backups-queue.ts'
import { useBulkOperation } from '#ui/layouts/shared/content-tab/composables/bulk-operations'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import { useBackupsSelection } from '#ui/composables/servers/backups-selection'

const messages = defineMessages({
	selectAll: {
		id: 'servers.backups.toolbar.select-all',
		defaultMessage: 'Select all',
	},
	selectBackupAria: {
		id: 'servers.backups.select-backup-aria',
		defaultMessage: 'Select backup {name}',
	},
	filterManual: {
		id: 'servers.backups.toolbar.filter-manual',
		defaultMessage: 'Manual',
	},
	filterAuto: {
		id: 'servers.backups.toolbar.filter-auto',
		defaultMessage: 'Auto',
	},
	selectedCount: {
		id: 'servers.backups.bulk-bar.selected-count',
		defaultMessage: '{count, plural, one {# backup selected} other {# backups selected}}',
	},
	bulkBarAriaLabel: {
		id: 'servers.backups.bulk-bar.aria-label',
		defaultMessage:
			'{count, plural, one {Bulk actions for one selected backup} other {Bulk actions for # selected backups}}',
	},
	createBackup: {
		id: 'servers.backups.toolbar.create-backup',
		defaultMessage: 'Create backup',
	},
	emptyHeading: {
		id: 'servers.backups.empty.heading',
		defaultMessage: 'No backups yet',
	},
	emptyDescription: {
		id: 'servers.backups.empty.description',
		defaultMessage: 'Create your first backup',
	},
	filteredEmptyHeading: {
		id: 'servers.backups.filtered-empty.heading',
		defaultMessage: 'No backups match',
	},
	filteredEmptyDescription: {
		id: 'servers.backups.filtered-empty.description',
		defaultMessage: 'Try a different filter or clear filters to see all backups.',
	},
	clearFilters: {
		id: 'servers.backups.filtered-empty.clear-filters',
		defaultMessage: 'Clear filters',
	},
	bulkDeleting: {
		id: 'servers.backups.bulk-bar.deleting',
		defaultMessage: 'Deleting {total, plural, one {# backup} other {# backups}}...',
	},
})

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const filterPillOptions = computed<FilterPillOption[]>(() => [
	{ id: 'manual', label: formatMessage(messages.filterManual) },
	{ id: 'auto', label: formatMessage(messages.filterAuto) },
])
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server, serverId, worldId, busyReasons } = injectModrinthServerContext()

const props = defineProps<{
	isServerRunning: boolean
	showCopyIdAction?: boolean
	showDebugInfo?: boolean
}>()

defineEmits(['onDownload'])

const { backups, invalidate, hasActiveCreate, hasActiveRestore, query } = useServerBackupsQueue(
	computed(() => serverId),
	worldId,
)

const error = computed(() => {
	const err = query.error.value
	return err instanceof Error ? err : err ? new Error(String(err)) : null
})
const refetch = () => query.refetch()

/** Until world exists we cannot fetch; `isLoading` is false while the query is disabled, which would flash empty state. */
const backupsReadyPending = computed(
	() => !worldId.value || (query.data.value === undefined && !query.error.value),
)

const selectedFilters = ref<string[]>([])

const completedBackups = computed(() => backups.value.filter((backup) => backup.status === 'done'))

const filteredBackups = computed(() => {
	const f = selectedFilters.value
	if (f.length === 0 || f.length === 2) {
		return completedBackups.value
	}
	const wantAuto = f.includes('auto')
	return completedBackups.value.filter((b) => b.automated === wantAuto)
})

/** Completed backups with a snapshot: queue API schedules deletion. */
const deleteQueueMutation = useMutation({
	mutationFn: (backupId: string) =>
		client.archon.backups_queue_v1.delete(serverId, worldId.value!, backupId),
	onSuccess: async () => {
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
})

/** In-progress / incomplete backups: legacy cancel + delete path. */
const deleteLegacyMutation = useMutation({
	mutationFn: (backupId: string) =>
		client.archon.backups_v1.delete(serverId, worldId.value!, backupId),
	onSuccess: async () => {
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
})

/** Bulk delete via queue API — handles both completed and in-progress backups (cancels the latter). */
const deleteManyMutation = useMutation({
	mutationFn: (backupIds: string[]) =>
		client.archon.backups_queue_v1.deleteMany(serverId, worldId.value!, backupIds),
	onSuccess: async () => {
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
})

type BackupGroup = {
	label: string
	icon: Component | null
	backups: Archon.BackupsQueue.v1.BackupQueueBackup[]
}

const groupedBackups = computed((): BackupGroup[] => {
	if (!filteredBackups.value.length) return []

	const now = dayjs()
	const groups: BackupGroup[] = []

	const addToGroup = (
		label: string,
		icon: Component | null,
		backup: Archon.BackupsQueue.v1.BackupQueueBackup,
	) => {
		let group = groups.find((g) => g.label === label)
		if (!group) {
			group = { label, icon, backups: [] }
			groups.push(group)
		}
		group.backups.push(backup)
	}

	for (const backup of filteredBackups.value) {
		const created = dayjs(backup.created_at)
		const diffMinutes = now.diff(created, 'minute')
		const isToday = created.isSame(now, 'day')
		const isYesterday = created.isSame(now.subtract(1, 'day'), 'day')
		const diffDays = now.diff(created, 'day')

		if (diffMinutes < 30 && isToday) {
			addToGroup('Just now', CalendarIcon, backup)
		} else if (isToday) {
			addToGroup('Earlier today', CalendarIcon, backup)
		} else if (isYesterday) {
			addToGroup('Yesterday', CalendarIcon, backup)
		} else if (diffDays <= 14) {
			addToGroup('Last 2 weeks', CalendarIcon, backup)
		} else {
			addToGroup('Older', CalendarIcon, backup)
		}
	}

	return groups
})

const displayOrderedBackups = computed(() => groupedBackups.value.flatMap((g) => g.backups))

const {
	selectedIds,
	toggleSelection,
	deselectAll,
	toggleSelectAll,
	allSelected,
	someSelected,
	selectedBackups,
} = useBackupsSelection(filteredBackups, displayOrderedBackups)

const { isBulkOperating, bulkTotal } = useBulkOperation()

const overTheTopDownloadAnimation = ref()
const createBackupModal = ref<InstanceType<typeof BackupCreateModal>>()
const renameBackupModal = ref<InstanceType<typeof BackupRenameModal>>()
const restoreBackupModal = ref<InstanceType<typeof BackupRestoreModal>>()
const deleteBackupModal = ref<InstanceType<typeof BackupDeleteModal>>()

const backupRestoreDisabled = computed(() => {
	if (props.isServerRunning) {
		return 'Cannot restore backup while server is running'
	}
	if (busyReasons.value.length > 0) {
		return formatMessage(busyReasons.value[0].reason)
	}
	if (hasActiveCreate.value || hasActiveRestore.value) {
		return 'A backup operation is already queued or in progress'
	}
	return undefined
})

const backupCreationDisabled = computed(() => {
	const quota = server.value.backup_quota
	if (quota !== undefined) {
		const usedCount = backups.value.length ?? server.value.used_backup_quota ?? 0
		if (usedCount >= quota) {
			return `All ${quota} of your backup slots are in use`
		}
	}
	if (busyReasons.value.length > 0) {
		return formatMessage(busyReasons.value[0].reason)
	}
	if (hasActiveCreate.value) {
		return 'A backup is already queued or in progress'
	}
	return undefined
})

const showCreateModel = () => {
	createBackupModal.value?.show()
}

function clearBackupFilters() {
	selectedFilters.value = []
}

function confirmBulkDelete() {
	if (!selectedBackups.value.length) return
	deleteBackupModal.value?.showBulk(selectedBackups.value)
}

async function bulkDelete(toRemove: Archon.BackupsQueue.v1.BackupQueueBackup[]) {
	if (!toRemove.length) return

	isBulkOperating.value = true
	bulkTotal.value = toRemove.length

	try {
		await deleteManyMutation.mutateAsync(toRemove.map((b) => b.id))
	} catch (err) {
		addNotification({
			type: 'error',
			title: `Failed to delete ${toRemove.length} backup${toRemove.length === 1 ? '' : 's'}`,
			text: err instanceof Error ? err.message : String(err),
		})
	} finally {
		deselectAll()
		isBulkOperating.value = false
		bulkTotal.value = 0
	}
}

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
}

function useQueueDeleteFor(backup: Archon.BackupsQueue.v1.BackupQueueBackup) {
	return backup.status === 'done'
}

function deleteBackup(backup?: Archon.BackupsQueue.v1.BackupQueueBackup) {
	if (!backup) {
		addNotification({
			type: 'error',
			title: 'Error deleting backup',
			text: 'Backup is null',
		})
		return
	}

	const mutation = useQueueDeleteFor(backup) ? deleteQueueMutation : deleteLegacyMutation

	mutation.mutate(backup.id, {
		onError: (err) => {
			const message = err instanceof Error ? err.message : String(err)
			addNotification({
				type: 'error',
				title: 'Error deleting backup',
				text: message,
			})
		},
	})
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}

.list-enter-active,
.list-leave-active {
	transition: all 200ms ease-in-out;
}

.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}

@keyframes indeterminate {
	0% {
		width: 20%;
		margin-left: -20%;
	}
	100% {
		width: 60%;
		margin-left: 100%;
	}
}

.animate-indeterminate {
	animation: indeterminate 1.5s ease-in-out infinite;
}

.over-the-top-download-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 1;

	&.animation-hidden {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}
		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}
		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: 1;
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}
</style>
