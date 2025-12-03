<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	ClockIcon,
	DownloadIcon,
	EditIcon,
	LockIcon,
	LockOpenIcon,
	MoreVerticalIcon,
	RotateCounterClockwiseIcon,
	ShieldIcon,
	SpinnerIcon,
	TrashIcon,
	UserIcon,
	XIcon,
} from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { commonMessages } from '../../../utils'
import ButtonStyled from '../../base/ButtonStyled.vue'
import OverflowMenu, { type Option as OverflowOption } from '../../base/OverflowMenu.vue'
import ProgressBar from '../../base/ProgressBar.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'download' | 'rename' | 'restore' | 'lock' | 'retry'): void
	(e: 'delete', skipConfirmation?: boolean): void
}>()

const props = withDefaults(
	defineProps<{
		backup: Archon.Backups.v1.Backup
		preview?: boolean
		kyrosUrl?: string
		jwt?: string
		showDebugInfo?: boolean
		disabled?: string
	}>(),
	{
		preview: false,
		kyrosUrl: undefined,
		jwt: undefined,
		showDebugInfo: false,
		disabled: undefined,
	},
)

const backupQueued = computed(
	() =>
		props.backup.task?.create?.progress === 0 ||
		(props.backup.ongoing && !props.backup.task?.create),
)
// const automated = computed(() => props.backup.automated)
const failedToCreate = computed(() => props.backup.interrupted)

const inactiveStates = ['failed', 'cancelled', 'done']

const creating = computed(() => {
	const task = props.backup.task?.create
	if (task && task.progress < 1 && !inactiveStates.includes(task.state)) {
		return task
	}

	if (props.backup.ongoing && !props.backup.task?.restore) {
		return {
			progress: 0,
			state: 'ongoing',
		}
	}
	return undefined
})

const restoring = computed(() => {
	const task = props.backup.task?.restore
	if (task && task.progress < 1 && !inactiveStates.includes(task.state)) {
		return task
	}

	if (props.backup.ongoing && props.backup.task?.restore) {
		return {
			progress: 0,
			state: 'ongoing',
		}
	}
	return undefined
})

const restoreQueued = computed(() => restoring.value?.progress === 0)

const failedToRestore = computed(() => props.backup.task?.restore?.state === 'failed')

const backupIcon = computed(() => {
	if (props.backup.automated) {
		return props.backup.locked ? ShieldIcon : ClockIcon
	}
	return UserIcon
})

const overflowMenuOptions = computed<OverflowOption[]>(() => {
	const options: OverflowOption[] = []

	// Download only available when not creating
	if (!creating.value) {
		options.push({
			id: 'download',
			action: () => emit('download'),
			link: `https://${props.kyrosUrl}/modrinth/v0/backups/${props.backup.id}/download?auth=${props.jwt}`,
			disabled: !props.kyrosUrl || !props.jwt,
		})
	}

	options.push({ id: 'rename', action: () => emit('rename') })
	options.push({ id: 'lock', action: () => emit('lock') })

	// Delete only available when not creating (has separate Cancel button)
	if (!creating.value) {
		options.push({ divider: true })
		options.push({
			id: 'delete',
			color: 'red',
			action: () => emit('delete'),
			disabled: !!props.disabled,
		})
	}

	return options
})

// TODO: Uncomment when API supports size field
// const formatBytes = (bytes?: number) => {
// 	if (!bytes) return ''
// 	const mb = bytes / (1024 * 1024)
// 	return `${mb.toFixed(0)} MiB`
// }

const messages = defineMessages({
	locked: {
		id: 'servers.backups.item.locked',
		defaultMessage: 'Locked',
	},
	lock: {
		id: 'servers.backups.item.lock',
		defaultMessage: 'Lock',
	},
	unlock: {
		id: 'servers.backups.item.unlock',
		defaultMessage: 'Unlock',
	},
	restore: {
		id: 'servers.backups.item.restore',
		defaultMessage: 'Restore',
	},
	rename: {
		id: 'servers.backups.item.rename',
		defaultMessage: 'Rename',
	},
	queuedForBackup: {
		id: 'servers.backups.item.queued-for-backup',
		defaultMessage: 'Backup queued',
	},
	queuedForRestore: {
		id: 'servers.backups.item.queued-for-restore',
		defaultMessage: 'Restore queued',
	},
	creatingBackup: {
		id: 'servers.backups.item.creating-backup',
		defaultMessage: 'Creating backup...',
	},
	restoringBackup: {
		id: 'servers.backups.item.restoring-backup',
		defaultMessage: 'Restoring from backup...',
	},
	failedToCreateBackup: {
		id: 'servers.backups.item.failed-to-create-backup',
		defaultMessage: 'Failed to create backup',
	},
	failedToRestoreBackup: {
		id: 'servers.backups.item.failed-to-restore-backup',
		defaultMessage: 'Failed to restore from backup',
	},
	auto: {
		id: 'servers.backups.item.auto',
		defaultMessage: 'Auto',
	},
	backupSchedule: {
		id: 'servers.backups.item.backup-schedule',
		defaultMessage: 'Backup schedule',
	},
	manualBackup: {
		id: 'servers.backups.item.manual-backup',
		defaultMessage: 'Manual backup',
	},
	retry: {
		id: 'servers.backups.item.retry',
		defaultMessage: 'Retry',
	},
})
</script>
<template>
	<div
		class="grid items-center gap-4 rounded-2xl bg-bg-raised p-4 shadow-md"
		:class="preview ? 'grid-cols-2' : 'grid-cols-[auto_1fr_auto] md:grid-cols-[400px_1fr_auto]'"
	>
		<div class="flex flex-row gap-4 items-center">
			<div
				class="flex size-12 shrink-0 items-center justify-center rounded-2xl border-solid border-[1px] border-surface-5 bg-surface-4 md:size-16"
			>
				<component :is="backupIcon" class="size-7 text-secondary md:size-10" />
			</div>

			<div class="flex min-w-0 flex-col gap-1.5">
				<div class="flex flex-wrap items-center gap-2">
					<span class="truncate font-semibold text-contrast max-w-[400px]">{{ backup.name }}</span>
					<span
						v-if="backup.automated"
						class="rounded-full border-solid border-[1px] border-surface-5 bg-surface-4 px-2.5 py-1 text-sm text-secondary"
					>
						{{ formatMessage(messages.auto) }}
					</span>
					<span v-if="backup.locked" class="flex items-center gap-1 text-sm text-secondary">
						<LockIcon class="size-4" />
					</span>
				</div>
				<div class="flex items-center gap-1.5 text-sm text-secondary">
					<template v-if="failedToCreate || failedToRestore">
						<XIcon class="size-4 text-red" />
						<span class="text-red">
							{{
								formatMessage(
									failedToCreate ? messages.failedToCreateBackup : messages.failedToRestoreBackup,
								)
							}}
						</span>
					</template>
					<template v-else>
						<!-- TODO: Uncomment when API supports creator_id field -->
						<!-- <template v-if="backup.creator_id && backup.creator_id !== 'auto'">
						<Avatar ... class="size-6 rounded-full" />
						<span>{{ creatorName }}</span>
					</template>
					<template v-else> -->
						<span>
							{{
								formatMessage(backup.automated ? messages.backupSchedule : messages.manualBackup)
							}}
						</span>
						<!-- </template> -->
					</template>
				</div>
			</div>
		</div>

		<div
			class="col-span-full row-start-2 flex flex-col gap-2 md:col-span-1 md:row-start-auto md:mr-16 max-w-[400px]"
		>
			<template v-if="creating">
				<div class="flex items-center justify-between">
					<span class="text-contrast">
						{{ formatMessage(backupQueued ? messages.queuedForBackup : messages.creatingBackup) }}
					</span>
					<div class="flex items-center gap-1 text-sm text-secondary">
						<span>{{ Math.round(creating.progress * 100) }}%</span>
						<SpinnerIcon class="size-5 animate-spin" />
					</div>
				</div>
				<ProgressBar
					:progress="creating.progress"
					:color="backupQueued ? 'orange' : 'brand'"
					:waiting="creating.progress === 0"
					full-width
				/>
			</template>
			<template v-else-if="restoring">
				<div class="flex items-center justify-between">
					<span class="text-purple">
						{{
							formatMessage(restoreQueued ? messages.queuedForRestore : messages.restoringBackup)
						}}
					</span>
					<div class="flex items-center gap-1 text-sm text-secondary">
						<span>{{ Math.round(restoring.progress * 100) }}%</span>
						<SpinnerIcon class="size-5 animate-spin" />
					</div>
				</div>
				<ProgressBar
					full-width
					:progress="restoring.progress"
					color="purple"
					:waiting="restoring.progress === 0"
				/>
			</template>
			<template v-else>
				<span class="font-medium text-contrast">
					{{ dayjs(backup.created_at).format('MMMM Do YYYY, h:mm A') }}
				</span>
				<!-- TODO: Uncomment when API supports size field -->
				<!-- <span class="text-secondary">{{ formatBytes(backup.size) }}</span> -->
			</template>
		</div>

		<div v-if="!preview" class="flex shrink-0 items-center gap-2">
			<template v-if="failedToCreate">
				<ButtonStyled>
					<button @click="() => emit('retry')">
						<RotateCounterClockwiseIcon class="size-5" />
						{{ formatMessage(messages.retry) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="() => emit('delete', true)">
						<TrashIcon class="size-5" />
						{{ formatMessage(commonMessages.deleteLabel) }}
					</button>
				</ButtonStyled>
			</template>
			<template v-else-if="creating">
				<ButtonStyled type="outlined">
					<button class="!border-[1px] !border-surface-5" @click="() => emit('delete')">
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu :options="overflowMenuOptions">
						<MoreVerticalIcon class="size-5" />
						<template #rename>
							<EditIcon class="size-5" /> {{ formatMessage(messages.rename) }}
						</template>
						<template v-if="backup.locked" #lock>
							<LockOpenIcon class="size-5" /> {{ formatMessage(messages.unlock) }}
						</template>
						<template v-else #lock>
							<LockIcon class="size-5" /> {{ formatMessage(messages.lock) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>
			<template v-else>
				<ButtonStyled color="brand" type="outlined">
					<button
						v-tooltip="props.disabled"
						class="!border-[1px]"
						:disabled="!!props.disabled"
						@click="() => emit('restore')"
					>
						<RotateCounterClockwiseIcon class="size-5" />
						{{ formatMessage(messages.restore) }}
					</button>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu :options="overflowMenuOptions">
						<MoreVerticalIcon class="size-5" />
						<template #download>
							<DownloadIcon class="size-5" /> {{ formatMessage(commonMessages.downloadButton) }}
						</template>
						<template #rename>
							<EditIcon class="size-5" /> {{ formatMessage(messages.rename) }}
						</template>
						<template v-if="backup.locked" #lock>
							<LockOpenIcon class="size-5" /> {{ formatMessage(messages.unlock) }}
						</template>
						<template v-else #lock>
							<LockIcon class="size-5" /> {{ formatMessage(messages.lock) }}
						</template>
						<template #delete>
							<TrashIcon class="size-5" /> {{ formatMessage(commonMessages.deleteLabel) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>
		</div>

		<pre v-if="!preview && showDebugInfo" class="w-full rounded-xl bg-surface-4 p-2 text-xs">{{
			backup
		}}</pre>
	</div>
</template>
