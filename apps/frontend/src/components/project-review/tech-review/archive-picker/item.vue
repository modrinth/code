<script setup lang="ts">
import { ExternalIcon, ShieldAlertIcon } from '@modrinth/assets'
import { BulletDivider, TagItem } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review/page-context'

import { formatBytes } from '../../format'
import { getHighestSeverity, getVersionPageHref } from '../helpers'
import { severityPresentation } from '../trace-presentation'
import type { FlattenedFileReport } from '../types'
import { injectTechReviewDecisions } from '../use-tech-review-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	selected?: boolean
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const { project } = injectProjectReviewPageContext()
const { getDetailDecision, isDetailGloballyPassed, isDetailActionable } =
	injectTechReviewDecisions()
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

const reviewableDetails = computed(() => details.value.filter(isDetailActionable))
const globallyPassedCount = computed(() => details.value.filter(isDetailGloballyPassed).length)
const pendingDetails = computed(() =>
	reviewableDetails.value.filter(
		(detail) => getDetailDecision(detail.id, detail.status) === 'pending',
	),
)
const reviewedCount = computed(() => reviewableDetails.value.length - pendingDetails.value.length)
const hasUnsafeTraces = computed(() =>
	details.value.some((detail) => getDetailDecision(detail.id, detail.status) === 'malware'),
)
const severity = computed(() =>
	getHighestSeverity(reviewableDetails.value.length ? reviewableDetails.value : details.value),
)
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
			<span class="flex min-w-0 items-center gap-1.5">
				<span class="truncate text-sm font-medium text-contrast" :title="file.file_name">{{
					file.file_name
				}}</span>
				<span
					v-if="hasUnsafeTraces"
					v-tooltip="'Unsafe'"
					role="img"
					aria-label="Unsafe"
					class="pointer-events-auto inline-flex shrink-0 items-center text-red"
					@click="viewFlags"
				>
					<ShieldAlertIcon class="size-4" aria-hidden="true" />
				</span>
				<TagItem
					v-if="pendingDetails.length"
					class="shrink-0 !px-1.5 !py-0.5 !text-xs"
					style="--_bg-color: var(--color-orange-bg); --_color: var(--color-orange)"
				>
					Pending
				</TagItem>
			</span>
			<span class="flex shrink-0 items-center gap-1">
				<TagItem v-if="hasFlags" class="!px-1.5 !py-0.5 !text-xs">
					<template v-if="reviewableDetails.length">
						{{ reviewedCount }}/{{ reviewableDetails.length }} done
					</template>
					<template v-else>No actionable traces</template>
				</TagItem>
				<TagItem
					v-if="hasFlags"
					v-tooltip="`Highest trace severity: ${severityLabel}`"
					role="img"
					:aria-label="`Highest trace severity: ${severityLabel}`"
					class="pointer-events-auto !px-0.5 !py-0.5 !text-xs"
					:style="severityStyle"
					@click="viewFlags"
				>
					<component
						:is="severityPresentation[severity].icon"
						:style="{ color: severityPresentation[severity].color }"
						aria-hidden="true"
					/>
				</TagItem>
				<TagItem
					v-else
					class="!px-1.5 !py-0.5 !text-xs"
					:style="{
						'--_color': isManualReview ? 'var(--color-blue)' : 'var(--color-secondary)',
					}"
				>
					{{ isManualReview ? 'Manual review' : 'No flags' }}
				</TagItem>
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
			</span>
			<span
				v-if="hiddenTraceCount || globallyPassedCount"
				class="flex shrink-0 items-center gap-1.5 whitespace-nowrap tabular-nums"
			>
				<span v-if="hiddenTraceCount">{{ hiddenTraceCount }} hidden</span>
				<BulletDivider
					v-if="hiddenTraceCount && globallyPassedCount"
					class="!mx-0 !max-h-1 !min-h-1 !min-w-1 !max-w-1 !bg-secondary opacity-90"
					aria-hidden="true"
				/>
				<span v-if="globallyPassedCount">{{ globallyPassedCount }} globally passed</span>
			</span>
		</span>
	</div>
</template>
