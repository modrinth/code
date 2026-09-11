<script setup lang="ts">
import {
	BoxIcon,
	CheckIcon,
	CopyIcon,
	PauseIcon,
	PlayIcon,
	TrashIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	IconButton,
	ProgressBar,
	truncatedTooltip,
	useVIntl,
} from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'
import { RouterLink } from 'vue-router'

import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{ job: DownloadManagerJob }>()
defineEmits<{
	retry: [id: string]
	cancel: [id: string]
	togglePause: [id: string]
	dismiss: [id: string]
	copyDetails: [id: string]
	open: []
}>()

const { formatMessage } = useVIntl()
const titleRef = useTemplateRef('title')
const textRef = useTemplateRef('text')
const messages = defineMessages({
	pause: { id: 'app.download-manager.pause', defaultMessage: 'Pause installation' },
	resume: { id: 'app.download-manager.resume', defaultMessage: 'Resume installation' },
	retry: { id: 'app.action-bar.install.retry', defaultMessage: 'Retry' },
	copyDetails: { id: 'app.action-bar.install.copy-details', defaultMessage: 'Copy details' },
	copied: { id: 'app.action-bar.install.copied-details', defaultMessage: 'Copied' },
	dismiss: { id: 'app.action-bar.install.dismiss', defaultMessage: 'Dismiss' },
	cancel: { id: 'app.download-manager.cancel', defaultMessage: 'Cancel installation' },
	remove: { id: 'app.download-manager.remove', defaultMessage: 'Remove from history' },
	openInstance: { id: 'app.action-bar.install.open-instance', defaultMessage: 'Open instance' },
	eta: {
		id: 'app.download-manager.eta',
		defaultMessage: 'Estimated time remaining for this download',
	},
})

const needsAttention = computed(
	() => props.job.status === 'failed' || props.job.status === 'interrupted',
)
const complete = computed(() => props.job.status === 'succeeded' || props.job.status === 'canceled')
const instanceLink = computed(() =>
	props.job.status === 'succeeded' && props.job.instanceId
		? `/instance/${encodeURIComponent(props.job.instanceId)}`
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
				:is="instanceLink ? RouterLink : 'div'"
				:to="instanceLink"
				:aria-label="
					instanceLink ? `${formatMessage(messages.openInstance)}: ${job.title}` : undefined
				"
				class="flex min-w-0 flex-1 items-center gap-2 rounded-lg text-inherit no-underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand"
				@click="instanceLink && $emit('open')"
			>
				<Avatar
					v-if="job.iconUrl"
					:src="job.iconUrl"
					size="36px"
					no-shadow
					class="!rounded-xl border border-solid border-surface-5"
				/>
				<div
					v-else
					class="flex size-9 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-purple/10"
				>
					<BoxIcon class="size-6 text-primary" aria-hidden="true" />
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
						ref="text"
						v-tooltip="truncatedTooltip(textRef, job.text)"
						class="truncate text-xs font-medium leading-4"
						:class="
							needsAttention
								? job.status === 'failed'
									? 'text-red'
									: 'text-orange'
								: 'text-primary'
						"
					>
						{{ job.text }}
					</span>
				</div>
			</component>
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
				v-tooltip="formatMessage(needsAttention ? messages.dismiss : messages.cancel)"
				:label="formatMessage(needsAttention ? messages.dismiss : messages.cancel)"
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

		<div v-if="needsAttention || job.canCopyDetails" class="flex flex-wrap items-center gap-1">
			<Button
				v-if="job.canCopyDetails"
				type="outlined"
				size="xs"
				:disabled="job.busy"
				@click="$emit('copyDetails', job.id)"
			>
				<CheckIcon v-if="job.copied" />
				<CopyIcon v-else />
				{{ formatMessage(job.copied ? messages.copied : messages.copyDetails) }}
			</Button>
			<Button
				v-if="needsAttention"
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
