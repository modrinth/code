<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import StackedAdmonitions, {
	type StackedAdmonitionItem,
} from '#ui/components/base/StackedAdmonitions.vue'
import InstallingBanner, {
	type ContentError,
	type SyncProgress,
} from '#ui/components/servers/InstallingBanner.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerBackupsQueue } from '#ui/composables/server-backups-queue'
import type { FileOperation } from '#ui/layouts/shared/files-tab/types'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

import BackupAdmonition, { type BackupAdmonitionEntry } from './BackupAdmonition.vue'
import FileOperationAdmonition from './FileOperationAdmonition.vue'
import UploadAdmonition from './UploadAdmonition.vue'

const props = defineProps<{
	syncProgress?: SyncProgress | null
	contentError?: ContentError | null
}>()

const emit = defineEmits<{
	'content-retry': []
}>()

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const ctx = injectModrinthServerContext()
const route = useRoute()

const { activeOperations, backups, progressFor, invalidate } = useServerBackupsQueue(
	computed(() => ctx.serverId),
	ctx.worldId,
)

const messages = defineMessages({
	backgroundTaskRunning: {
		id: 'servers.admonitions.background-task-running',
		defaultMessage: 'Background task running',
	},
	contentBusyBody: {
		id: 'content.page-layout.busy-description',
		defaultMessage: 'Please wait for the operation to complete before editing content.',
	},
	filesBusyBody: {
		id: 'files.layout.busy-warning',
		defaultMessage: 'File operations are disabled while the operation is in progress.',
	},
})

const isOnContentTab = computed(() => route.path.includes('/content'))
const isOnFilesTab = computed(() => route.path.includes('/files'))

const bannerCoversInstalling = computed(
	() =>
		ctx.server.value?.status === 'installing' ||
		ctx.isSyncingContent.value ||
		ctx.busyReasons.value.some(
			(r) =>
				r.reason.id === 'servers.busy.installing' || r.reason.id === 'servers.busy.syncing-content',
		),
)

function isBackupReason(id: string) {
	return id === 'servers.busy.backup-creating' || id === 'servers.busy.backup-restoring'
}

function isInstallingReason(id: string) {
	return id === 'servers.busy.installing' || id === 'servers.busy.syncing-content'
}

const filteredBusyReasons = computed(() =>
	ctx.busyReasons.value.filter((r) => {
		if (isBackupReason(r.reason.id)) return false
		if (bannerCoversInstalling.value && isInstallingReason(r.reason.id)) return false
		return true
	}),
)

const contentBusyHeader = computed(() =>
	filteredBusyReasons.value.length > 0 ? formatMessage(filteredBusyReasons.value[0].reason) : null,
)

const filesBusyHeader = computed(() =>
	filteredBusyReasons.value.length > 0 ? formatMessage(filteredBusyReasons.value[0].reason) : null,
)

const dismissedIds = reactive(new Set<string>())
const cancellingIds = reactive(new Set<string>())
const dismissedContentErrorKey = ref<string | null>(null)

const contentErrorKey = computed(() =>
	props.contentError ? `${props.contentError.step}:${props.contentError.description}` : null,
)

watch(contentErrorKey, (key) => {
	if (!key) {
		dismissedContentErrorKey.value = null
	}
})

const backupAdmonitionEntries = computed<BackupAdmonitionEntry[]>(() => {
	const result: BackupAdmonitionEntry[] = []
	const backupById = new Map(backups.value.map((b) => [b.id, b]))

	for (const op of activeOperations.value) {
		const key = `${op.backup_id}:${op.operation_type}:${op.operation_id ?? 'legacy'}`
		if (dismissedIds.has(key)) continue
		const backup = backupById.get(op.backup_id)
		const history = backup?.history.find(
			(h) =>
				h.operation_type === op.operation_type &&
				(h.operation_id ?? null) === (op.operation_id ?? null),
		)
		const rawProgress = progressFor(op.backup_id, op.operation_type) ?? 0
		result.push({
			key,
			backupId: op.backup_id,
			type: op.operation_type,
			state: history?.state ?? 'ongoing',
			progress: rawProgress,
			operationId: op.operation_id ?? null,
			syntheticLegacy: op.synthetic_legacy,
			name: backup?.name,
			timestamp: history?.scheduled_for ?? op.scheduled_for,
		})
	}

	for (const backup of backups.value) {
		const last = backup.history[0]
		if (!last || !last.should_prompt) continue
		if (last.state === 'pending' || last.state === 'ongoing') continue
		const key = `${backup.id}:${last.operation_type}:${last.operation_id ?? 'legacy'}`
		if (dismissedIds.has(key)) continue
		if (result.some((r) => r.key === key)) continue
		result.push({
			key,
			backupId: backup.id,
			type: last.operation_type,
			state: last.state,
			progress: 0,
			operationId: last.operation_id ?? null,
			syntheticLegacy: last.synthetic_legacy,
			name: backup.name,
			timestamp: last.completed_at ?? last.scheduled_for,
			error: last.error ?? null,
		})
	}

	return result
})

type ServerAdmonitionItem = StackedAdmonitionItem & {
	priority: number
	sortIndex: number
} & (
		| { kind: 'installing' }
		| { kind: 'upload' }
		| { kind: 'fs-op'; op: FileOperation }
		| { kind: 'backup'; entry: BackupAdmonitionEntry }
		| { kind: 'busy-content' }
		| { kind: 'busy-files' }
	)

const showInstallingBanner = computed(() => {
	if (!ctx.server.value) return false
	const installing = bannerCoversInstalling.value || !!props.contentError
	if (!installing) return false
	if (contentErrorKey.value && dismissedContentErrorKey.value === contentErrorKey.value)
		return false
	return props.syncProgress?.phase !== 'Analyzing'
})

function fsOpType(op: FileOperation): StackedAdmonitionItem['type'] {
	if (op.state === 'done') return 'success'
	if (op.state?.startsWith('fail')) return 'critical'
	return 'info'
}

function fsOpPriority(op: FileOperation): number {
	if (op.state?.startsWith('fail')) return 1
	if (op.state === 'done') return 4
	if (op.state === 'queued') return 3
	return 2
}

function backupType(entry: BackupAdmonitionEntry): StackedAdmonitionItem['type'] {
	if (entry.state === 'failed' || entry.state === 'timed_out') return 'critical'
	if (entry.state === 'completed') return 'success'
	return 'info'
}

function backupPriority(entry: BackupAdmonitionEntry): number {
	if (entry.state === 'failed' || entry.state === 'timed_out') return 1
	if (entry.state === 'ongoing') return 2
	if (entry.state === 'pending') return 3
	return 4
}

const stackItems = computed<ServerAdmonitionItem[]>(() => {
	const out: ServerAdmonitionItem[] = []
	let sortIndex = 0

	if (showInstallingBanner.value) {
		out.push({
			id: 'installing',
			type: props.contentError ? 'critical' : 'info',
			dismissible: !!props.contentError,
			kind: 'installing',
			priority: 0,
			sortIndex: sortIndex++,
		})
	}

	if (ctx.uploadState.value.isUploading) {
		out.push({
			id: 'upload-active',
			type: 'info',
			dismissible: false,
			kind: 'upload',
			priority: 2,
			sortIndex: sortIndex++,
		})
	}

	for (const op of ctx.activeOperations.value) {
		out.push({
			id: op.id ? `fs-op-${op.id}` : `fs-op-${op.op}-${op.src}`,
			type: fsOpType(op),
			dismissible: !!op.id && (op.state === 'done' || !!op.state?.startsWith('fail')),
			kind: 'fs-op',
			op,
			priority: fsOpPriority(op),
			sortIndex: sortIndex++,
		})
	}

	for (const entry of backupAdmonitionEntries.value) {
		out.push({
			id: `backup-${entry.key}`,
			type: backupType(entry),
			dismissible: entry.state !== 'pending' && entry.state !== 'ongoing',
			kind: 'backup',
			entry,
			priority: backupPriority(entry),
			sortIndex: sortIndex++,
		})
	}

	if (contentBusyHeader.value) {
		const p = isOnContentTab.value ? 0 : 5
		out.push({
			id: 'busy-content',
			type: 'warning',
			dismissible: false,
			kind: 'busy-content',
			priority: p,
			sortIndex: sortIndex++,
		})
	}

	if (filesBusyHeader.value) {
		const p = isOnFilesTab.value ? 0 : 5
		out.push({
			id: 'busy-files',
			type: 'warning',
			dismissible: false,
			kind: 'busy-files',
			priority: p,
			sortIndex: sortIndex++,
		})
	}

	return out.sort((a, b) => a.priority - b.priority || a.sortIndex - b.sortIndex)
})

const hasBulkDismissableItems = computed(() => stackItems.value.some((it) => it.dismissible))

async function onBackupDismiss(item: BackupAdmonitionEntry) {
	dismissedIds.add(item.key)
	if (item.syntheticLegacy || item.operationId == null) {
		await invalidate()
		return
	}
	try {
		if (item.type === 'create') {
			await client.archon.backups_queue_v1.ackCreate(
				ctx.serverId,
				ctx.worldId.value!,
				item.operationId,
			)
		} else {
			await client.archon.backups_queue_v1.ackRestore(
				ctx.serverId,
				ctx.worldId.value!,
				item.operationId,
			)
		}
	} catch (err) {
		dismissedIds.delete(item.key)
		console.error('Failed to acknowledge backup operation', err)
	} finally {
		await invalidate()
	}
}

async function onBackupCancel(item: BackupAdmonitionEntry) {
	if (cancellingIds.has(item.key)) return
	cancellingIds.add(item.key)
	try {
		await client.archon.backups_v1.delete(ctx.serverId, ctx.worldId.value!, item.backupId)
		await invalidate()
	} catch (err) {
		cancellingIds.delete(item.key)
		throw err
	}
}

async function onBackupRetry(item: BackupAdmonitionEntry) {
	await client.archon.backups_queue_v1.retry(ctx.serverId, ctx.worldId.value!, item.backupId)
	dismissedIds.add(item.key)
	await invalidate()
}

async function onDismissAll() {
	const tasks: Promise<unknown>[] = []
	for (const it of stackItems.value) {
		if (!it.dismissible) continue
		if (it.kind === 'installing' && props.contentError) {
			onContentErrorDismiss()
		} else if (it.kind === 'fs-op' && it.op.id) {
			const { op } = it
			if (op.state === 'done' || op.state?.startsWith('fail')) {
				tasks.push(ctx.dismissOperation(it.op.id, 'dismiss'))
			}
		} else if (it.kind === 'backup') {
			tasks.push(onBackupDismiss(it.entry))
		}
	}
	await Promise.all(tasks)
}

function onFileOpDismiss(item: ServerAdmonitionItem) {
	if (item.kind === 'fs-op' && item.op.id) {
		void ctx.dismissOperation(item.op.id, 'dismiss')
	}
}

function onContentErrorDismiss() {
	if (contentErrorKey.value) {
		dismissedContentErrorKey.value = contentErrorKey.value
	}
}
</script>

<template>
	<StackedAdmonitions
		:items="stackItems"
		:dismiss-all-enabled="hasBulkDismissableItems"
		class="w-full"
		@dismiss-all="onDismissAll"
	>
		<template #item="{ item, dismissible }">
			<InstallingBanner
				v-if="item.kind === 'installing'"
				:progress="syncProgress"
				:fallback-phase="isOnContentTab && !syncProgress ? 'Addons' : null"
				:content-error="contentError"
				:dismissible="dismissible && !!contentError"
				@dismiss="onContentErrorDismiss"
				@retry="emit('content-retry')"
			/>
			<UploadAdmonition v-else-if="item.kind === 'upload'" />
			<FileOperationAdmonition
				v-else-if="item.kind === 'fs-op'"
				:op="item.op"
				:dismissible="dismissible"
				@dismiss="onFileOpDismiss(item)"
			/>
			<BackupAdmonition
				v-else-if="item.kind === 'backup'"
				:item="item.entry"
				:dismissible="dismissible"
				:cancelling="cancellingIds.has(item.entry.key)"
				@dismiss="onBackupDismiss(item.entry)"
				@cancel="onBackupCancel(item.entry)"
				@retry="onBackupRetry(item.entry)"
			/>
			<Admonition
				v-else-if="item.kind === 'busy-content'"
				type="warning"
				:header="formatMessage(messages.backgroundTaskRunning)"
			>
				{{ formatMessage(messages.contentBusyBody) }}
			</Admonition>
			<Admonition
				v-else-if="item.kind === 'busy-files'"
				type="warning"
				:header="formatMessage(messages.backgroundTaskRunning)"
			>
				{{ formatMessage(messages.filesBusyBody) }}
			</Admonition>
		</template>
	</StackedAdmonitions>
</template>
