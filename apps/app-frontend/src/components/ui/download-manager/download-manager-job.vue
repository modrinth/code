<script setup lang="ts">
import {
	BoxIcon,
	CheckIcon,
	DownloadIcon,
	MinecraftServerIcon,
	PauseIcon,
	PlayIcon,
	TextSearchIcon,
	TrashIcon,
	UpdatedIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	BulletDivider,
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	ProgressBar,
	truncatedTooltip,
	useGeneratedIconSrc,
	useRelativeTime,
	useServerIcon,
	useVIntl,
} from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'
import { RouterLink } from 'vue-router'

import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{ job: DownloadManagerJob }>()
const { config: serverIconConfig, image: serverIconImage } = useServerIcon(
	() => props.job.serverId ?? '',
)
const serverIconGeneratedSrc = useGeneratedIconSrc(serverIconConfig)
const serverIcon = computed(() => serverIconGeneratedSrc.value ?? serverIconImage.value)
const displayIcon = computed(() =>
	props.job.serverId ? (serverIcon.value ?? MinecraftServerIcon) : props.job.iconUrl,
)
defineEmits<{
	retry: [id: string]
	cancel: [id: string]
	togglePause: [id: string]
	dismiss: [id: string]
	copyDetails: [id: string]
	reveal: [id: string]
	open: []
}>()

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow', numeric: 'always' })
const titleRef = useTemplateRef('title')
const textRef = useTemplateRef('text')
const messages = defineMessages({
	pause: { id: 'app.download-manager.pause', defaultMessage: 'Pause installation' },
	resume: { id: 'app.download-manager.resume', defaultMessage: 'Resume installation' },
	retry: { id: 'app.action-bar.install.retry', defaultMessage: 'Retry' },
	inspectDetails: {
		id: 'app.action-bar.install.inspect-details',
		defaultMessage: 'Inspect details',
	},
	copied: { id: 'app.action-bar.install.copied-details', defaultMessage: 'Copied' },
	dismiss: { id: 'app.action-bar.install.dismiss', defaultMessage: 'Dismiss' },
	cancel: { id: 'app.download-manager.cancel', defaultMessage: 'Cancel installation' },
	remove: { id: 'app.download-manager.remove', defaultMessage: 'Remove from history' },
	openInstanceLabel: {
		id: 'app.download-manager.open-instance-label',
		defaultMessage: 'Open instance: {title}',
	},
	openInFolderLabel: {
		id: 'app.download-manager.open-in-folder-label',
		defaultMessage: 'Open in folder: {title}',
	},
	eta: {
		id: 'app.download-manager.eta',
		defaultMessage: 'Estimated time remaining for this download',
	},
})

const needsAttention = computed(
	() => props.job.status === 'failed' || props.job.status === 'interrupted',
)
const complete = computed(() => props.job.status === 'succeeded' || props.job.status === 'canceled')
const cancelLabel = computed(() =>
	formatMessage(
		needsAttention.value
			? messages.dismiss
			: props.job.kind === 'external-file' || props.job.kind === 'debug-export'
				? commonMessages.cancelButton
				: messages.cancel,
	),
)
const instanceLink = computed(() =>
	props.job.status === 'succeeded' && props.job.instanceId && !props.job.revealPath
		? `/instance/${encodeURIComponent(props.job.instanceId)}`
		: undefined,
)
const rowLabel = computed(() =>
	instanceLink.value
		? formatMessage(messages.openInstanceLabel, { title: props.job.title })
		: props.job.revealPath
			? formatMessage(messages.openInFolderLabel, { title: props.job.title })
			: undefined,
)
</script>

<template>
	<div
		class="flex flex-col gap-2.5 rounded-xl p-1.5"
		:class="{ 'hover:bg-surface-4 focus-within:bg-surface-4': complete }"
		:aria-busy="job.busy || undefined"
	>
		<div class="flex items-center justify-between gap-2">
			<component
				:is="instanceLink ? RouterLink : job.revealPath ? 'button' : 'div'"
				:to="instanceLink"
				:type="!instanceLink && job.revealPath ? 'button' : undefined"
				:aria-label="rowLabel"
				class="flex min-w-0 flex-1 items-center gap-2 rounded-lg border-none bg-transparent p-0 text-left text-inherit no-underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand"
				:class="{ 'cursor-pointer': !instanceLink && job.revealPath }"
				@click="instanceLink ? $emit('open') : job.revealPath && $emit('reveal', job.id)"
			>
				<Avatar
					v-if="displayIcon"
					:src="displayIcon"
					size="36px"
					no-shadow
					class="!rounded-xl border border-solid border-surface-5"
				/>
				<div
					v-else
					class="flex size-9 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-purple/10"
				>
					<component
						:is="
							job.kind === 'shared-instance-publish'
								? UploadIcon
								: job.kind === 'external-file' || job.kind === 'debug-export'
									? DownloadIcon
									: BoxIcon
						"
						class="size-6 text-primary"
						aria-hidden="true"
					/>
				</div>
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<span
						ref="title"
						v-tooltip="truncatedTooltip(titleRef, job.title)"
						class="truncate text-base font-semibold leading-5 text-contrast"
					>
						{{ job.title }}
					</span>
					<span
						v-if="job.kind !== 'external-file' && job.taskType && job.taskType !== job.text"
						class="truncate text-xs font-medium leading-4 text-primary"
					>
						{{ job.taskType }}
					</span>
					<div
						class="flex min-w-0 items-center gap-1 text-xs font-medium leading-4"
						:class="
							needsAttention
								? job.status === 'failed'
									? 'text-red'
									: 'text-orange'
								: 'text-primary'
						"
					>
						<template
							v-if="job.kind === 'external-file' && job.taskType && job.taskType !== job.text"
						>
							<span class="min-w-0 truncate text-primary">{{ job.taskType }}</span>
							<BulletDivider class="shrink-0 text-primary" />
						</template>
						<template v-if="complete && job.finishedAt">
							<time :datetime="job.finishedAt" class="shrink-0">
								{{ formatRelativeTime(job.finishedAt) }}
							</time>
							<BulletDivider class="shrink-0" />
						</template>
						<span ref="text" v-tooltip="truncatedTooltip(textRef, job.text)" class="truncate">
							{{ job.text }}
						</span>
					</div>
				</div>
			</component>
			<div class="flex items-center gap-0.5">
				<IconButton
					v-if="job.canPause"
					v-tooltip="formatMessage(job.paused ? messages.resume : messages.pause)"
					:label="formatMessage(job.paused ? messages.resume : messages.pause)"
					type="quiet"
					size="sm"
					:disabled="job.busy"
					@click="$emit('togglePause', job.id)"
				>
					<PlayIcon v-if="job.paused" />
					<PauseIcon v-else />
				</IconButton>
				<IconButton
					v-if="job.canCancel || job.canceling || needsAttention"
					v-tooltip="cancelLabel"
					:label="cancelLabel"
					type="quiet"
					size="sm"
					:disabled="job.busy || job.canceling"
					@click="needsAttention ? $emit('dismiss', job.id) : $emit('cancel', job.id)"
				>
					<XIcon class="text-primary" />
				</IconButton>
				<IconButton
					v-else-if="complete"
					v-tooltip="formatMessage(messages.remove)"
					:label="formatMessage(messages.remove)"
					type="quiet"
					size="sm"
					:disabled="job.busy"
					@click="$emit('dismiss', job.id)"
				>
					<TrashIcon />
				</IconButton>
			</div>
		</div>

		<div v-if="job.status === 'running'" class="flex flex-col gap-2">
			<ProgressBar
				:progress="job.progress"
				:waiting="job.waiting && !job.paused && !job.canceling"
				:label="`${job.title}: ${job.text}`"
				full-width
				class="download-manager-progress"
			/>
			<div
				v-if="job.progressLabel || job.eta"
				class="flex items-center justify-between gap-2 text-xs font-medium leading-4 text-primary tabular-nums"
			>
				<span>{{ job.progressLabel }}</span>
				<span v-if="job.eta" v-tooltip="formatMessage(messages.eta)" class="pr-2">
					{{ job.eta }}
				</span>
			</div>
		</div>

		<div v-if="job.canRetry || job.canCopyDetails" class="flex flex-wrap items-center gap-1">
			<Button
				v-if="job.canCopyDetails"
				type="outlined"
				size="xs"
				:disabled="job.busy"
				@click="$emit('copyDetails', job.id)"
			>
				<CheckIcon v-if="job.copied" />
				<TextSearchIcon v-else />
				{{ formatMessage(job.copied ? messages.copied : messages.inspectDetails) }}
			</Button>
			<Button
				v-if="job.canRetry"
				type="colored"
				color="brand"
				size="xs"
				:disabled="job.busy"
				@click="$emit('retry', job.id)"
			>
				<UpdatedIcon />
				{{ formatMessage(messages.retry) }}
			</Button>
		</div>
	</div>
</template>

<style scoped>
.download-manager-progress :deep(> div:first-child:not([role='progressbar'])) {
	display: none;
}

.download-manager-progress :deep([role='progressbar']) {
	background: var(--surface-2);
}

.download-manager-progress :deep(.progress-bar) {
	border-top-right-radius: 0;
	border-bottom-right-radius: 0;
}
</style>
