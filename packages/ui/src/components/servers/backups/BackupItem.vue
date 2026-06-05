<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	IntercomBubbleIcon,
	MoreVerticalIcon,
	RotateCounterClockwiseIcon,
	ShieldIcon,
	TrashIcon,
	UserRoundIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { useFormatDateTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages, truncatedTooltip } from '../../../utils'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import OverflowMenu, { type Option as OverflowOption } from '../../base/OverflowMenu.vue'

const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const emit = defineEmits<{
	(e: 'download' | 'rename' | 'restore'): void
	(e: 'delete', skipConfirmation?: boolean): void
}>()

const props = withDefaults(
	defineProps<{
		backup: Archon.BackupsQueue.v1.BackupQueueBackup
		creator?: Archon.BackupsQueue.v1.UserInfo | null
		preview?: boolean
		kyrosUrl?: string
		jwt?: string
		showCopyIdAction?: boolean
		showDebugInfo?: boolean
		restoreDisabled?: string
		writeDisabled?: boolean
		writeDisabledTooltip?: string
		selected?: boolean
		highlighted?: boolean
	}>(),
	{
		creator: undefined,
		preview: false,
		kyrosUrl: undefined,
		jwt: undefined,
		showCopyIdAction: false,
		showDebugInfo: false,
		restoreDisabled: undefined,
		writeDisabled: false,
		writeDisabledTooltip: undefined,
		selected: false,
		highlighted: false,
	},
)

const nameRef = ref<HTMLElement | null>(null)

const backupCreator = computed(() => {
	if (props.creator !== undefined) return props.creator

	return (
		props.backup.history.find(
			(operation) => operation.operation_type === 'create' && operation.user_info,
		)?.user_info ?? null
	)
})

const creatorProfileLink = computed(() =>
	backupCreator.value && backupCreator.value.id !== 'support'
		? `https://modrinth.com/user/${encodeURIComponent(backupCreator.value.username)}`
		: undefined,
)

const backupIcon = computed(() => {
	if (props.backup.automated) {
		return ShieldIcon
	}
	return UserRoundIcon
})

const itemBorderClass = computed(() => {
	if (props.selected) return 'border-brand-green'
	if (props.highlighted) return 'border-purple backup-item-highlighted'
	return 'border-transparent'
})

const overflowMenuOptions = computed<OverflowOption[]>(() => {
	const options: OverflowOption[] = []

	if (props.showCopyIdAction) {
		options.push({
			id: 'copy-id',
			action: () => copyId(),
		})
	}

	if (options.length > 0) {
		options.push({ divider: true })
	}

	options.push({
		id: 'download',
		action: () => emit('download'),
		link: `https://${props.kyrosUrl}/modrinth/v0/backups/${props.backup.id}/download?auth=${props.jwt}`,
		disabled: !props.kyrosUrl || !props.jwt,
	})

	options.push({
		id: 'rename',
		action: () => emit('rename'),
		disabled: props.writeDisabled,
		tooltip: props.writeDisabled ? props.writeDisabledTooltip : undefined,
	})

	options.push({ divider: true })
	options.push({
		id: 'delete',
		color: 'red',
		action: () => emit('delete'),
		disabled: props.writeDisabled,
		tooltip: props.writeDisabled ? props.writeDisabledTooltip : undefined,
	})

	return options
})

async function copyId() {
	await navigator.clipboard.writeText(props.backup.id)
}

const messages = defineMessages({
	restore: {
		id: 'servers.backups.item.restore',
		defaultMessage: 'Restore',
	},
	rename: {
		id: 'servers.backups.item.rename',
		defaultMessage: 'Rename',
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
	creatorAvatarAlt: {
		id: 'servers.backups.item.creator-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	supportCreator: {
		id: 'servers.backups.item.creator.support',
		defaultMessage: 'Support',
	},
})

const creatorName = computed(() => {
	if (!backupCreator.value) return ''
	if (backupCreator.value.id !== 'support') return backupCreator.value.username
	return backupCreator.value.username === 'support'
		? formatMessage(messages.supportCreator)
		: backupCreator.value.username
})

const creatorAvatarSrc = computed(() =>
	backupCreator.value?.id === 'support'
		? IntercomBubbleIcon
		: (backupCreator.value?.avatar_url ?? undefined),
)
</script>
<template>
	<div
		class="flex items-center gap-4 rounded-[20px] border border-solid bg-surface-3 p-4 shadow-[0px_1px_2px_0px_rgba(0,0,0,0.3),0px_1px_3px_0px_rgba(0,0,0,0.15)]"
		:class="itemBorderClass"
	>
		<div class="flex min-w-0 flex-1 items-center gap-4">
			<!-- Icon tile -->
			<div
				class="flex shrink-0 items-center justify-center rounded-2xl border border-solid border-surface-5 bg-surface-4"
				:class="preview ? 'size-10' : 'size-14'"
			>
				<component
					:is="backupIcon"
					class="text-secondary"
					:class="preview ? 'size-6' : 'size-10'"
				/>
			</div>

			<!-- Name + badge + subtitle -->
			<div class="flex min-w-0 flex-col gap-1.5">
				<div class="flex min-w-0 items-center gap-2">
					<span
						ref="nameRef"
						v-tooltip="truncatedTooltip(nameRef, backup.name)"
						class="min-w-0 truncate font-semibold text-contrast"
					>
						{{ backup.name }}
					</span>
					<span
						v-if="backup.automated"
						class="shrink-0 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium text-secondary"
					>
						{{ formatMessage(messages.auto) }}
					</span>
				</div>
				<div class="flex items-center gap-2 text-sm font-medium text-secondary">
					<template v-if="preview">
						<span>{{ formatDateTime(backup.created_at) }}</span>
					</template>
					<template v-else-if="backupCreator">
						<AutoLink
							:to="creatorProfileLink"
							:target="creatorProfileLink ? '_blank' : undefined"
							:rel="creatorProfileLink ? 'noopener noreferrer' : undefined"
							class="group flex min-w-0 items-center gap-1.5"
							:class="creatorProfileLink ? 'text-secondary hover:underline' : 'text-primary'"
						>
							<Avatar
								:src="creatorAvatarSrc"
								:alt="formatMessage(messages.creatorAvatarAlt, { username: creatorName })"
								:tint-by="creatorName"
								size="24px"
								circle
								no-shadow
								class="shrink-0 transition"
								:class="creatorProfileLink ? 'group-hover:brightness-125' : ''"
							/>
							<span
								class="min-w-0 truncate font-medium"
								:class="backupCreator.id === 'support' ? 'text-blue' : ''"
							>
								{{ creatorName }}
							</span>
						</AutoLink>
					</template>
					<template v-else>
						<span>
							{{
								formatMessage(backup.automated ? messages.backupSchedule : messages.manualBackup)
							}}
						</span>
					</template>
				</div>
			</div>
		</div>

		<!-- Date (middle column) -->
		<div v-if="!preview" class="flex shrink-0 items-center">
			<span class="whitespace-nowrap font-medium text-contrast">{{
				formatDateTime(backup.created_at)
			}}</span>
		</div>

		<!-- Right side actions -->
		<div v-if="!preview" class="flex min-w-0 flex-1 items-center justify-end gap-2">
			<ButtonStyled color="brand" type="outlined">
				<button
					v-tooltip="props.restoreDisabled"
					class="!border"
					:disabled="!!props.restoreDisabled"
					@click="() => emit('restore')"
				>
					<RotateCounterClockwiseIcon class="size-5" />
					{{ formatMessage(messages.restore) }}
				</button>
			</ButtonStyled>
			<ButtonStyled circular type="transparent">
				<OverflowMenu :options="overflowMenuOptions">
					<MoreVerticalIcon class="size-5" />
					<template #copy-id>
						<ClipboardCopyIcon class="size-5" />
						{{ formatMessage(commonMessages.copyIdButton) }}
					</template>
					<template #download>
						<DownloadIcon class="size-5" /> {{ formatMessage(commonMessages.downloadButton) }}
					</template>
					<template #rename>
						<EditIcon class="size-5" /> {{ formatMessage(messages.rename) }}
					</template>
					<template #delete>
						<TrashIcon class="size-5" /> {{ formatMessage(commonMessages.deleteLabel) }}
					</template>
				</OverflowMenu>
			</ButtonStyled>
		</div>

		<pre v-if="!preview && showDebugInfo" class="w-full rounded-xl bg-surface-4 p-2 text-xs">{{
			backup
		}}</pre>
	</div>
</template>

<style scoped>
@keyframes backup-item-highlight-pulse {
	0%,
	100% {
		box-shadow:
			0 0 0 0 var(--color-purple-highlight),
			0px 1px 2px 0px rgba(0, 0, 0, 0.3),
			0px 1px 3px 0px rgba(0, 0, 0, 0.15);
	}
	50% {
		box-shadow:
			0 0 0 3px var(--color-purple-highlight),
			0px 1px 2px 0px rgba(0, 0, 0, 0.3),
			0px 1px 3px 0px rgba(0, 0, 0, 0.15);
	}
}

.backup-item-highlighted {
	animation: backup-item-highlight-pulse 1.25s ease-in-out infinite;
}
</style>
