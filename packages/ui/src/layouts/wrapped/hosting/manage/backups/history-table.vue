<template>
	<div class="@container">
		<Table
			v-model:selected-ids="tableSelectedIds"
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="rows"
			:row-class="rowClass"
			:row-below-visible="showDebugInfo"
			show-selection
			table-min-width="40rem"
			class="hidden @[44rem]:block"
		>
			<template #cell-name="{ row: backup }">
				<span :data-backup-id="backup.id" class="block truncate text-primary" :title="backup.name">
					{{ backup.name }}
				</span>
			</template>

			<template #cell-created="{ row: backup }">
				<span v-tooltip="formatDateTime(backup.created_at)" class="whitespace-nowrap">
					{{ formatRelativeTime(backup.created_at) }}
				</span>
			</template>

			<template #cell-createdBy="{ row: backup }">
				<span
					v-if="backup.creator"
					class="inline-flex min-w-0 max-w-full items-center gap-2 whitespace-nowrap text-primary"
				>
					<Avatar
						:src="backup.creator.avatar"
						:alt="formatMessage(messages.creatorAvatarAlt, { username: backup.creator.name })"
						:tint-by="backup.creator.name"
						size="22px"
						circle
						no-shadow
					/>
					<span
						class="min-w-0 truncate font-medium"
						:class="backup.creator.isSupport ? 'text-blue' : ''"
					>
						{{ backup.creator.name }}
					</span>
				</span>
				<span
					v-else-if="backup.automated"
					class="inline-flex h-7 items-center rounded-full border border-solid border-blue bg-highlight-blue px-2.5 text-sm font-semibold text-blue"
				>
					{{ formatMessage(messages.autoBadge) }}
				</span>
			</template>

			<template #cell-actions="{ row: backup }">
				<div class="flex items-center justify-end gap-1">
					<Button
						v-tooltip="restoreDisabled"
						:disabled="!!restoreDisabled"
						@click="emit('restore', backup)"
					>
						<RefreshCwIcon aria-hidden="true" />
						{{ formatMessage(messages.restore) }}
					</Button>
					<TeleportOverflowMenu
						type="quiet"
						:label="formatMessage(messages.moreOptions, { name: backup.name })"
						:options="overflowOptions(backup)"
					>
						<MoreHorizontalIcon aria-hidden="true" />
					</TeleportOverflowMenu>
				</div>
			</template>

			<template #row-below="{ row: backup }">
				<pre class="m-2 overflow-x-auto rounded-xl bg-surface-3 p-2 text-xs">{{ backup }}</pre>
			</template>

			<template #empty-state>
				<div class="flex h-40 items-center justify-center text-secondary">
					{{ formatMessage(messages.noResults) }}
				</div>
			</template>
		</Table>

		<div v-if="rows.length > 0" class="flex flex-col gap-3 @[44rem]:hidden">
			<div
				v-for="backup in rows"
				:key="backup.id"
				class="flex min-w-0 flex-col gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4"
				:class="rowClass(backup)"
			>
				<div class="flex min-w-0 items-center gap-3">
					<Checkbox
						:model-value="selectedIds.includes(backup.id)"
						class="shrink-0"
						@update:model-value="(value) => toggleSelected(backup.id, value)"
					/>
					<div class="flex min-w-0 flex-1 flex-col gap-1">
						<span
							:data-backup-id="backup.id"
							class="truncate font-medium text-primary"
							:title="backup.name"
						>
							{{ backup.name }}
						</span>
						<div class="flex min-w-0 items-center gap-1.5 text-sm text-secondary">
							<span
								v-if="backup.creator"
								class="inline-flex min-w-0 items-center gap-1.5 text-primary"
							>
								<Avatar
									:src="backup.creator.avatar"
									:alt="formatMessage(messages.creatorAvatarAlt, { username: backup.creator.name })"
									:tint-by="backup.creator.name"
									size="18px"
									circle
									no-shadow
								/>
								<span
									class="min-w-0 truncate font-medium"
									:class="backup.creator.isSupport ? 'text-blue' : ''"
								>
									{{ backup.creator.name }}
								</span>
							</span>
							<span v-else-if="backup.automated" class="shrink-0 font-semibold text-blue">
								{{ formatMessage(messages.autoBadge) }}
							</span>
							<BulletDivider v-if="backup.creator || backup.automated" class="shrink-0" />
							<span
								v-tooltip="formatDateTime(backup.created_at)"
								class="shrink-0 whitespace-nowrap"
							>
								{{ formatRelativeTime(backup.created_at) }}
							</span>
						</div>
					</div>
					<div class="flex shrink-0 items-center gap-1">
						<IconButton
							v-tooltip="restoreDisabled ?? formatMessage(messages.restore)"
							:label="formatMessage(messages.restore)"
							:circular="false"
							:disabled="!!restoreDisabled"
							@click="emit('restore', backup)"
						>
							<RefreshCwIcon aria-hidden="true" />
						</IconButton>
						<TeleportOverflowMenu
							type="quiet"
							:label="formatMessage(messages.moreOptions, { name: backup.name })"
							:options="overflowOptions(backup)"
						>
							<MoreHorizontalIcon aria-hidden="true" />
						</TeleportOverflowMenu>
					</div>
				</div>
				<pre v-if="showDebugInfo" class="m-0 overflow-x-auto rounded-xl bg-surface-3 p-2 text-xs">{{
					backup
				}}</pre>
			</div>
		</div>
		<div
			v-else
			class="flex h-40 items-center justify-center rounded-2xl border border-solid border-surface-4 text-secondary @[44rem]:hidden"
		>
			{{ formatMessage(messages.noResults) }}
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	IntercomBubbleIcon,
	MoreHorizontalIcon,
	RefreshCwIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import type { SortDirection, TableColumn } from '#ui/components/base'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { Button, IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import Table from '#ui/components/base/Table.vue'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useRelativeTime } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

type Backup = Archon.BackupsQueue.v1.BackupQueueBackup

const props = defineProps<{
	backups: Backup[]
	restoreDisabled?: string
	writeDisabledTooltip?: string
	canManage: boolean
	highlightedId?: string | null
	showCopyIdAction?: boolean
	showDebugInfo?: boolean
	canDownload: boolean
}>()

const emit = defineEmits<{
	restore: [backup: Backup]
	download: [backup: Backup]
	rename: [backup: Backup]
	delete: [backup: Backup]
}>()

const selectedIds = defineModel<string[]>('selectedIds', { required: true })
const tableSelectedIds = computed<unknown[]>({
	get: () => selectedIds.value,
	set: (ids) => {
		selectedIds.value = ids.filter((id): id is string => typeof id === 'string')
	},
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'long', timeStyle: 'short' })

const messages = defineMessages({
	nameColumn: {
		id: 'servers.backups.history.column.name',
		defaultMessage: 'Name',
	},
	createdColumn: {
		id: 'servers.backups.history.column.created',
		defaultMessage: 'Created',
	},
	createdByColumn: {
		id: 'servers.backups.history.column.created-by',
		defaultMessage: 'Created by',
	},
	autoBadge: {
		id: 'servers.backups.history.auto-badge',
		defaultMessage: 'Auto',
	},
	support: {
		id: 'servers.backups.history.creator.support',
		defaultMessage: 'Support',
	},
	creatorAvatarAlt: {
		id: 'servers.backups.history.creator-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	restore: {
		id: 'servers.backups.history.restore',
		defaultMessage: 'Restore',
	},
	rename: {
		id: 'servers.backups.history.rename',
		defaultMessage: 'Rename',
	},
	moreOptions: {
		id: 'servers.backups.history.more-options',
		defaultMessage: 'More options for {name}',
	},
	noResults: {
		id: 'servers.backups.history.no-results',
		defaultMessage: 'No backups match your search.',
	},
})

const columns = computed<TableColumn<'name' | 'created' | 'createdBy' | 'actions'>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.nameColumn),
		enableSorting: true,
		cellClass: 'pr-4',
	},
	{
		key: 'created',
		label: formatMessage(messages.createdColumn),
		width: 'max(8rem, 20%)',
		enableSorting: true,
		defaultSortDirection: 'desc',
		cellClass: 'pr-4',
	},
	{
		key: 'createdBy',
		label: formatMessage(messages.createdByColumn),
		width: 'max(10rem, 22%)',
		cellClass: 'pr-4',
	},
	{
		key: 'actions',
		label: formatMessage(commonMessages.actionsLabel),
		width: '10.5rem',
		align: 'right',
	},
])

const sortColumn = ref<string | undefined>('created')
const sortDirection = ref<SortDirection>('desc')

const sortedBackups = computed(() => {
	const direction = sortDirection.value === 'asc' ? 1 : -1
	return [...props.backups].sort((a, b) => {
		const order =
			sortColumn.value === 'name'
				? a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
				: new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
		return order * direction
	})
})

const rows = computed(() =>
	sortedBackups.value.map((backup) => {
		const user = backup.history.find(
			(operation) => operation.operation_type === 'create' && operation.user_info,
		)?.user_info
		const isSupport = user?.id === 'support'

		return {
			...backup,
			creator: user
				? {
						name: isSupport ? formatMessage(messages.support) : user.username,
						avatar: isSupport ? IntercomBubbleIcon : (user.avatar_url ?? undefined),
						isSupport,
					}
				: null,
		}
	}),
)

function toggleSelected(id: string, selected: boolean) {
	selectedIds.value = selected
		? [...selectedIds.value, id]
		: selectedIds.value.filter((selectedId) => selectedId !== id)
}

function rowClass(backup: Backup) {
	return backup.id === props.highlightedId ? 'backup-row-highlighted' : ''
}

function overflowOptions(backup: Backup): ButtonMenuOption[] {
	const writeTooltip = props.canManage ? undefined : props.writeDisabledTooltip

	return [
		{
			id: 'copy-id',
			label: formatMessage(commonMessages.copyIdButton),
			icon: ClipboardCopyIcon,
			shown: !!props.showCopyIdAction,
			action: () => navigator.clipboard.writeText(backup.id),
		},
		{ type: 'divider', shown: !!props.showCopyIdAction },
		{
			id: 'download',
			label: formatMessage(commonMessages.downloadButton),
			icon: DownloadIcon,
			disabled: !props.canDownload,
			action: () => emit('download', backup),
		},
		{
			id: 'rename',
			label: formatMessage(messages.rename),
			icon: EditIcon,
			disabled: !props.canManage,
			tooltip: writeTooltip,
			action: () => emit('rename', backup),
		},
		{ type: 'divider' },
		{
			id: 'delete',
			label: formatMessage(commonMessages.deleteLabel),
			icon: TrashIcon,
			tone: 'red',
			disabled: !props.canManage,
			tooltip: writeTooltip,
			action: () => emit('delete', backup),
		},
	]
}
</script>

<style scoped>
@keyframes backup-row-highlight {
	0%,
	100% {
		background-color: transparent;
	}
	50% {
		background-color: var(--color-purple-highlight);
	}
}

:deep(.backup-row-highlighted) {
	animation: backup-row-highlight 1.25s ease-in-out infinite;
}
</style>
