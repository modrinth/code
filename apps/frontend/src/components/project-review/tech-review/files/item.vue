<script setup lang="ts">
import { ExternalIcon, ShieldAlertIcon } from '@modrinth/assets'
import { BulletDivider, TagItem } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { formatBytes } from '../../format'
import { getHighestSeverity, getVersionPageHref } from '../helpers'
import { severityPresentation } from '../traces/presentation'
import type { FlattenedFileReport } from '../types'
import { injectTechReviewDecisions } from '../verdict/use-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	selected?: boolean
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const { project } = injectProjectReviewPageContext()
const { getDetailDecision, isDetailGloballyPassed } = injectTechReviewDecisions()
const details = computed(() => props.file.issues.flatMap((issue) => issue.details))
const hasFlags = computed(() => details.value.length > 0)
const hiddenTraceCount = computed(
	() => details.value.filter((detail) => detail.severity === 'hidden').length,
)
const canSelect = computed(() => !props.selected)
const isManualReview = computed(() => props.file.flag_reason.toString() === 'manual')
const fileSizeLabel = computed(() => formatBytes(props.file.file_size))
const versionHref = computed(() =>
	project.value ? getVersionPageHref(project.value, props.file.version_id) : undefined,
)

const reviewableDetails = computed(() =>
	details.value.filter((detail) => !isDetailGloballyPassed(detail)),
)
const globallyPassedCount = computed(() => details.value.length - reviewableDetails.value.length)
const pendingDetails = computed(() =>
	reviewableDetails.value.filter(
		(detail) => getDetailDecision(detail.id, detail.status) === 'pending',
	),
)
const reviewedCount = computed(() => reviewableDetails.value.length - pendingDetails.value.length)
const hasUnsafeTraces = computed(() =>
	details.value.some((detail) => getDetailDecision(detail.id, detail.status) === 'malware'),
)
const severity = computed(() => getHighestSeverity(pendingDetails.value))
const severityLabel = computed(
	() => severity.value.charAt(0).toUpperCase() + severity.value.slice(1),
)
const severityStyle = computed(() => ({
	'--_bg-color': severityPresentation[severity.value].bgColor,
	'--_color': severityPresentation[severity.value].color,
}))

function viewFlags() {
	if (canSelect.value) {
		emit('viewFlags', props.file)
	}
}
</script>

<template>
	<div
		class="relative flex h-[72px] w-full min-w-0 flex-col justify-center gap-1.5 rounded-lg px-3 py-2.5 text-left"
	>
		<button
			type="button"
			:disabled="!canSelect"
			:aria-current="selected ? 'true' : undefined"
			:aria-label="file.file_name"
			class="absolute inset-0 rounded-lg border-0 transition-[background-color,filter] focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand disabled:cursor-default"
			:class="[
				selected ? '!bg-surface-4' : 'bg-transparent',
				canSelect ? 'cursor-pointer hover:bg-surface-3 hover:brightness-125' : 'cursor-default',
			]"
			@click="viewFlags"
		/>
		<span
			class="pointer-events-none relative flex w-full min-w-0 items-center justify-between gap-3"
		>
			<span class="truncate text-sm font-medium text-contrast" :title="file.file_name">{{
				file.file_name
			}}</span>
			<span class="flex shrink-0 items-center gap-1">
				<TagItem
					v-if="hasUnsafeTraces"
					class="!px-1.5 !py-0.5 !text-xs"
					style="--_bg-color: var(--color-red-bg); --_color: var(--color-red)"
				>
					<ShieldAlertIcon aria-hidden="true" />
					Unsafe
				</TagItem>
				<TagItem
					v-if="pendingDetails.length"
					class="!px-1.5 !py-0.5 !text-xs"
					style="--_bg-color: var(--color-orange-bg); --_color: var(--color-orange)"
				>
					Pending
				</TagItem>
				<TagItem
					v-if="pendingDetails.length"
					class="!px-1.5 !py-0.5 !text-xs"
					:style="severityStyle"
				>
					<component
						:is="severityPresentation[severity].icon"
						:style="{ color: severityPresentation[severity].color }"
						aria-hidden="true"
					/>
					{{ severityLabel }}
				</TagItem>
				<span
					v-else-if="!hasUnsafeTraces"
					class="text-xs"
					:class="!hasFlags && isManualReview ? 'text-blue' : 'text-secondary'"
					>{{ hasFlags ? 'Reviewed' : isManualReview ? 'Manual review' : 'No flags' }}</span
				>
			</span>
		</span>
		<span
			class="pointer-events-none relative flex w-full min-w-0 items-center justify-between gap-3 text-xs text-secondary"
		>
			<span class="flex min-w-0 items-center gap-1.5">
				<span class="shrink-0">{{ fileSizeLabel }}</span>
				<BulletDivider
					v-if="versionHref"
					class="!mx-0 !max-h-1 !min-h-1 !min-w-1 !max-w-1 !bg-secondary opacity-90"
					aria-hidden="true"
				/>
				<a
					v-if="versionHref"
					:href="versionHref"
					target="_blank"
					rel="noopener noreferrer"
					class="pointer-events-auto inline-flex shrink-0 items-center gap-1 text-secondary hover:underline"
					@click.stop
					@auxclick.stop
				>
					View version
					<ExternalIcon class="size-3" aria-hidden="true" />
				</a>
				<BulletDivider
					v-if="hiddenTraceCount"
					class="!mx-0 !max-h-1 !min-h-1 !min-w-1 !max-w-1 !bg-secondary opacity-90"
					aria-hidden="true"
				/>
				<span v-if="hiddenTraceCount" class="shrink-0">
					{{ hiddenTraceCount }} hidden {{ hiddenTraceCount === 1 ? 'trace' : 'traces' }}
				</span>
			</span>
			<span
				v-if="hasFlags"
				class="flex shrink-0 items-center gap-1.5 whitespace-nowrap tabular-nums"
			>
				<span>{{ globallyPassedCount }} globally passed</span>
				<BulletDivider
					class="!mx-0 !max-h-1 !min-h-1 !min-w-1 !max-w-1 !bg-secondary opacity-90"
					aria-hidden="true"
				/>
				<span>{{ reviewedCount }}/{{ reviewableDetails.length }} reviewed</span>
			</span>
		</span>
	</div>
</template>
