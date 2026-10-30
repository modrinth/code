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
import { moderationSettings } from '@modrinth/moderation'
import {
	Button,
	codeClipboardHtml,
	formatCodeBlock,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import { highlightCodeLines } from '@modrinth/utils'
import { useDebounceFn, useResizeObserver } from '@vueuse/core'
import { computed, nextTick, reactive, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { slicerFileName } from '~/providers/project-review/review-slicer'

import FileActions from '../files/actions.vue'
import { severityOrder, truncateMiddle } from '../helpers'
import PanelDivider from '../panel-divider.vue'
import TraceData from '../traces/data.vue'
import TraceName from '../traces/name.vue'
import type { ClassGroup, FlattenedFileReport, TraceVerdictEvent } from '../types'
import VerdictButtons from '../verdict/buttons.vue'
import { injectTechReviewDecisions } from '../verdict/use-decisions'
import {
	collectSourceValues,
	findSourceMatches,
	markSourceLine,
	mergeSourceMatches,
	parseSourceHighlightWords,
	type SourceMatch,
} from './source-matches'
import { isUsableDecompiledSource } from './source-status'

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
const settings = useModerationSettings()
const wrapCode = computed(() => settings.value.get(moderationSettings.General.WrapTechReviewCode))
const globalHighlightWords = computed(() =>
	parseSourceHighlightWords(
		settings.value.get(moderationSettings.General.TechReviewHighlightWords),
	),
)

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
	const sources = props.classItem.flags.map((flag) => props.decompiledSources.get(flag.detail.id))
	return sources.find(isUsableDecompiledSource) ?? sources.find((source) => source?.trim())
})

const highlightedSource = computed(() => {
	const source = decompiledSource.value
	if (!source) return []
	return highlightCodeLines(source, 'java')
})

const sourcePane = ref<HTMLElement>()
const sourceMatches = computed(() => {
	const lines = decompiledSource.value?.split('\n') ?? []
	const cache = new Map<string, ReturnType<typeof findSourceMatches>>()
	return (value: string) => {
		if (!cache.has(value)) cache.set(value, findSourceMatches(lines, value))
		return cache.get(value)!
	}
})
function canHighlight(value: string) {
	return sourceMatches.value(value).size > 0
}
const traceMatches = computed(() =>
	mergeSourceMatches(
		visibleFlags.value
			.flatMap((flag) => collectSourceValues(flag.detail.data))
			.map((value) => sourceMatches.value(value)),
	),
)
const globalMatches = computed(() =>
	mergeSourceMatches(globalHighlightWords.value.map((value) => sourceMatches.value(value))),
)
const activeMatches = computed(() => mergeSourceMatches([traceMatches.value, globalMatches.value]))
const navigableMatches = computed(() =>
	[...activeMatches.value]
		.sort(([a], [b]) => a - b)
		.flatMap(([line, ranges]) => ranges.map((range) => ({ line, range }))),
)
let currentHighlight = -1

function clearSelectedHighlight() {
	for (const mark of sourcePane.value?.querySelectorAll('[data-selected-source-match]') ?? []) {
		mark.removeAttribute('data-selected-source-match')
	}
}

function navigateHighlight(direction: number) {
	const matches = navigableMatches.value
	if (!matches.length || !sourcePane.value) return false
	if (currentHighlight < 0) currentHighlight = direction > 0 ? 0 : matches.length - 1
	else currentHighlight = (currentHighlight + direction + matches.length) % matches.length
	const { line, range } = matches[currentHighlight]
	selectSourceMatch(line, range)
	return true
}

defineExpose({ navigateHighlight })

const sourceWithMatches = computed(() => {
	const document = sourcePane.value?.ownerDocument
	if (!document || !activeMatches.value.size) return highlightedSource.value
	return highlightedSource.value.map((line, index) => {
		const matches = activeMatches.value.get(index)
		return matches ? markSourceLine(line, matches, document, globalMatches.value.get(index)) : line
	})
})
watch(
	sourceWithMatches,
	() => {
		currentHighlight = -1
		clearSelectedHighlight()
	},
	{ flush: 'post' },
)

const sourceContent = ref<HTMLElement>()
const scrollbarMatches = ref<{ position: number; global: boolean; trace: boolean }[]>([])
const scrollbarTrackWidth = ref(0)

function updateScrollbarMatches() {
	const pane = sourcePane.value
	const content = sourceContent.value
	if (pane) scrollbarTrackWidth.value = pane.offsetWidth - pane.clientWidth
	const scrollHeight = pane?.scrollHeight ?? 0
	if (!props.splitView || !pane || !content || scrollHeight <= pane.clientHeight) {
		scrollbarMatches.value = []
		return
	}
	const positions = new Map<number, { global: boolean; trace: boolean }>()
	for (const row of content.querySelectorAll<HTMLElement>('.source-match-row')) {
		const position = ((row.offsetTop + row.offsetHeight / 2) / scrollHeight) * 100
		const line = Number(row.dataset.sourceLine)
		const previous = positions.get(position)
		positions.set(position, {
			global: !!previous?.global || globalMatches.value.has(line),
			trace: !!previous?.trace || traceMatches.value.has(line),
		})
	}
	const nextPositions = [...positions].map(([position, matches]) => ({ position, ...matches }))
	if (
		nextPositions.length !== scrollbarMatches.value.length ||
		nextPositions.some(
			(match, index) =>
				match.position !== scrollbarMatches.value[index].position ||
				match.global !== scrollbarMatches.value[index].global ||
				match.trace !== scrollbarMatches.value[index].trace,
		)
	) {
		scrollbarMatches.value = nextPositions
	}
}

const updateScrollbarMatchesAfterResize = useDebounceFn(updateScrollbarMatches, 100)
useResizeObserver(sourcePane, updateScrollbarMatchesAfterResize)
useResizeObserver(sourceContent, updateScrollbarMatchesAfterResize)
watch(sourceWithMatches, () => nextTick(updateScrollbarMatches), { flush: 'post' })

function seekSource(event: PointerEvent) {
	const track = event.currentTarget as HTMLElement
	const pane = sourcePane.value
	if (!pane || (event.type === 'pointerdown' && event.button !== 0)) return
	if (event.type === 'pointerdown') {
		event.preventDefault()
		track.setPointerCapture(event.pointerId)
	} else if (!track.hasPointerCapture(event.pointerId)) return
	const bounds = track.getBoundingClientRect()
	const position = Math.max(0, Math.min(1, (event.clientY - bounds.top) / bounds.height))
	pane.scrollTo({ top: position * pane.scrollHeight - pane.clientHeight / 2, behavior: 'instant' })
}

function scrollSourceTrack(event: WheelEvent) {
	const pane = sourcePane.value
	if (!pane) return
	const unit = event.deltaMode === 1 ? 20 : event.deltaMode === 2 ? pane.clientHeight : 1
	pane.scrollBy({ top: event.deltaY * unit, left: event.deltaX * unit, behavior: 'instant' })
}

async function selectSourceValue(value: string) {
	await nextTick()
	const firstMatch = sourceMatches.value(value).entries().next().value
	if (!firstMatch) return
	const [line, ranges] = firstMatch
	currentHighlight = navigableMatches.value.findIndex(
		(match) =>
			match.line === line &&
			match.range.start <= ranges[0].start &&
			match.range.end > ranges[0].start,
	)
	selectSourceMatch(line, ranges[0])
}

function selectSourceMatch(line: number, range: SourceMatch) {
	const pane = sourcePane.value
	clearSelectedHighlight()
	const marks = [
		...(pane?.querySelectorAll<HTMLElement>(`[data-source-line="${line}"] [data-source-match]`) ??
			[]),
	].filter(
		(element) =>
			Number(element.dataset.sourceStart) < range.end &&
			Number(element.dataset.sourceEnd) > range.start,
	)
	const match = marks[0]
	if (!pane || !match) return
	for (const mark of marks) mark.setAttribute('data-selected-source-match', '')
	const bounds = pane.getBoundingClientRect()
	const target = match.getBoundingClientRect()
	const viewportLeft = bounds.left + pane.clientLeft
	const viewportRight = viewportLeft + pane.clientWidth
	let left = pane.scrollLeft
	if (target.left < viewportLeft) {
		left += target.left - viewportLeft - 8
	} else if (target.right > viewportRight) {
		left += Math.max(0, Math.min(target.right - viewportRight + 8, target.left - viewportLeft - 8))
	}
	const reducedMotion = pane.ownerDocument.defaultView?.matchMedia(
		'(prefers-reduced-motion: reduce)',
	).matches
	pane.scrollTo({
		top: pane.scrollTop + target.top - bounds.top - pane.clientHeight / 2,
		left,
		behavior: reducedMotion ? 'auto' : 'smooth',
	})
}

function emitVerdict(event: TraceVerdictEvent) {
	emit('verdict', event)
}

async function copyToClipboard() {
	const source = decompiledSource.value
	if (!source) return

	try {
		const document = sourcePane.value?.ownerDocument
		const view = document?.defaultView
		if (document && view?.ClipboardItem && view.navigator.clipboard.write) {
			await view.navigator.clipboard.write([
				new view.ClipboardItem({
					'text/plain': new Blob([source], { type: 'text/plain' }),
					'text/html': new Blob([codeClipboardHtml(document, source, 'java')], {
						type: 'text/html',
					}),
				}),
			])
		} else {
			await (view?.navigator ?? navigator).clipboard.writeText(formatCodeBlock(source, 'java'))
		}
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

				<div class="group/file-actions flex min-w-0 items-center gap-1">
					<span
						v-tooltip="pathTooltip"
						class="min-w-0 truncate font-mono text-sm font-semibold text-primary"
						:class="{ 'opacity-50': allFlagsMarked }"
					>
						{{ truncatedPath }}
					</span>
					<FileActions :file="file" :entry-path="slicerEntry(file.download_url, classItem)" />
				</div>

				<div class="ml-auto flex items-center gap-2">
					<Transition name="fade">
						<div v-if="isLoadingSource" class="text-secondary">
							<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
								<LoaderCircleIcon class="size-4 animate-spin" />
								Loading source...
							</span>
						</div>
					</Transition>
				</div>
			</div>
		</div>

		<div
			v-if="expanded"
			:class="splitView ? 'archive-class-panes grid min-h-0 flex-1' : 'contents'"
			:style="splitView ? { '--traces-width': `${tracesWidth}px` } : undefined"
		>
			<aside
				:class="
					splitView ? 'order-3 flex min-h-0 min-w-0 flex-col overflow-auto pb-1 pl-4' : 'contents'
				"
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
				<p v-if="!visibleFlags.length" class="m-0 py-1.5 text-sm text-secondary">
					All traces are hidden.
				</p>
				<div
					v-for="flag in visibleFlags"
					:id="`tech-review-detail-${flag.detail.id}`"
					:key="`${flag.issueId}-${flag.detail.id}`"
					class="flex shrink-0 flex-col gap-0.5 border-0 border-b border-solid border-surface-3 py-1.5 first:pt-0 last:border-b-0"
					:class="{ 'pl-1': splitView }"
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
								size="sm"
								:label="copiedCelDetails.has(flag.detail.id) ? 'Copied!' : 'Copy CEL to clipboard'"
								:disabled="copyingCelDetails.has(flag.detail.id)"
								@click="copyDetailCelInput(flag.detail.id)"
							>
								<LoaderCircleIcon
									v-if="copyingCelDetails.has(flag.detail.id)"
									class="!size-4 !min-h-4 !min-w-4 animate-spin"
									aria-hidden="true"
								/>
								<CheckIcon
									v-else-if="copiedCelDetails.has(flag.detail.id)"
									class="!size-4 !min-h-4 !min-w-4"
									aria-hidden="true"
								/>
								<ClipboardCopyIcon v-else class="!size-4 !min-h-4 !min-w-4" aria-hidden="true" />
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
						:can-highlight="canHighlight"
						@select="selectSourceValue"
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
			<div :class="splitView ? 'source-scroll-area relative order-1 min-h-0 min-w-0' : 'contents'">
				<section
					ref="sourcePane"
					:class="
						splitView
							? 'review-scrollbars h-full min-h-0 min-w-0 overflow-auto pr-2 [container-type:inline-size]'
							: 'contents'
					"
					:aria-label="splitView ? 'File content' : undefined"
				>
					<div
						v-if="highlightedSource.length > 0"
						ref="sourceContent"
						class="relative min-h-full bg-surface-1"
						:class="{
							'w-max min-w-full': splitView && !wrapCode,
							'source-line-wrap w-full min-w-0': wrapCode,
						}"
					>
						<div class="sticky left-0 top-0 z-10 h-0" :class="splitView ? 'w-[100cqw]' : 'w-full'">
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

						<div data-source-code class="py-3" :class="wrapCode ? 'w-full min-w-0' : 'w-max'">
							<div
								v-for="(line, n) in sourceWithMatches"
								:key="n"
								class="flex font-mono text-[13px] leading-[1.6]"
								:data-source-line="n"
								:class="{
									'source-match-row': activeMatches.has(n),
									'trace-source-match-row': traceMatches.has(n),
								}"
							>
								<div
									class="shrink-0 select-none border-0 border-r border-solid border-surface-5 px-4 py-0 text-right text-primary"
									style="min-width: 3.5rem"
								>
									{{ n + 1 }}
								</div>
								<div class="min-w-0 flex-1 px-4 py-0 text-primary">
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
				<div
					v-if="splitView"
					aria-hidden="true"
					class="absolute inset-y-0 right-0 z-20 touch-none overflow-hidden"
					:style="{ width: `max(0.75rem, ${scrollbarTrackWidth}px)` }"
					@pointerdown.stop="seekSource"
					@pointermove="seekSource"
					@wheel.prevent="scrollSourceTrack"
				>
					<span
						v-for="match in scrollbarMatches"
						:key="match.position"
						class="absolute right-0.5 h-0.5 opacity-50"
						:style="{
							top: `${match.position}%`,
							width: `${Math.max(2, (scrollbarTrackWidth || 6) - 4)}px`,
							background:
								match.global && match.trace
									? 'linear-gradient(to right, var(--color-blue) 50%, var(--color-orange) 50%)'
									: match.global
										? 'var(--color-blue)'
										: 'var(--color-orange)',
						}"
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<style scoped>
@property --traces-width {
	syntax: '<length>';
	inherits: false;
	initial-value: 360px;
}

@property --review-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.review-scrollbars {
	--review-scrollbar-color: transparent;
	--overlay-scrollbar-color: var(--review-scrollbar-color);
	transition: --review-scrollbar-color 1750ms ease;
}

.review-scrollbars:hover,
.source-scroll-area:hover .review-scrollbars {
	--review-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.review-scrollbars,
.review-scrollbars :deep(*) {
	scrollbar-color: var(--review-scrollbar-color) transparent;
}

.review-scrollbars::-webkit-scrollbar-thumb,
.review-scrollbars :deep(*::-webkit-scrollbar-thumb) {
	background: var(--review-scrollbar-color);
}

@media (hover: none) {
	.review-scrollbars {
		--review-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.review-scrollbars {
		transition: none;
	}
}

.trace-source-match-row {
	background: color-mix(in srgb, var(--color-orange-bg) 50%, transparent);
}

.source-scroll-area > .review-scrollbars {
	scrollbar-width: auto;
}

.source-scroll-area > .review-scrollbars::-webkit-scrollbar {
	width: calc(0.75rem + 4px);
}

:deep(mark[data-source-match]) {
	background: transparent;
	color: inherit;
	text-decoration: underline;
	text-decoration-style: wavy;
	text-decoration-color: var(--color-orange);
}

:deep(mark[data-global-source-match]) {
	text-decoration-color: var(--color-blue);
}

:deep(mark[data-selected-source-match]) {
	background: color-mix(in srgb, var(--color-orange) 50%, transparent);
}

:deep(mark[data-selected-source-match][data-global-source-match]) {
	background: color-mix(in srgb, var(--color-blue) 50%, transparent);
}

.archive-class-panes {
	grid-template-columns: minmax(0, 1fr) 8px min(var(--traces-width), max(220px, calc(100% - 208px)));
}

pre {
	all: unset;
	display: inline;
	white-space: pre;
}

.source-line-wrap pre {
	white-space: pre-wrap;
	overflow-wrap: anywhere;
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
