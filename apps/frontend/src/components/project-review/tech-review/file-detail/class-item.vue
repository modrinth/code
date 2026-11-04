<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	ClipboardCopyIcon,
	CopyIcon,
	LoaderCircleIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	Accordion,
	codeClipboardHtml,
	formatCodeBlock,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import { highlightCodeLines } from '@modrinth/utils'
import {
	StorageSerializers,
	useDebounceFn,
	useEventListener,
	useLocalStorage,
	useResizeObserver,
} from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { slicerFileName } from '~/providers/project-review/review-slicer'

import { injectReviewPanelVisible } from '../../layout/context'
import FileActions from '../files/actions.vue'
import HeightDivider from '../height-divider.vue'
import { severityOrder, truncateMiddle } from '../helpers'
import PanelDivider from '../panel-divider.vue'
import { matchesSourceLink, type SourceLinkTarget, type SourceView } from '../source-link'
import TraceData from '../traces/data.vue'
import TraceName from '../traces/name.vue'
import type { ClassGroup, FlattenedFileReport, TraceVerdictEvent } from '../types'
import { useFastScroll } from '../use-fast-scroll'
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
import StringViewer from './string-viewer/index.vue'

const props = defineProps<{
	file: FlattenedFileReport
	classItem: ClassGroup
	expanded: boolean
	splitView?: boolean
	focusedTraceId?: string | null
	sourceLink?: SourceLinkTarget
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const tracesWidth = defineModel<number>('tracesWidth', { default: 360 })
const classPane = ref<HTMLElement>()
const tracesPane = ref<HTMLElement>()
const panelVisible = injectReviewPanelVisible()

watch(
	[tracesPane, panelVisible, () => props.splitView],
	([pane, visible, splitView], _, onCleanup) => {
		if (!pane || !visible || !splitView || pane.ownerDocument !== document) return
		const body = pane.ownerDocument.body
		const updateOffset = () => {
			if (pane.ownerDocument !== document || !pane.getClientRects().length) {
				body.style.removeProperty('--project-review-traces-panel-offset')
				return
			}
			const viewport = pane.ownerDocument.defaultView
			if (!viewport) return
			body.style.setProperty(
				'--project-review-traces-panel-offset',
				`${Math.max(0, viewport.innerWidth - pane.getBoundingClientRect().left)}px`,
			)
		}
		const { stop } = useResizeObserver([classPane, tracesPane], updateOffset)
		const stopResize = useEventListener(pane.ownerDocument.defaultView, 'resize', updateOffset)
		updateOffset()
		onCleanup(() => {
			stop()
			stopResize()
			body.style.removeProperty('--project-review-traces-panel-offset')
		})
	},
	{ flush: 'post' },
)
const tracesHeight = useLocalStorage<number | null>('moderation-tech-review-traces-height', null, {
	serializer: StorageSerializers.number,
})

const emit = defineEmits<{
	toggle: []
	verdict: [event: TraceVerdictEvent]
	sourceView: [view: SourceView]
	sourceLinkHandled: [target: SourceLinkTarget]
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

const showHiddenTraces = ref(false)
const traceSections = computed(() => [
	{
		id: 'traces',
		label: 'Traces',
		flags: props.classItem.flags
			.filter((flag) => flag.detail.severity !== 'hidden')
			.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity]),
	},
	{
		id: 'hidden',
		label: 'Hidden traces',
		flags: props.classItem.flags.filter((flag) => flag.detail.severity === 'hidden'),
	},
])

const visibleFlags = computed(() =>
	props.classItem.flags
		.filter((flag) => showHiddenTraces.value || flag.detail.severity !== 'hidden')
		.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity]),
)

watch(
	() => props.focusedTraceId,
	(id) => {
		const flag = props.classItem.flags.find((flag) => String(flag.detail.id) === id)
		if (flag?.detail.severity === 'hidden') showHiddenTraces.value = true
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
const stringViewer = ref<InstanceType<typeof StringViewer>>()
const scrollSource = useFastScroll()
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

defineExpose({
	navigateHighlight,
	canNavigateHighlight: () => !!navigableMatches.value.length && !!sourcePane.value,
})

const sourceWithMatches = computed(() => {
	const document = sourcePane.value?.ownerDocument
	if (!document || !activeMatches.value.size) return highlightedSource.value
	return highlightedSource.value.map((line, index) => {
		const matches = activeMatches.value.get(index)
		return matches ? markSourceLine(line, matches, document, globalMatches.value.get(index)) : line
	})
})
const highlightContext = computed(() =>
	JSON.stringify([
		props.file.id,
		props.classItem.key,
		decompiledSource.value,
		navigableMatches.value,
	]),
)
watch(
	[highlightContext, sourceWithMatches],
	([context], [previousContext]) => {
		if (context !== previousContext) {
			currentHighlight = -1
			clearSelectedHighlight()
			return
		}
		const selected = navigableMatches.value[currentHighlight]
		if (selected) markSelectedSourceMatch(selected.line, selected.range)
	},
	{ flush: 'post' },
)

const sourceContent = ref<HTMLElement>()
const selectedLines = ref<{ start: number; end: number }>()
const lineSelection = ref<{ anchor: number; pointerId: number }>()
let selectionAnchor: number | undefined
let releaseSourceSelection: (() => void) | undefined

function stopSourceSelection() {
	releaseSourceSelection?.()
}

function containSourceSelection(event: PointerEvent) {
	if (event.button !== 0) return
	stopSourceSelection()
	const content = event.currentTarget as HTMLElement
	const document = content.ownerDocument
	const view = document.defaultView
	const style = document.body.style
	const properties = ['user-select', '-webkit-user-select'].map((property) => ({
		property,
		value: style.getPropertyValue(property),
		priority: style.getPropertyPriority(property),
	}))
	for (const { property } of properties) style.setProperty(property, 'none')

	function onPointerEnd(event: PointerEvent) {
		if (event.pointerId === pointerId) stopSourceSelection()
	}

	const pointerId = event.pointerId
	document.addEventListener('pointerup', onPointerEnd, true)
	document.addEventListener('pointercancel', onPointerEnd, true)
	view?.addEventListener('blur', stopSourceSelection)
	view?.addEventListener('beforeunload', stopSourceSelection)
	releaseSourceSelection = () => {
		for (const { property, value, priority } of properties) {
			if (value) style.setProperty(property, value, priority)
			else style.removeProperty(property)
		}
		document.removeEventListener('pointerup', onPointerEnd, true)
		document.removeEventListener('pointercancel', onPointerEnd, true)
		view?.removeEventListener('blur', stopSourceSelection)
		view?.removeEventListener('beforeunload', stopSourceSelection)
		lineSelection.value = undefined
		releaseSourceSelection = undefined
	}
}

onBeforeUnmount(stopSourceSelection)

watch(
	[() => props.file.id, () => props.classItem, selectedLines],
	() =>
		emit('sourceView', {
			reportId: props.file.id,
			path: props.classItem.filePath,
			jar: props.classItem.jar ?? '',
			lines: selectedLines.value
				? {
						start: selectedLines.value.start + 1,
						end: selectedLines.value.end + 1,
					}
				: undefined,
		}),
	{ immediate: true, flush: 'post' },
)

function isLineSelected(line: number) {
	return (
		!!selectedLines.value && line >= selectedLines.value.start && line <= selectedLines.value.end
	)
}

useEventListener(
	computed(() => sourceContent.value?.ownerDocument),
	'selectionchange',
	() => {
		if (!selectedLines.value) return
		const content = sourceContent.value
		const selection = content?.ownerDocument.getSelection()
		const range = selection?.rangeCount ? selection.getRangeAt(0) : undefined
		const first = content?.querySelector(`[data-source-line="${selectedLines.value.start}"] pre`)
		const last = content?.querySelector(`[data-source-line="${selectedLines.value.end}"] pre`)
		if (
			!range ||
			selection?.isCollapsed ||
			range.startContainer !== first ||
			range.endContainer !== last ||
			range.startOffset !== 0 ||
			range.endOffset !== last?.childNodes.length
		) {
			selectedLines.value = undefined
		}
	},
)

watch(sourceContent, () => {
	stopSourceSelection()
	selectedLines.value = undefined
	lineSelection.value = undefined
	selectionAnchor = undefined
})

function selectSourceLines(anchor: number, end: number) {
	const content = sourceContent.value
	const first = content?.querySelector(`[data-source-line="${Math.min(anchor, end)}"] pre`)
	const last = content?.querySelector(`[data-source-line="${Math.max(anchor, end)}"] pre`)
	if (!content || !first || !last) return
	selectionAnchor = anchor
	const range = content.ownerDocument.createRange()
	range.setStart(first, 0)
	range.setEnd(last, last.childNodes.length)
	const selection = content.ownerDocument.getSelection()
	selection?.removeAllRanges()
	selection?.addRange(range)
	if (
		selectedLines.value?.start !== Math.min(anchor, end) ||
		selectedLines.value?.end !== Math.max(anchor, end)
	) {
		selectedLines.value = {
			start: Math.min(anchor, end),
			end: Math.max(anchor, end),
		}
	}
}

function selectGutterLine(line: number, extend = false) {
	const anchor = extend && selectedLines.value ? (selectionAnchor ?? line) : line
	selectSourceLines(anchor, line)
	return anchor
}

let appliedSourceLink: SourceLinkTarget | undefined
watch(
	[() => props.sourceLink, sourceWithMatches, sourceContent, () => props.expanded, isLoadingSource],
	async ([target]) => {
		if (
			!target ||
			appliedSourceLink === target ||
			!props.expanded ||
			isLoadingSource.value ||
			!isUsableDecompiledSource(decompiledSource.value) ||
			!sourceContent.value ||
			target.reportId !== props.file.id ||
			!matchesSourceLink(target, props.classItem.filePath, props.classItem.jar)
		)
			return
		await nextTick()
		if (props.sourceLink !== target || appliedSourceLink === target || !sourceContent.value) return
		appliedSourceLink = target
		if (target.lines && target.lines.end <= sourceWithMatches.value.length) {
			selectSourceLines(target.lines.start - 1, target.lines.end - 1)
			sourceContent.value
				.querySelector(`[data-source-line="${target.lines.start - 1}"]`)
				?.scrollIntoView({ block: 'center' })
		}
		emit('sourceLinkHandled', target)
	},
	{ immediate: true, flush: 'post' },
)

function startLineSelection(event: PointerEvent, line: number) {
	if (event.button !== 0) return
	event.preventDefault()
	const gutter = event.currentTarget as HTMLElement
	gutter.setPointerCapture(event.pointerId)
	const anchor = selectGutterLine(line, event.shiftKey)
	lineSelection.value = { anchor, pointerId: event.pointerId }
}

function dragLineSelection(event: PointerEvent) {
	if (!lineSelection.value || lineSelection.value.pointerId !== event.pointerId) return
	const rows = sourceContent.value?.querySelectorAll<HTMLElement>('[data-source-line]')
	if (!rows?.length) return
	let start = 0
	let end = rows.length - 1
	while (start < end) {
		const middle = Math.floor((start + end) / 2)
		if (event.clientY >= rows[middle].getBoundingClientRect().bottom) start = middle + 1
		else end = middle
	}
	selectSourceLines(lineSelection.value.anchor, Number(rows[start].dataset.sourceLine))
}

function endLineSelection(event: PointerEvent) {
	if (lineSelection.value?.pointerId === event.pointerId) lineSelection.value = undefined
}

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
	const nextPositions = [...positions].map(([position, matches]) => ({
		position,
		...matches,
	}))
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
watch(sourceWithMatches, () => nextTick(updateScrollbarMatches), {
	flush: 'post',
})

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
	pane.scrollTo({
		top: position * pane.scrollHeight - pane.clientHeight / 2,
		behavior: 'instant',
	})
}

function scrollSourceTrack(event: WheelEvent) {
	const pane = sourcePane.value
	if (!pane) return
	const unit = event.deltaMode === 1 ? 20 : event.deltaMode === 2 ? pane.clientHeight : 1
	pane.scrollBy({
		top: event.deltaY * unit,
		left: event.deltaX * unit,
		behavior: 'instant',
	})
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

function selectClickedMatch(event: MouseEvent) {
	const pane = sourcePane.value
	if (!pane || pane.ownerDocument.getSelection()?.isCollapsed === false) return
	const mark = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-source-match]')
	const row = mark?.closest<HTMLElement>('[data-source-line]')
	if (!mark || !row || !pane.contains(mark)) return
	const line = Number(row.dataset.sourceLine)
	const start = Number(mark.dataset.sourceStart)
	const index = navigableMatches.value.findIndex(
		(match) => match.line === line && match.range.start <= start && match.range.end > start,
	)
	if (index < 0) return
	currentHighlight = index
	const match = navigableMatches.value[index]
	selectSourceMatch(match.line, match.range)
}

function markSelectedSourceMatch(line: number, range: SourceMatch) {
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
	return match
}

function selectSourceMatch(line: number, range: SourceMatch) {
	const pane = sourcePane.value
	const match = markSelectedSourceMatch(line, range)
	if (!pane || !match) return
	const bounds = pane.getBoundingClientRect()
	const target = match.getBoundingClientRect()
	const viewportLeft = bounds.left + pane.clientLeft
	const viewportRight = viewportLeft + pane.clientWidth
	let left = pane.scrollLeft
	if (
		target.left + pane.scrollLeft >= viewportLeft &&
		target.right + pane.scrollLeft <= viewportRight
	) {
		left = 0
	} else if (target.left < viewportLeft) {
		left += target.left - viewportLeft - 8
	} else if (target.right > viewportRight) {
		left += Math.max(0, Math.min(target.right - viewportRight + 8, target.left - viewportLeft - 8))
	}
	const viewportTop = bounds.top + pane.clientTop
	const viewportBottom = viewportTop + pane.clientHeight
	const top =
		target.top >= viewportTop && target.bottom <= viewportBottom
			? pane.scrollTop
			: pane.scrollTop + target.top - viewportTop - (pane.clientHeight - target.height) / 2
	scrollSource(pane, top, left)
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
	<div
		ref="classPane"
		:class="
			splitView
				? 'archive-class-panes grid min-h-0 min-w-0 gap-y-1'
				: 'flex min-h-0 min-w-0 flex-col gap-1'
		"
		:style="splitView ? { '--traces-width': `${tracesWidth}px` } : undefined"
	>
		<div
			class="flex min-w-0 shrink-0 items-center justify-between gap-2"
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

		<div v-if="expanded" class="contents">
			<aside
				ref="tracesPane"
				:class="
					splitView
						? 'col-start-3 row-span-2 row-start-1 flex min-h-0 min-w-0 flex-col pb-1 pl-1'
						: 'contents'
				"
				:aria-label="splitView ? 'Traces' : undefined"
			>
				<div
					:class="splitView ? 'flex min-h-0 flex-col overflow-hidden' : 'contents'"
					:style="
						splitView && $slots.conversation
							? {
									height: tracesHeight === null ? 'calc(50% - 0.5rem)' : `${tracesHeight}px`,
									maxHeight: 'calc(50% - 0.5rem)',
									flexShrink: 0,
								}
							: { flex: 1 }
					"
				>
					<div
						data-trace-list
						:class="splitView ? 'min-h-0 flex-1 overflow-y-auto px-2' : 'contents'"
					>
						<component
							v-for="(section, sectionIndex) in traceSections.filter(
								(section) => section.flags.length,
							)"
							:key="section.id"
							:is="section.id === 'hidden' ? Accordion : 'section'"
							:open-by-default="section.id === 'hidden' ? showHiddenTraces : undefined"
							:button-class="
								section.id === 'hidden'
									? `custom-focus-indicator sticky top-0 z-10 flex w-full cursor-pointer items-center gap-1 border-0 bg-bg px-0 py-2 text-left text-sm font-semibold text-primary ${sectionIndex > 0 ? 'pt-5' : ''}`
									: undefined
							"
							@on-open="showHiddenTraces = true"
							@on-close="showHiddenTraces = false"
						>
							<template #button="{ open }">
								<span>
									{{ section.label }} ({{ getMarkedFlagsCount(section.flags) }}/{{
										section.flags.length
									}})
								</span>
								<ChevronDownIcon
									class="size-4 shrink-0 transition-transform"
									:class="{ '-rotate-90': !open }"
									aria-hidden="true"
								/>
							</template>
							<h4
								v-if="section.id !== 'hidden'"
								class="sticky top-0 z-10 m-0 bg-bg py-2 text-sm font-semibold text-primary"
							>
								{{ section.label }} ({{ getMarkedFlagsCount(section.flags) }}/{{
									section.flags.length
								}})
							</h4>
							<div
								v-for="(flag, flagIndex) in section.flags"
								:id="`tech-review-detail-${flag.detail.id}`"
								:key="`${flag.issueId}-${flag.detail.id}`"
								class="flex shrink-0 flex-col gap-0.5 border-0 border-solid border-surface-3"
								:class="{
									'border-b pb-4':
										flagIndex < section.flags.length - 1 ||
										(section.id === 'traces' &&
											traceSections.some((item) => item.id === 'hidden' && item.flags.length > 0)),
									'focused-trace -mx-2 p-2': String(flag.detail.id) === focusedTraceId,
									'py-1.5 first:pt-0': String(flag.detail.id) !== focusedTraceId,
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
											size="sm"
											:label="
												copiedCelDetails.has(flag.detail.id) ? 'Copied!' : 'Copy CEL to clipboard'
											"
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
											<ClipboardCopyIcon
												v-else
												class="!size-4 !min-h-4 !min-w-4"
												aria-hidden="true"
											/>
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
						</component>
					</div>
				</div>
				<HeightDivider v-if="$slots.conversation" v-model="tracesHeight" />
				<div v-if="$slots.conversation" class="flex min-h-0 flex-1 flex-col gap-2.5">
					<slot name="conversation" />
				</div>
			</aside>
			<PanelDivider
				v-if="splitView"
				v-model="tracesWidth"
				class="col-start-2 row-span-2 row-start-1"
				label="Traces width"
				width-property="--traces-width"
				side="right"
				:min="220"
				:max="720"
				:reserve="200"
				:default-width="360"
			/>
			<div
				:class="
					splitView
						? 'source-scroll-area relative col-start-1 row-start-2 min-h-0 min-w-0'
						: 'contents'
				"
			>
				<section
					ref="sourcePane"
					:class="[
						splitView
							? 'review-scrollbars h-full min-h-0 min-w-0 overflow-auto pr-1 [container-type:inline-size]'
							: 'contents',
						{ 'overflow-x-hidden': splitView && wrapCode },
					]"
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
						<div
							class="sticky left-0 top-0 z-10 h-0"
							:class="splitView && !wrapCode ? 'w-[100cqw]' : 'w-full'"
						>
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

						<div
							data-source-code
							:data-report-id="file.id"
							:data-file-path="classItem.filePath"
							:data-jar="classItem.jar ?? ''"
							class="select-text py-3"
							:class="[
								wrapCode ? 'w-full min-w-0' : 'w-max',
								{ 'source-line-selection': selectedLines },
							]"
							@click="selectClickedMatch"
							@contextmenu="stringViewer?.onContextMenu($event)"
							@pointerdown.capture="containSourceSelection"
							@pointermove="dragLineSelection"
							@pointerup="endLineSelection"
							@pointercancel="endLineSelection"
							@lostpointercapture="endLineSelection"
						>
							<div
								v-for="(line, n) in sourceWithMatches"
								:key="n"
								class="flex font-mono text-[13px] leading-[1.6]"
								:data-source-line="n"
								:class="{
									'source-match-row': activeMatches.has(n),
									'trace-source-match-row': traceMatches.has(n),
									'source-selected-row': isLineSelected(n),
								}"
							>
								<button
									type="button"
									class="source-line-number shrink-0 touch-none select-none border-0 border-r border-solid border-surface-5 bg-transparent px-4 py-0 text-right font-[inherit] text-primary"
									:class="lineSelection ? 'cursor-text' : 'cursor-default'"
									style="min-width: 3.5rem"
									:aria-label="`Select line ${n + 1}`"
									:aria-pressed="isLineSelected(n)"
									title="Drag to select lines; Shift-click to extend. Enter or Space selects a line."
									@pointerdown="startLineSelection($event, n)"
									@click="$event.detail === 0 && selectGutterLine(n, $event.shiftKey)"
								>
									{{ n + 1 }}
								</button>
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
		<StringViewer ref="stringViewer" />
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

.focused-trace {
	background-color: color-mix(in srgb, var(--color-blue) 15%, transparent);
}

.trace-source-match-row {
	background: color-mix(in srgb, var(--color-orange-bg) 50%, transparent);
}

.source-line-number:hover {
	background: color-mix(in srgb, var(--color-blue) 15%, transparent);
	color: var(--color-blue);
	box-shadow: inset 3px 0 var(--color-blue);
}

.source-selected-row {
	background: color-mix(in srgb, var(--color-blue) 15%, transparent);
}

.source-selected-row .source-line-number {
	background: color-mix(in srgb, var(--color-blue) 20%, transparent);
	color: var(--color-blue);
	font-weight: 700;
	box-shadow: inset 3px 0 var(--color-blue);
}

.source-line-selection :deep(::selection) {
	background: transparent;
	color: inherit;
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
	grid-template-rows: auto minmax(0, 1fr);
	grid-template-columns: minmax(0, 1fr) 8px min(var(--traces-width), max(220px, calc(100% - 208px)));
}

pre {
	all: unset;
	display: inline;
	white-space: pre;
}

.source-line-wrap pre {
	display: block;
	white-space: break-spaces;
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
