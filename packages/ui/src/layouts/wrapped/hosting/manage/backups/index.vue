<template>
	<div v-if="error" class="flex w-full justify-center p-4">
		<ErrorInformationCard
			:title="formatMessage(messages.loadErrorTitle)"
			:description="formatMessage(messages.loadErrorDescription)"
			:icon="IssuesIcon"
			:error-details="[
				{ label: formatMessage(commonMessages.errorLabel), value: error.message, type: 'block' },
			]"
			:action="{ label: formatMessage(commonMessages.retryButton), onClick: () => query.refetch() }"
		/>
	</div>

	<ReadyTransition v-else :pending="readyPending">
		<div class="flex flex-col gap-6">
			<div class="grid gap-3 md:grid-cols-2">
				<SlotCard
					:title="formatMessage(messages.autoTitle)"
					:icon="CalendarClockIcon"
					:used="autoUsed"
					:limit="options?.split_automatic ?? 0"
					:off-label="autoEnabled ? undefined : formatMessage(messages.autoOff)"
					color="blue"
				>
					<template v-if="autoEnabled && options">
						<span>{{ formatFrequency(options.interval) }}</span>
						<BulletDivider />
						<span>{{ nextAutoBackupLabel }}</span>
					</template>
					<span v-else>{{ formatMessage(messages.autoOffDescription) }}</span>
				</SlotCard>
				<SlotCard
					:title="formatMessage(messages.manualTitle)"
					:icon="SaveIcon"
					:used="manualUsed"
					:limit="options?.split_manual ?? 0"
					color="purple"
				>
					{{ formatMessage(messages.manualDescription) }}
				</SlotCard>
			</div>

			<section class="flex flex-col gap-4">
				<h2 class="m-0 text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.historyTitle) }}
				</h2>

				<div class="flex flex-col gap-2 md:flex-row">
					<Input
						v-model="search"
						:icon="SearchIcon"
						type="search"
						size="medium"
						wrapper-class="min-w-0 md:flex-1"
						:placeholder="
							formatMessage(messages.searchPlaceholder, { count: completedBackups.length })
						"
						:aria-label="formatMessage(messages.searchLabel)"
						:clear-label="formatMessage(commonMessages.clearButton)"
						clearable
					/>
					<div class="flex shrink-0 flex-col gap-2 md:flex-row">
						<Button
							v-if="settingsModal"
							size="lg"
							@click="settingsModal.openServerSettings({ tabId: 'backups' })"
						>
							<Settings2Icon aria-hidden="true" />
							{{ formatMessage(commonMessages.settingsLabel) }}
						</Button>
						<SplitButton
							v-tooltip="createDisabledReason"
							type="colored"
							color="brand"
							size="lg"
							class="[&>[data-button]:first-child]:flex-1"
							:options="createMenuOptions"
							:menu-label="formatMessage(messages.moreCreateOptions)"
							:primary-disabled="!!createDisabledReason"
							@click="createModal?.show()"
						>
							<PlusIcon aria-hidden="true" />
							{{ formatMessage(messages.createBackup) }}
							<template #download-files>
								<DownloadIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.downloadFilesButton) }}
							</template>
						</SplitButton>
					</div>
				</div>

				<EmptyState
					v-if="completedBackups.length === 0"
					type="empty-inbox"
					:heading="formatMessage(messages.emptyHeading)"
					:description="formatMessage(messages.emptyDescription)"
				/>
				<HistoryTable
					v-else
					v-model:selected-ids="selectedIds"
					:backups="filteredBackups"
					:restore-disabled="restoreDisabledReason"
					:can-manage="canManageBackups"
					:write-disabled-tooltip="permissionDeniedMessage"
					:highlighted-id="highlightedBackupId"
					:show-copy-id-action="showCopyIdAction"
					:show-debug-info="showDebugInfo"
					:can-download="!!server.node && !!worldId"
					@download="downloadBackup"
					@restore="(backup) => restoreModal?.show(backup)"
					@rename="(backup) => renameModal?.show(backup)"
					@delete="(backup) => deleteModal?.show(backup)"
				/>
			</section>
		</div>

		<FloatingActionBar
			:shown="selectedBackups.length > 0"
			:aria-label="formatMessage(messages.selectedCount, { count: selectedBackups.length })"
		>
			<span class="px-4 py-2.5 font-semibold tabular-nums text-contrast">
				{{ formatMessage(messages.selectedCount, { count: selectedBackups.length }) }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<Button type="quiet" :disabled="deleteMutation.isPending.value" @click="selectedIds = []">
				{{ formatMessage(commonMessages.clearButton) }}
			</Button>
			<Button
				v-tooltip="canManageBackups ? undefined : permissionDeniedMessage"
				type="quiet"
				color="red"
				interaction="filled"
				class="ml-auto"
				:disabled="!canManageBackups"
				:loading="deleteMutation.isPending.value"
				@click="deleteModal?.showBulk(selectedBackups)"
			>
				<TrashIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.deleteLabel) }}
			</Button>
		</FloatingActionBar>

		<BackupCreateModal
			ref="createModal"
			:backups="completedBackups"
			:can-create="canManageBackups"
			:permission-denied-message="permissionDeniedMessage"
		/>
		<BackupRenameModal
			ref="renameModal"
			:backups="completedBackups"
			:can-rename="canManageBackups"
			:permission-denied-message="permissionDeniedMessage"
		/>
		<BackupRestoreModal
			ref="restoreModal"
			:can-restore="canManageBackups"
			:permission-denied-message="permissionDeniedMessage"
		/>
		<BackupDeleteModal
			ref="deleteModal"
			:can-delete="canManageBackups"
			:permission-denied-message="permissionDeniedMessage"
			@delete="(backup) => backup && deleteBackups([backup])"
			@bulk-delete="deleteBackups"
		/>
	</ReadyTransition>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	CalendarClockIcon,
	DownloadIcon,
	IssuesIcon,
	PlusIcon,
	SaveIcon,
	SearchIcon,
	Settings2Icon,
	TrashIcon,
} from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import BulletDivider from '#ui/components/base/BulletDivider.vue'
import { Button, type ButtonMenuOption, SplitButton } from '#ui/components/base/buttons'
import EmptyState from '#ui/components/base/EmptyState.vue'
import ErrorInformationCard from '#ui/components/base/ErrorInformationCard.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import BackupCreateModal from '#ui/components/servers/backups/BackupCreateModal.vue'
import BackupDeleteModal from '#ui/components/servers/backups/BackupDeleteModal.vue'
import BackupRenameModal from '#ui/components/servers/backups/BackupRenameModal.vue'
import BackupRestoreModal from '#ui/components/servers/backups/BackupRestoreModal.vue'
import { useBackupIntervalFormat } from '#ui/composables/hosting/backup-interval'
import { useRelativeTime } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerBackupsQueue } from '#ui/composables/server-backups-queue'
import { useServerWorldDownload } from '#ui/composables/server-download'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectFileDownload,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { injectServerSettingsModal } from '#ui/providers/server-settings-modal'
import { commonMessages } from '#ui/utils/common-messages'

import HistoryTable from './history-table.vue'
import SlotCard from './slot-card.vue'
import { useFocusedBackup } from './use-focused-backup'

type Backup = Archon.BackupsQueue.v1.BackupQueueBackup

const props = defineProps<{
	isServerRunning: boolean
	showCopyIdAction?: boolean
	showDebugInfo?: boolean
}>()

const messages = defineMessages({
	loadErrorTitle: {
		id: 'servers.backups.load-error.title',
		defaultMessage: 'Failed to load backups',
	},
	loadErrorDescription: {
		id: 'servers.backups.load-error.description',
		defaultMessage: "We couldn't load your server's backups.",
	},
	autoTitle: {
		id: 'servers.backups.slots.auto.title',
		defaultMessage: 'Auto backups',
	},
	autoOff: {
		id: 'servers.backups.slots.auto.off',
		defaultMessage: 'Off',
	},
	autoOffDescription: {
		id: 'servers.backups.slots.auto.off-description',
		defaultMessage: 'Turn on auto backups in settings',
	},
	nextBackup: {
		id: 'servers.backups.slots.auto.next',
		defaultMessage: 'Next {time}',
	},
	nextBackupSoon: {
		id: 'servers.backups.slots.auto.next-soon',
		defaultMessage: 'Next backup starting soon',
	},
	manualTitle: {
		id: 'servers.backups.slots.manual.title',
		defaultMessage: 'Manual backups',
	},
	manualDescription: {
		id: 'servers.backups.slots.manual.description',
		defaultMessage: 'Manually created',
	},
	historyTitle: {
		id: 'servers.backups.history.title',
		defaultMessage: 'Backup history',
	},
	searchPlaceholder: {
		id: 'servers.backups.history.search-placeholder',
		defaultMessage: 'Search {count, plural, one {# backup} other {# backups}}...',
	},
	searchLabel: {
		id: 'servers.backups.history.search-label',
		defaultMessage: 'Search backups',
	},
	createBackup: {
		id: 'servers.backups.toolbar.create-backup',
		defaultMessage: 'Create backup',
	},
	moreCreateOptions: {
		id: 'servers.backups.toolbar.more-create-options',
		defaultMessage: 'More backup options',
	},
	emptyHeading: {
		id: 'servers.backups.empty.heading',
		defaultMessage: 'No backups yet',
	},
	emptyDescription: {
		id: 'servers.backups.empty.description',
		defaultMessage: 'Create your first backup',
	},
	selectedCount: {
		id: 'servers.backups.bulk-bar.selected-count',
		defaultMessage: '{count, plural, one {# backup selected} other {# backups selected}}',
	},
	manualSlotsFull: {
		id: 'servers.backups.create.slots-full',
		defaultMessage:
			'All {limit, plural, one {# manual backup slot is} other {# manual backup slots are}} in use',
	},
	backupInProgress: {
		id: 'servers.backups.create.in-progress',
		defaultMessage: 'A backup is already queued or in progress',
	},
	restoreWhileRunning: {
		id: 'servers.backups.restore.server-running',
		defaultMessage: 'Stop your server before restoring a backup',
	},
	operationInProgress: {
		id: 'servers.backups.restore.operation-in-progress',
		defaultMessage: 'A backup operation is already queued or in progress',
	},
	deleteError: {
		id: 'servers.backups.delete.error',
		defaultMessage: 'Failed to delete {count, plural, one {backup} other {# backups}}',
	},
})

const LIVE_STATUSES: Backup['status'][] = ['pending', 'in_progress', 'done']

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const { formatFrequency } = useBackupIntervalFormat()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { server, serverId, worldId, busyReasons } = injectModrinthServerContext()
const settingsModal = injectServerSettingsModal(null)
const fileDownload = injectFileDownload(null)
const { canManageBackups, permissionDeniedMessage } = useServerPermissions()

const { query, backups, options, nextAutoBackupAt, hasActiveCreate, hasActiveRestore, invalidate } =
	useServerBackupsQueue(
		computed(() => serverId),
		worldId,
	)

const error = computed(() => query.error.value)
const readyPending = computed(
	() => !worldId.value || (query.data.value === undefined && !query.error.value),
)

const completedBackups = computed(() => backups.value.filter((b) => b.status === 'done'))
const search = ref('')
const filteredBackups = computed(() => {
	const term = search.value.trim().toLowerCase()
	if (!term) return completedBackups.value
	return completedBackups.value.filter((backup) => backup.name.toLowerCase().includes(term))
})

const selectedIds = ref<string[]>([])
const selectedBackups = computed(() =>
	completedBackups.value.filter((backup) => selectedIds.value.includes(backup.id)),
)

const { highlightedBackupId } = useFocusedBackup(
	computed(() => completedBackups.value.map((backup) => backup.id)),
)

const autoEnabled = computed(() => (options.value?.split_automatic ?? 0) > 0)
const autoUsed = computed(() => backups.value.filter((b) => b.rotates_at).length)
const manualUsed = computed(
	() => backups.value.filter((b) => !b.automated && LIVE_STATUSES.includes(b.status)).length,
)

const nextAutoBackupLabel = computed(() => {
	const next = nextAutoBackupAt.value
	if (!next || new Date(next).getTime() <= Date.now()) {
		return formatMessage(messages.nextBackupSoon)
	}
	return formatMessage(messages.nextBackup, { time: formatRelativeTime(next) })
})

const createDisabledReason = computed(() => {
	if (!canManageBackups.value) return permissionDeniedMessage.value
	const limit = options.value?.split_manual ?? 0
	if (manualUsed.value >= limit) return formatMessage(messages.manualSlotsFull, { limit })
	if (busyReasons.value.length > 0) return formatMessage(busyReasons.value[0].reason)
	if (hasActiveCreate.value) return formatMessage(messages.backupInProgress)
	return undefined
})

const { downloadWorldFiles } = useServerWorldDownload()
const isDownloadingFiles = ref(false)

async function downloadFiles() {
	const node = server.value.node
	const currentWorldId = worldId.value
	if (!node || !currentWorldId || isDownloadingFiles.value) return

	isDownloadingFiles.value = true
	try {
		await downloadWorldFiles(node.instance, currentWorldId)
	} finally {
		isDownloadingFiles.value = false
	}
}

const createMenuOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'download-files',
		label: formatMessage(commonMessages.downloadFilesButton),
		disabled: !server.value.node || !worldId.value || isDownloadingFiles.value,
		action: downloadFiles,
	},
])

const restoreDisabledReason = computed(() => {
	if (!canManageBackups.value) return permissionDeniedMessage.value
	if (props.isServerRunning) return formatMessage(messages.restoreWhileRunning)
	if (busyReasons.value.length > 0) return formatMessage(busyReasons.value[0].reason)
	if (hasActiveCreate.value || hasActiveRestore.value) {
		return formatMessage(messages.operationInProgress)
	}
	return undefined
})

async function downloadBackup(backup: Backup) {
	const node = server.value.node
	const currentWorldId = worldId.value
	if (!node || !currentWorldId) return

	try {
		if (fileDownload) {
			await fileDownload.download({
				type: 'server-backup',
				filename: `${backup.name}.zip`,
				serverId,
				serverName: server.value.name,
				nodeUrlHost: node.instance,
				worldId: currentWorldId,
				backupId: backup.id,
			})
			return
		}

		const { token } = await client.kyros.backups_v1.authorizeDownload(
			node.instance,
			currentWorldId,
			backup.id,
		)
		const link = document.createElement('a')
		link.href = client.kyros.backups_v1.getDownloadUrl(
			node.instance,
			currentWorldId,
			backup.id,
			token,
		)
		link.download = ''
		link.rel = 'noopener'
		link.click()
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(commonMessages.downloadFailedLabel),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}

const createModal = ref<InstanceType<typeof BackupCreateModal>>()
const renameModal = ref<InstanceType<typeof BackupRenameModal>>()
const restoreModal = ref<InstanceType<typeof BackupRestoreModal>>()
const deleteModal = ref<InstanceType<typeof BackupDeleteModal>>()

const deleteMutation = useMutation({
	mutationFn: (backupIds: string[]) =>
		backupIds.length === 1
			? client.archon.backups_queue_v1.delete(serverId, worldId.value!, backupIds[0])
			: client.archon.backups_queue_v1.deleteMany(serverId, worldId.value!, backupIds),
	onSuccess: async () => {
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
})

function deleteBackups(toDelete: Backup[]) {
	if (!canManageBackups.value || toDelete.length === 0) return

	const ids = toDelete.map((backup) => backup.id)
	deleteMutation.mutate(ids, {
		onSuccess: () => {
			selectedIds.value = selectedIds.value.filter((id) => !ids.includes(id))
		},
		onError: (err) => {
			addNotification({
				type: 'error',
				title: formatMessage(messages.deleteError, { count: ids.length }),
				text: err instanceof Error ? err.message : String(err),
			})
		},
	})
}
</script>
