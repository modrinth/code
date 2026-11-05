<script setup lang="ts">
import { moderationSettings, type TechReviewFindAction } from '@modrinth/moderation'
import { highlightCodeLines } from '@modrinth/utils'
import { useDebounceFn, useEventListener, useResizeObserver } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import { useModerationKeybinds, useModerationSettings } from '~/composables/moderation'

import { injectReviewPanelVisible } from '../../layout/context'
import { reviewShortcutBlocked, useReviewEventListener } from '../../shortcuts'
import { matchesSourceLink, type SourceLinkTarget, type SourceView } from '../source-link'
import { isUsableDecompiledSource } from '../source-status'
import { useFastScroll } from '../use-fast-scroll'
import MatchToolbar from './match-toolbar.vue'
import {
	collectSourceValues,
	findSourceMatches,
	markSourceLine,
	mergeSourceMatches,
	parseSourceHighlightWords,
	type SourceMatch,
} from './source-matches'
import { searchSource, type SourceMatchMode, type SourceSearchOptions } from './source-search'
import StringViewer from './string-viewer/index.vue'

const props = defineProps<{
	reportId: string
	filePath: string
	jar: string | null
	source?: string
	loading: boolean
	expanded: boolean
	scrollable?: boolean
	sourceLink?: SourceLinkTarget
	highlightData: unknown[]
}>()

const emit = defineEmits<{
	sourceView: [view: SourceView]
	sourceLinkHandled: [target: SourceLinkTarget]
}>()

const settings = useModerationSettings()
const keybinds = useModerationKeybinds()
const panelVisible = injectReviewPanelVisible()
const route = useRoute()
const root = ref<HTMLElement>()
const wrapCode = computed(() => settings.value.get(moderationSettings.General.WrapTechReviewCode))
const globalHighlightWords = computed(() =>
	parseSourceHighlightWords(
		settings.value.get(moderationSettings.General.TechReviewHighlightWords),
	),
)

const decompiledSource = computed(() => props.source)
const isLoadingSource = computed(() => props.loading)

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
		props.highlightData
			.flatMap((data) => collectSourceValues(data))
			.map((value) => sourceMatches.value(value)),
	),
)
const globalMatches = computed(() =>
	mergeSourceMatches(globalHighlightWords.value.map((value) => sourceMatches.value(value))),
)
const matchMode = ref<SourceMatchMode>('closed')
const findQuery = useState('tech-review-source-find-query', () => '')
const findOptions = useState<SourceSearchOptions>('tech-review-source-find-options', () => ({
	matchCase: false,
	wholeWord: false,
	regex: false,
}))
const matchToolbar = ref<InstanceType<typeof MatchToolbar>>()
const isSearchMode = computed(() => matchMode.value === 'search')
const findResult = computed(() =>
	searchSource(decompiledSource.value?.split('\n') ?? [], findQuery.value, findOptions.value),
)
const searchMatches = computed(() =>
	isSearchMode.value ? findResult.value.matches : new Map<number, SourceMatch[]>(),
)
const highlightMatches = computed(() =>
	mergeSourceMatches([traceMatches.value, globalMatches.value]),
)
const activeMatches = computed(() =>
	mergeSourceMatches([highlightMatches.value, searchMatches.value]),
)
const navigableMatches = computed(() =>
	[...(isSearchMode.value ? searchMatches.value : highlightMatches.value)]
		.sort(([a], [b]) => a - b)
		.flatMap(([line, ranges]) => ranges.map((range) => ({ line, range }))),
)
const currentMatchIndex = ref(-1)

async function openSearch() {
	const pane = sourcePane.value
	const selection = pane?.ownerDocument.getSelection()
	if (selection && pane?.contains(selection.anchorNode) && pane.contains(selection.focusNode)) {
		const text = selection.toString()
		if (text && !text.includes('\n')) findQuery.value = text
	}
	matchMode.value = 'search'
	await nextTick()
	matchToolbar.value?.focus()
}

function closeMatchPanel() {
	matchMode.value = 'closed'
	currentMatchIndex.value = -1
	clearSelectedHighlight()
	sourcePane.value?.focus({ preventScroll: true })
}

watch(
	[
		() => props.reportId,
		() => props.filePath,
		() => props.jar,
		() => props.expanded,
		() => route.fullPath,
		panelVisible,
	],
	() => {
		matchMode.value = 'closed'
		currentMatchIndex.value = -1
		clearSelectedHighlight()
	},
)

async function openTraceMatches() {
	matchMode.value = 'traces'
	await nextTick()
	currentMatchIndex.value = -1
	navigateHighlight(1)
}

const findActions: Record<TechReviewFindAction, () => unknown> = {
	highlights: openTraceMatches,
	'select-all': selectAllSourceText,
	find: openSearch,
	close: closeMatchPanel,
	'previous-match': () => navigateHighlight(-1),
	'next-match': () => navigateHighlight(1),
	'match-case': () => (findOptions.value.matchCase = !findOptions.value.matchCase),
	'whole-word': () => (findOptions.value.wholeWord = !findOptions.value.wholeWord),
	regex: () => (findOptions.value.regex = !findOptions.value.regex),
}

useReviewEventListener(
	'keydown',
	(event) => {
		const pane = sourcePane.value
		const target = event.target as Node | null
		if (
			!pane ||
			!props.expanded ||
			!panelVisible.value ||
			!pane.getClientRects().length ||
			target?.ownerDocument !== pane.ownerDocument ||
			(!props.scrollable && !root.value?.contains(target)) ||
			event.isComposing ||
			event.defaultPrevented
		)
			return
		const inWidget = !!matchToolbar.value?.contains(target)
		if (!inWidget && reviewShortcutBlocked(event, true)) return
		const handled = keybinds.value.handle(event, {
			scope: 'tech-review-find',
			actions: {
				available: (action) => {
					if (action === 'select-all') return !!highlightedSource.value.length && !inWidget
					if (action === 'find') return true
					if (action === 'highlights') return !!highlightMatches.value.size
					if (action === 'close') return matchMode.value !== 'closed'
					if (!isSearchMode.value) return false
					if (action === 'previous-match' || action === 'next-match') {
						return (
							!!navigableMatches.value.length &&
							(!inWidget || !!matchToolbar.value?.isInput(target))
						)
					}
					return true
				},
				run: (action) => findActions[action](),
			},
		})
		if (handled) event.stopImmediatePropagation()
	},
	{ capture: true },
)

function clearSelectedHighlight() {
	for (const mark of sourcePane.value?.querySelectorAll('[data-selected-source-match]') ?? []) {
		mark.removeAttribute('data-selected-source-match')
	}
}

function navigateHighlight(direction: number) {
	const matches = navigableMatches.value
	if (!matches.length || !sourcePane.value) return false
	if (matchMode.value === 'closed') matchMode.value = 'traces'
	if (currentMatchIndex.value < 0) currentMatchIndex.value = direction > 0 ? 0 : matches.length - 1
	else
		currentMatchIndex.value =
			(currentMatchIndex.value + direction + matches.length) % matches.length
	const { line, range } = matches[currentMatchIndex.value]
	selectSourceMatch(line, range)
	return true
}

defineExpose({
	canHighlight,
	selectSourceValue,
	navigateHighlight,
	canNavigateHighlight: () => !!navigableMatches.value.length && !!sourcePane.value,
})

const sourceWithMatches = computed(() => {
	const document = sourcePane.value?.ownerDocument
	if (!document || !activeMatches.value.size) return highlightedSource.value
	return highlightedSource.value.map((line, index) => {
		const matches = activeMatches.value.get(index)
		return matches
			? markSourceLine(
					line,
					matches,
					document,
					globalMatches.value.get(index),
					searchMatches.value.get(index),
				)
			: line
	})
})
const highlightContext = computed(() =>
	JSON.stringify([
		props.reportId,
		[props.filePath, props.jar],
		decompiledSource.value,
		isSearchMode.value,
		findQuery.value,
		findOptions.value,
		navigableMatches.value,
	]),
)
watch(
	[highlightContext, sourceWithMatches],
	([context], [previousContext]) => {
		if (context !== previousContext) {
			currentMatchIndex.value = -1
			clearSelectedHighlight()
			if (matchMode.value !== 'closed') navigateHighlight(1)
			return
		}
		const selected = navigableMatches.value[currentMatchIndex.value]
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
	sourcePane.value?.focus({ preventScroll: true })
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
	[() => props.reportId, () => [props.filePath, props.jar], selectedLines],
	() =>
		emit('sourceView', {
			reportId: props.reportId,
			path: props.filePath,
			jar: props.jar ?? '',
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

function selectAllSourceText() {
	stopSourceSelection()
	selectedLines.value = undefined
	selectionAnchor = undefined
	const content = sourceContent.value
	const lines = content?.querySelectorAll('pre')
	const first = lines?.[0]
	const last = lines?.[lines.length - 1]
	if (!content || !first || !last) return
	const range = content.ownerDocument.createRange()
	range.setStart(first, 0)
	range.setEnd(last, last.childNodes.length)
	const selection = content.ownerDocument.getSelection()
	selection?.removeAllRanges()
	selection?.addRange(range)
}

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
			target.reportId !== props.reportId ||
			!matchesSourceLink(target, props.filePath, props.jar)
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
	if (!props.scrollable || !pane || !content || scrollHeight <= pane.clientHeight) {
		scrollbarMatches.value = []
		return
	}
	const positions = new Map<number, { global: boolean; trace: boolean }>()
	for (const row of content.querySelectorAll<HTMLElement>('.source-match-row')) {
		const position = ((row.offsetTop + row.offsetHeight / 2) / scrollHeight) * 100
		const line = Number(row.dataset.sourceLine)
		const previous = positions.get(position)
		positions.set(position, {
			global: !!previous?.global || globalMatches.value.has(line) || searchMatches.value.has(line),
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
	matchMode.value = 'traces'
	await nextTick()
	const firstMatch = sourceMatches.value(value).entries().next().value
	if (!firstMatch) return
	const [line, ranges] = firstMatch
	currentMatchIndex.value = navigableMatches.value.findIndex(
		(match) =>
			match.line === line &&
			match.range.start <= ranges[0].start &&
			match.range.end > ranges[0].start,
	)
	selectSourceMatch(line, ranges[0])
}

async function selectClickedMatch(event: MouseEvent) {
	const pane = sourcePane.value
	if (!pane || pane.ownerDocument.getSelection()?.isCollapsed === false) return
	const mark = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-source-match]')
	const row = mark?.closest<HTMLElement>('[data-source-line]')
	if (!mark || !row || !pane.contains(mark)) return
	const line = Number(row.dataset.sourceLine)
	const start = Number(mark.dataset.sourceStart)
	if (!mark.hasAttribute('data-search-source-match')) {
		matchMode.value = 'traces'
		await nextTick()
	}
	const index = navigableMatches.value.findIndex(
		(match) => match.line === line && match.range.start <= start && match.range.end > start,
	)
	if (index < 0) return
	currentMatchIndex.value = index
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

const sourceMatchRightPadding = 200

function selectSourceMatch(line: number, range: SourceMatch) {
	const pane = sourcePane.value
	const match = markSelectedSourceMatch(line, range)
	if (!pane || !match) return
	const bounds = pane.getBoundingClientRect()
	const target = match.getBoundingClientRect()
	const viewportLeft = bounds.left + pane.clientLeft
	const viewportRight = viewportLeft + pane.clientWidth - sourceMatchRightPadding
	let left = pane.scrollLeft
	if (
		target.left + pane.scrollLeft >= viewportLeft &&
		target.right + pane.scrollLeft <= viewportRight
	) {
		left = 0
	} else if (target.left < viewportLeft) {
		left += target.left - viewportLeft - 8
	} else if (target.right > viewportRight) {
		left += Math.max(0, Math.min(target.right - viewportRight, target.left - viewportLeft - 8))
	}
	const viewportTop = bounds.top + pane.clientTop
	const viewportBottom = viewportTop + pane.clientHeight
	const top =
		target.top >= viewportTop && target.bottom <= viewportBottom
			? pane.scrollTop
			: pane.scrollTop + target.top - viewportTop - (pane.clientHeight - target.height) / 2
	scrollSource(pane, top, left)
}
</script>

<template>
	<div
		v-if="expanded"
		ref="root"
		:class="
			scrollable
				? 'source-scroll-area relative col-start-1 row-start-2 min-h-0 min-w-0'
				: 'source-scroll-area relative min-w-0'
		"
	>
		<MatchToolbar
			ref="matchToolbar"
			v-model:query="findQuery"
			v-model:options="findOptions"
			:mode="matchMode"
			:has-trace-matches="!!highlightMatches.size"
			:current="currentMatchIndex"
			:total="navigableMatches.length"
			:error="isSearchMode ? findResult.error : ''"
			@search="openSearch"
			@traces="openTraceMatches"
			@close="closeMatchPanel"
			@navigate="navigateHighlight"
		/>
		<section
			ref="sourcePane"
			tabindex="-1"
			class="!outline-none"
			:class="[
				scrollable
					? 'review-scrollbars h-full min-h-0 min-w-0 overflow-auto pr-1 [container-type:inline-size]'
					: 'contents',
				{ 'overflow-x-hidden': scrollable && wrapCode },
			]"
			:aria-label="scrollable ? 'File content' : undefined"
		>
			<div
				v-if="highlightedSource.length > 0"
				ref="sourceContent"
				class="relative min-h-full bg-surface-1"
				:class="{
					'w-max min-w-full': scrollable && !wrapCode,
					'source-line-wrap w-full min-w-0': wrapCode,
				}"
			>
				<div
					data-source-code
					:data-report-id="reportId"
					:data-file-path="filePath"
					:data-jar="jar ?? ''"
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
							:data-line-number="n + 1"
							:aria-label="`Select line ${n + 1}`"
							:aria-pressed="isLineSelected(n)"
							title="Drag to select lines; Shift-click to extend. Enter or Space selects a line."
							@pointerdown="startLineSelection($event, n)"
							@click="$event.detail === 0 && selectGutterLine(n, $event.shiftKey)"
						/>
						<div
							class="min-w-0 flex-1 px-4 py-0 text-primary"
							:style="{
								paddingRight: scrollable && !wrapCode ? `${sourceMatchRightPadding}px` : undefined,
							}"
						>
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
			v-if="scrollable"
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
	<StringViewer ref="stringViewer" />
</template>

<style scoped>
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

.source-line-number:hover {
	background: color-mix(in srgb, var(--color-blue) 15%, transparent);
	color: var(--color-blue);
	box-shadow: inset 3px 0 var(--color-blue);
}

.source-line-number::before {
	content: attr(data-line-number);
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
	text-decoration-color: var(--color-purple);
}

:deep(mark[data-selected-source-match]) {
	background: color-mix(in srgb, var(--color-orange) 35%, transparent);
}

:deep(mark[data-selected-source-match][data-global-source-match]) {
	background: color-mix(in srgb, var(--color-purple) 45%, transparent);
}

:deep(mark[data-search-source-match]) {
	background: color-mix(in srgb, var(--color-blue) 45%, transparent);
	text-decoration: none;
}

:deep(mark[data-search-source-match][data-selected-source-match]) {
	background: color-mix(in srgb, var(--color-blue) 45%, transparent);
	outline: 1px solid var(--color-blue);
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
</style>
