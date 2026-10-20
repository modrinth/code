<template>
	<Table
		v-model:selected-ids="tableSelectedIds"
		:columns="columns"
		:data="rows"
		:row-class="rowClass"
		:row-below-visible="showDebugInfo"
		show-selection
		table-min-width="44rem"
	>
		<template #cell-name="{ row: backup }">
			<span
				:data-backup-id="backup.id"
				class="block truncate font-medium text-primary"
				:title="backup.name"
			>
				{{ backup.name }}
			</span>
		</template>

		<template #cell-created="{ row: backup }">
			<span v-tooltip="formatDateTime(backup.created_at)" class="font-medium">
				{{ formatRelativeTime(backup.created_at) }}
			</span>
		</template>

		<template #cell-createdBy="{ row: backup }">
			<span
				v-if="backup.creator"
				class="inline-flex min-w-0 max-w-full items-center gap-2 font-medium text-primary"
			>
				<Avatar
					:src="backup.creator.avatar"
					:alt="formatMessage(messages.creatorAvatarAlt, { username: backup.creator.name })"
					:tint-by="backup.creator.name"
					size="24px"
					circle
					no-shadow
				/>
				<span class="min-w-0 truncate">{{ backup.creator.name }}</span>
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
					type="outlined"
					size="sm"
					:disabled="!!restoreDisabled"
					@click="emit('restore', backup)"
				>
					<RotateCounterClockwiseIcon aria-hidden="true" />
					{{ formatMessage(messages.restore) }}
				</Button>
				<TeleportOverflowMenu
					type="quiet"
					size="sm"
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
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	IntercomBubbleIcon,
	MoreHorizontalIcon,
	RotateCounterClockwiseIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import type { TableColumn } from '#ui/components/base'
import Avatar from '#ui/components/base/Avatar.vue'
import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { Button, TeleportOverflowMenu } from '#ui/components/base/buttons'
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
	downloadHref: (backup: Backup) => string | undefined
}>()

const emit = defineEmits<{
	restore: [backup: Backup]
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
	{ key: 'name', label: formatMessage(messages.nameColumn) },
	{ key: 'created', label: formatMessage(messages.createdColumn), width: '20%' },
	{ key: 'createdBy', label: formatMessage(messages.createdByColumn), width: '24%' },
	{
		key: 'actions',
		label: formatMessage(commonMessages.actionsLabel),
		width: '11rem',
		align: 'right',
	},
])

const rows = computed(() =>
	props.backups.map((backup) => {
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
					}
				: null,
		}
	}),
)

function rowClass(backup: Backup) {
	return backup.id === props.highlightedId ? 'backup-row-highlighted' : ''
}

function overflowOptions(backup: Backup): ButtonMenuOption[] {
	const writeTooltip = props.canManage ? undefined : props.writeDisabledTooltip
	const downloadHref = props.downloadHref(backup)

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
			type: 'link',
			label: formatMessage(commonMessages.downloadButton),
			icon: DownloadIcon,
			href: downloadHref,
			disabled: !downloadHref,
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
