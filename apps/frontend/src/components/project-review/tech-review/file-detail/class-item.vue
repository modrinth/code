<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	ClipboardCopyIcon,
	CopyIcon,
	EyeIcon,
	EyeOffIcon,
	LoaderCircleIcon,
} from '@modrinth/assets'
import { Button, IconButton, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { highlightCodeLines } from '@modrinth/utils'
import { computed, reactive, ref, watch } from 'vue'

import { slicerFileName } from '~/providers/project-review/review-slicer'

import FileActions from '../files/actions.vue'
import { severityOrder, truncateMiddle } from '../helpers'
import PanelDivider from '../panel-divider.vue'
import TraceData from '../traces/data.vue'
import TraceName from '../traces/name.vue'
import type { ClassGroup, FlattenedFileReport, TraceVerdictEvent } from '../types'
import VerdictButtons from '../verdict/buttons.vue'
import { injectTechReviewDecisions } from '../verdict/use-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	classItem: ClassGroup
	expanded: boolean
	splitView?: boolean
	focusedDetailId?: string | null
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const tracesWidth = defineModel<number>('tracesWidth', { default: 360 })

const emit = defineEmits<{
	toggle: []
	verdict: [event: TraceVerdictEvent]
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { isPreReviewed, getMarkedFlagsCount } = injectTechReviewDecisions()

const showHiddenTraces = defineModel<boolean>('showHiddenTraces', { default: false })
const hiddenTraceCount = computed(
	() => props.classItem.flags.filter((flag) => flag.detail.severity === 'hidden').length,
)
const visibleFlags = computed(() =>
	props.classItem.flags
		.filter((flag) => showHiddenTraces.value || flag.detail.severity !== 'hidden')
		.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity]),
)

watch(
	() => props.focusedDetailId,
	(id) => {
		if (
			props.classItem.flags.some(
				(flag) => String(flag.detail.id) === id && flag.detail.severity === 'hidden',
			)
		) {
			showHiddenTraces.value = true
		}
	},
	{ immediate: true },
)

const showCopyFeedback = ref(false)
const copyingCelDetails = reactive<Set<string>>(new Set())
const copiedCelDetails = reactive<Set<string>>(new Set())

const truncatedPath = computed(() => truncateMiddle(props.classItem.filePath))
const pathTooltip = computed(() =>
	truncatedPath.value !== props.classItem.filePath ? props.classItem.filePath : undefined,
)

const allFlagsMarked = computed(
	() => getMarkedFlagsCount(props.classItem.flags) === props.classItem.flags.length,
)

const isLoadingSource = computed(() =>
	props.classItem.flags.some((flag) => props.loadingIssues.has(flag.issueId)),
)

const decompiledSource = computed(() => {
	for (const flag of props.classItem.flags) {
		const source = props.decompiledSources.get(flag.detail.id)
		if (source) return source
	}
	return undefined
})

const highlightedSource = computed(() => {
	const source = decompiledSource.value
	if (!source) return []
	return highlightCodeLines(source, 'java')
})

function emitVerdict(event: TraceVerdictEvent) {
	emit('verdict', event)
}

async function copyToClipboard() {
	const source = decompiledSource.value
	if (!source) return

	try {
		await navigator.clipboard.writeText(source)
		showCopyFeedback.value = true
		setTimeout(() => {
			showCopyFeedback.value = false
		}, 2000)
	} catch (error) {
		console.error('Failed to copy code:', error)
	}
}

async function copyDetailCelInput(detailId: string) {
	if (copyingCelDetails.has(detailId)) return

	copyingCelDetails.add(detailId)
	try {
		const input = await client.labrinth.tech_review_internal.getDetailRuleInput(detailId)
		await navigator.clipboard.writeText(JSON.stringify(input, null, 2))
		copiedCelDetails.add(detailId)
		setTimeout(() => {
			copiedCelDetails.delete(detailId)
		}, 2000)
	} catch (error) {
		console.error('Failed to copy CEL input:', error)
		addNotification({
			type: 'error',
			title: 'Failed to copy CEL input',
			text: 'An error occurred while loading the trace rule input.',
		})
	} finally {
		copyingCelDetails.delete(detailId)
	}
}

function trimSlashes(path: string): string {
	if (path.startsWith('/')) {
		path = path.slice(1)
	}
	if (path.endsWith('/')) {
		path = path.slice(0, -1)
	}
	return path
}

/**
 * Resolves the slicer workspace entry for a class group, removing trailing and leading slashes and
 * assuming `.class` when the path has no file extension.
 */
function slicerEntry(url: string, group: ClassGroup) {
	const jarPath = trimSlashes(group.jar ? group.jar.replace('#', '/') : slicerFileName(url))
	const filePath = trimSlashes(group.filePath)
	const hasFileExtension = (filePath.split('/').pop() || '').includes('.')
	return `${jarPath}/${hasFileExtension ? filePath : `${filePath}.class`}`
}
</script>

<template>
	<div class="flex min-h-0 min-w-0 flex-col gap-1">
		<div
			class="flex shrink-0 items-center justify-between gap-2"
			@click="!splitView && emit('toggle')"
		>
			<div class="flex min-w-0 grow items-center gap-2 pl-4">
				<IconButton
					v-if="!splitView"
					type="quiet"
					label="Toggle details"
					class="transition-transform"
					:class="{ 'rotate-180': expanded }"
				>
					<ChevronDownIcon class="h-5 w-5 text-contrast" />
				</IconButton>

				<span
					v-tooltip="pathTooltip"
					class="min-w-0 truncate font-mono text-sm"
					:class="{ 'opacity-50': allFlagsMarked }"
				>
					{{ truncatedPath }}
				</span>

				<div class="ml-auto flex items-center gap-2">
					<Transition name="fade">
						<div v-if="isLoadingSource" class="text-secondary">
							<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
								<LoaderCircleIcon class="size-4 animate-spin" />
								Loading source...
							</span>
						</div>
					</Transition>
					<FileActions :file="file" :entry-path="slicerEntry(file.download_url, classItem)" />
				</div>
			</div>
		</div>

		<div
			v-if="expanded"
			:class="splitView ? 'archive-class-panes grid min-h-0 flex-1' : 'contents'"
			:style="splitView ? { '--traces-width': `${tracesWidth}px` } : undefined"
		>
			<aside
				:class="splitView ? 'order-3 flex min-h-0 min-w-0 flex-col overflow-auto pl-2' : 'contents'"
				:aria-label="splitView ? 'Traces' : undefined"
			>
				<div
					v-if="splitView || hiddenTraceCount > 0"
					class="sticky top-0 z-[1] flex shrink-0 items-center justify-between gap-2 bg-bg pt-1"
				>
					<h4 class="m-0 text-sm font-semibold text-contrast">
						Traces ({{ getMarkedFlagsCount(classItem.flags) }}/{{ classItem.flags.length }})
					</h4>
					<Button
						v-if="!splitView && hiddenTraceCount > 0"
						type="quiet"
						size="sm"
						:aria-pressed="showHiddenTraces"
						:aria-label="showHiddenTraces ? 'Hide hidden traces' : 'Show hidden traces'"
						@click="showHiddenTraces = !showHiddenTraces"
					>
						<EyeIcon v-if="showHiddenTraces" aria-hidden="true" />
						<EyeOffIcon v-else aria-hidden="true" />
						Hidden ({{ hiddenTraceCount }})
					</Button>
				</div>
				<p v-if="!visibleFlags.length" class="m-0 py-3 text-sm text-secondary">
					All traces are hidden.
				</p>
				<div
					v-for="flag in visibleFlags"
					:id="`tech-review-detail-${flag.detail.id}`"
					:key="`${flag.issueId}-${flag.detail.id}`"
					class="flex shrink-0 flex-col gap-3 border-0 border-b border-solid border-surface-3 py-3 first:pt-0 last:border-b-0"
					:class="{
						'bg-brand-highlight': focusedDetailId === String(flag.detail.id),
					}"
				>
					<div class="flex flex-wrap items-center justify-between gap-2">
						<div
							class="flex flex-wrap items-center gap-1.5"
							:class="{
								'opacity-50': isPreReviewed(flag.detail.id, flag.detail.status),
							}"
						>
							<TraceName :issue-type="flag.issueType" :severity="flag.detail.severity" />
						</div>

						<div class="flex flex-wrap items-center gap-1">
							<IconButton
								v-tooltip="
									copiedCelDetails.has(flag.detail.id) ? 'Copied!' : 'Copy CEL to clipboard'
								"
								type="quiet"
								:label="copiedCelDetails.has(flag.detail.id) ? 'Copied!' : 'Copy CEL to clipboard'"
								:disabled="copyingCelDetails.has(flag.detail.id)"
								@click="copyDetailCelInput(flag.detail.id)"
							>
								<LoaderCircleIcon
									v-if="copyingCelDetails.has(flag.detail.id)"
									class="animate-spin"
									aria-hidden="true"
								/>
								<CheckIcon v-else-if="copiedCelDetails.has(flag.detail.id)" aria-hidden="true" />
								<ClipboardCopyIcon v-else aria-hidden="true" />
							</IconButton>
							<VerdictButtons
								variant="trace"
								:detail="flag.detail"
								@global-safe="
									emitVerdict({
										detail: flag.detail,
										decision: 'safe',
										scope: 'global',
									})
								"
								@local-safe="
									emitVerdict({
										detail: flag.detail,
										decision: 'safe',
										scope: 'local',
									})
								"
								@local-unsafe="
									emitVerdict({
										detail: flag.detail,
										decision: 'malware',
										scope: 'local',
									})
								"
								@global-unsafe="
									emitVerdict({
										detail: flag.detail,
										decision: 'malware',
										scope: 'global',
									})
								"
							/>
						</div>
					</div>
					<TraceData
						v-if="flag.detail.data && Object.keys(flag.detail.data).length > 0"
						:value="flag.detail.data"
					/>
				</div>
			</aside>
			<PanelDivider
				v-if="splitView"
				v-model="tracesWidth"
				class="order-2"
				label="Traces width"
				width-property="--traces-width"
				side="right"
				:min="220"
				:max="720"
				:reserve="200"
				:default-width="360"
			/>
			<section
				:class="splitView ? 'order-1 min-h-0 min-w-0 overflow-auto pr-2' : 'contents'"
				:aria-label="splitView ? 'File content' : undefined"
			>
				<div v-if="highlightedSource.length > 0" class="relative min-h-full bg-surface-1">
					<div class="sticky left-0 top-0 z-10 h-0">
						<IconButton
							v-tooltip="'Copy code'"
							type="quiet"
							size="sm"
							label="Copy code"
							class="!absolute right-0 top-2 bg-surface-1"
							@click.stop="copyToClipboard"
						>
							<CopyIcon v-if="!showCopyFeedback" />
							<CheckIcon v-else />
						</IconButton>
					</div>

					<div class="w-max py-3">
						<div
							v-for="(line, n) in highlightedSource"
							:key="n"
							class="flex font-mono text-[13px] leading-[1.6]"
						>
							<div
								class="select-none border-0 border-r border-solid border-surface-5 px-4 py-0 text-right text-primary"
								style="min-width: 3.5rem"
							>
								{{ n + 1 }}
							</div>
							<div class="flex-1 px-4 py-0 text-primary">
								<pre v-html="line || ' '"></pre>
							</div>
						</div>
					</div>
				</div>
				<div v-else-if="!isLoadingSource" class="p-4">
					<p class="text-sm text-secondary">
						Source code not available or failed to decompile for this file.
					</p>
				</div>
				<p v-else class="m-0 p-4 text-sm text-secondary" role="status">Loading source...</p>
			</section>
		</div>
	</div>
</template>

<style scoped>
.archive-class-panes {
	grid-template-columns: minmax(0, 1fr) 8px min(var(--traces-width), max(220px, calc(100% - 208px)));
}

pre {
	all: unset;
	display: inline;
	white-space: pre;
}

.fade-enter-active {
	transition: opacity 0.3s ease-in;
	transition-delay: 0.2s;
}

.fade-leave-active {
	transition: opacity 0.15s ease-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
