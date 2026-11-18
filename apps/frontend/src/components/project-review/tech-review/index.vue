<template>
	<div
		ref="panel"
		class="review-scrollbars h-full min-h-0 min-w-0"
		:class="[
			!showArchiveReview && $slots.conversation
				? 'fallback-review-workspace grid'
				: 'flex flex-col gap-3',
			{ 'overflow-auto': !showArchiveReview && !$slots.conversation },
		]"
		:style="{ '--traces-width': `${tracesWidth}px` }"
	>
		<p v-if="!selection" class="m-0 text-secondary">No project selected</p>
		<div v-else-if="error || reportsQuery.isError.value" role="alert">
			<p class="m-0">Could not load project data</p>
			<Button @click="retry">Retry</Button>
		</div>
		<p v-else-if="isLoading || reportsQuery.isPending.value" role="status" class="m-0">
			Loading...
		</p>
		<p v-else-if="!reports.length" class="m-0 text-secondary">
			No technical-review reports are available for this project
		</p>
		<ArchiveReview
			v-else-if="showArchiveReview && selectedFile"
			ref="archiveReview"
			:key="selectedFile.id"
			:file="selectedFile"
			file-tree
			:focused-trace-id="focusedTraceId"
			:source-link="sourceLink"
			:loading-issues="loadingIssues"
			:settled-issues="settledIssues"
			:decompiled-sources="decompiledSources"
			@refetch="refreshReports"
			@source-view="syncSourceView"
			@source-link-handled="finishSourceLink"
		>
			<template v-if="$slots.conversation" #conversation>
				<div
					ref="conversationTarget"
					class="flex min-h-0 flex-1 flex-col gap-2.5"
					@vue:before-unmount="captureConversationScroll"
				/>
			</template>
			<template #header="{ verdictProps }">
				<div
					class="mb-2.5 flex shrink-0 items-center gap-2 border-0 border-b border-solid border-surface-3 pb-2.5"
				>
					<div class="group/file-actions flex min-w-0 items-center gap-1">
						<FloatingMenu
							ref="allArchivesMenu"
							class="min-w-0"
							placement="bottom-start"
							:arrow="false"
							bare
						>
							<h3 class="m-0 min-w-0 text-lg font-semibold text-contrast">
								<button
									type="button"
									class="flex max-w-full items-center gap-1 rounded-lg bg-transparent px-1.5 py-1 font-[inherit] text-[inherit] hover:bg-surface-3"
									:title="selectedFile.file_name"
									:aria-label="`${selectedFile.file_name}: choose archive`"
								>
									<span class="min-w-0 truncate">{{ selectedFile.file_name }}</span>
									<ChevronDownIcon class="size-4 shrink-0" aria-hidden="true" />
								</button>
							</h3>
							<template #popper>
								<ArchivePicker
									:files="reports"
									:selected-id="selectedFile.id"
									@select="selectFile"
								/>
							</template>
						</FloatingMenu>
						<FileActions :file="selectedFile" />
					</div>
					<VerdictButtons class="ml-auto shrink-0" v-bind="verdictProps" />
					<div
						v-if="pendingReports.length"
						aria-hidden="true"
						class="mx-2 h-6 w-px shrink-0 bg-divider"
					/>
					<div
						v-if="pendingReports.length"
						class="flex shrink-0 items-center"
						role="group"
						aria-label="Archive navigation"
					>
						<FloatingMenu ref="archiveMenu" placement="bottom-end" :arrow="false" bare>
							<button
								type="button"
								class="mr-1 rounded-full bg-transparent p-0 transition-[filter] hover:brightness-125"
								:aria-label="`Choose pending archive (${pendingReports.length} pending)`"
							>
								<TagItem style="--_bg-color: var(--color-orange-bg); --_color: var(--color-orange)">
									{{ pendingReports.length }} pending
								</TagItem>
							</button>
							<template #popper>
								<ArchivePicker
									:files="pendingReports"
									:selected-id="selectedFile.id"
									label="Pending archives"
									@select="selectFile"
								/>
							</template>
						</FloatingMenu>
						<IconButton
							v-tooltip="'Previous pending archive'"
							type="quiet"
							size="sm"
							circular
							label="Previous pending archive"
							:disabled="!previousFile"
							@click="previousFile && selectFile(previousFile)"
						>
							<ChevronLeftIcon aria-hidden="true" />
						</IconButton>
						<IconButton
							v-tooltip="'Next pending archive'"
							type="quiet"
							size="sm"
							circular
							label="Next pending archive"
							:disabled="!nextFile"
							@click="nextFile && selectFile(nextFile)"
						>
							<ChevronRightIcon aria-hidden="true" />
						</IconButton>
					</div>
				</div>
				<div v-if="sourceQueries.some((query) => query.isError)" role="alert">
					<p class="m-0">Could not load source code for some files.</p>
					<Button @click="retrySources">Retry</Button>
				</div>
			</template>
		</ArchiveReview>
		<p v-else class="m-0 text-secondary">No archive selected</p>
		<PanelDivider
			v-if="$slots.conversation && !showArchiveReview"
			v-model="tracesWidth"
			label="Conversation width"
			width-property="--traces-width"
			side="right"
			:min="220"
			:max="720"
			:reserve="200"
			:default-width="360"
		/>
		<div
			v-if="$slots.conversation"
			v-show="!conversationTarget"
			ref="conversationFallback"
			class="flex min-h-0 min-w-0 flex-1 flex-col gap-2.5 pb-1 pl-1"
		/>
		<Teleport
			v-if="$slots.conversation && conversationFallback"
			:to="conversationTarget ?? conversationFallback"
		>
			<div ref="conversation" class="flex min-h-0 flex-1 flex-col gap-2.5">
				<slot name="conversation" />
			</div>
		</Teleport>
		<Teleport v-if="codeSelection && panel" :to="panel.ownerDocument.body">
			<Button
				ref="selectionAction"
				class="!fixed z-50 !font-medium shadow-lg"
				:style="{
					left: `${codeSelection.left}px`,
					top: `${codeSelection.top}px`,
					maxWidth: 'calc(100vw - 16px)',
				}"
				:disabled="pending"
				aria-label="Add to message"
				@pointerdown.prevent
				@click="addSelectedCode"
			>
				<span class="min-w-0 truncate">Add to message</span>
				<ArrowRightToLineIcon aria-hidden="true" />
			</Button>
		</Teleport>
	</div>
</template>

<script setup lang="ts">
import {
	ArrowRightToLineIcon,
	ChevronDownIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
} from '@modrinth/assets'
import { Button, codeClipboardHtml, IconButton, injectModrinthClient, TagItem } from '@modrinth/ui'
import { type QueryFunctionContext, useQueries } from '@tanstack/vue-query'
import { useLocalStorage } from '@vueuse/core'
import { computed, nextTick, provide, ref, shallowRef, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { injectProjectReviewContext, injectReviewPanelVisible } from '../layout/context'
import { injectReviewShortcuts, reviewShortcutBlocked, useReviewEventListener } from '../shortcuts'
import ArchivePicker from './archive-picker/index.vue'
import ArchiveReview from './archive-review/index.vue'
import FileActions from './file-actions.vue'
import FloatingMenu from './menu.vue'
import { provideTechReviewMenus } from './menus'
import PanelDivider from './panel-divider.vue'
import {
	parseSourceLines,
	type SourceLinkTarget,
	sourceQueryKey,
	type SourceView,
} from './source-link'
import { createSourceRequestQueue } from './source-request-queue'
import type { FlattenedFileReport } from './types'
import { TECH_REVIEW_DECISIONS_KEY } from './use-tech-review-decisions'
import VerdictButtons from './verdict-buttons/index.vue'

const emit = defineEmits<{
	addCode: [code: string, link: string]
	ready: [projectId: string]
}>()
const { pending } = injectReviewSubmission()
const panel = ref<HTMLElement>()
const tracesWidth = useLocalStorage('moderation-tech-review-traces-width', 360)
const conversationTarget = ref<HTMLElement>()
const conversationFallback = ref<HTMLElement>()
const conversation = ref<HTMLElement>()
let conversationScroll: { element: Element; top: number; left: number }[] = []

function captureConversationScroll() {
	conversationScroll = [...(conversation.value?.querySelectorAll('*') ?? [])]
		.filter((element) => element.scrollTop || element.scrollLeft)
		.map((element) => ({
			element,
			top: element.scrollTop,
			left: element.scrollLeft,
		}))
}

watch(
	conversationTarget,
	async () => {
		if (!conversationScroll.length) captureConversationScroll()
		await nextTick()
		for (const { element, top, left } of conversationScroll) {
			element.scrollTop = top
			element.scrollLeft = left
		}
		conversationScroll = []
	},
	{ flush: 'pre' },
)
const archiveReview = ref<InstanceType<typeof ArchiveReview>>()
const panelVisible = injectReviewPanelVisible()
const keybinds = useModerationKeybinds()

function getSelectedCode() {
	const document = panel.value?.ownerDocument
	const selection = document?.getSelection()
	if (
		!document ||
		!selection ||
		selection.isCollapsed ||
		!selection.rangeCount ||
		!panel.value?.getClientRects().length
	)
		return
	const range = selection.getRangeAt(0)
	const container = range.startContainer.parentElement?.closest('[data-source-code]')
	if (!container || !panel.value.contains(container) || !container.contains(range.endContainer))
		return
	const lines: string[] = []
	for (const line of container.querySelectorAll('pre')) {
		if (!range.intersectsNode(line)) continue
		const selected = document.createRange()
		selected.selectNodeContents(line)
		let leadingIndentation = ''
		if (range.compareBoundaryPoints(range.START_TO_START, selected) > 0) {
			const prefix = selected.cloneRange()
			prefix.setEnd(range.startContainer, range.startOffset)
			const unselectedPrefix = prefix.toString()
			if (!lines.length && /^[\t ]*$/.test(unselectedPrefix)) {
				leadingIndentation = unselectedPrefix
			}
			selected.setStart(range.startContainer, range.startOffset)
		}
		if (range.compareBoundaryPoints(range.END_TO_END, selected) < 0)
			selected.setEnd(range.endContainer, range.endOffset)
		lines.push(leadingIndentation + selected.toString())
	}
	return lines.join('\n')
}

const shortcuts = injectReviewShortcuts()
const selectionAction = ref<InstanceType<typeof Button>>()
const codeSelection = ref<{
	code: string
	link: string
	left: number
	top: number
}>()
let selectingCode = false
let dismissedCodeSelection: Range | undefined

watch(
	codeSelection,
	(selection) => {
		const action = selectionAction.value?.element
		const view = action?.ownerDocument.defaultView
		if (!selection || !action || !view) return
		selection.left = Math.max(
			8,
			Math.min(selection.left, view.innerWidth - action.getBoundingClientRect().width - 8),
		)
	},
	{ flush: 'post' },
)

function updateCodeSelection() {
	codeSelection.value = undefined
	if (selectingCode) return
	const document = panel.value?.ownerDocument
	const selection = document?.getSelection()
	const range = selection?.rangeCount ? selection.getRangeAt(0) : undefined
	if (
		range &&
		dismissedCodeSelection &&
		range.startContainer === dismissedCodeSelection.startContainer &&
		range.startOffset === dismissedCodeSelection.startOffset &&
		range.endContainer === dismissedCodeSelection.endContainer &&
		range.endOffset === dismissedCodeSelection.endOffset
	)
		return
	dismissedCodeSelection = undefined
	const container = range?.startContainer.parentElement?.closest('.source-line-selection')
	if (!container || !panel.value?.contains(container)) return
	const code = getSelectedCode()
	const view = document?.defaultView
	if (!code?.trim() || !view || !range) return
	const start = Number(
		range.startContainer.parentElement?.closest<HTMLElement>('[data-source-line]')?.dataset
			.sourceLine,
	)
	const end = Number(
		range.endContainer.parentElement?.closest<HTMLElement>('[data-source-line]')?.dataset
			.sourceLine,
	)
	const source = container as HTMLElement
	if (
		!Number.isSafeInteger(start) ||
		!Number.isSafeInteger(end) ||
		!source.dataset.reportId ||
		!source.dataset.filePath
	)
		return
	const href = router.resolve({
		path: route.path,
		query: {
			...route.query,
			detail: undefined,
			highlightTrace: undefined,
			tech_review_report: source.dataset.reportId,
			tech_review_path: source.dataset.filePath,
			tech_review_jar: source.dataset.jar ?? '',
			lines: `${start + 1}-${end + 1}`,
		},
	}).href
	const bounds = range.getBoundingClientRect()
	if (!bounds.width && !bounds.height) return
	codeSelection.value = {
		code,
		link: new URL(href, source.ownerDocument.location.href).href,
		left: Math.max(8, bounds.left),
		top: Math.max(8, Math.min(bounds.bottom + 8, view.innerHeight - 48)),
	}
}

function scheduleCodeSelectionUpdate() {
	void nextTick(updateCodeSelection)
}

function addSelectedCode() {
	if (!codeSelection.value || pending.value) return
	const { code, link } = codeSelection.value
	panel.value?.ownerDocument.getSelection()?.removeAllRanges()
	codeSelection.value = undefined
	emit('addCode', code, link)
}

watch(
	shortcuts.windows,
	(windows, _, onCleanup) => {
		for (const view of windows)
			view.document.addEventListener('selectionchange', scheduleCodeSelectionUpdate)
		onCleanup(() => {
			for (const view of windows)
				view.document.removeEventListener('selectionchange', scheduleCodeSelectionUpdate)
		})
	},
	{ immediate: true },
)
useReviewEventListener(
	'pointerdown',
	(event) => {
		const target = event.target as HTMLElement | null
		if (event.button !== 0 || !target || !panel.value?.contains(target)) return
		if (!target.closest('[data-source-code]')) return
		dismissedCodeSelection = undefined
		selectingCode = true
		codeSelection.value = undefined
	},
	{ capture: true },
)
useReviewEventListener('pointerup', () => {
	if (!selectingCode) return
	selectingCode = false
	scheduleCodeSelectionUpdate()
})
useReviewEventListener('pointercancel', () => {
	selectingCode = false
	codeSelection.value = undefined
})
useReviewEventListener('blur', () => {
	selectingCode = false
	codeSelection.value = undefined
})
useReviewEventListener('scroll', scheduleCodeSelectionUpdate, { capture: true })
useReviewEventListener('resize', scheduleCodeSelectionUpdate)

useReviewEventListener(
	'keydown',
	(event) => {
		const target = event.target as HTMLElement | null
		if (
			(event.key === 'Enter' || event.key === ' ') &&
			target?.closest('.source-line-number') &&
			panel.value?.contains(target)
		)
			dismissedCodeSelection = undefined
		if (event.key !== 'Escape' || event.defaultPrevented || !panelVisible.value) return
		const document = panel.value?.ownerDocument
		if (
			target?.ownerDocument !== document ||
			target?.closest('input, textarea, select, [contenteditable="true"], [role="dialog"]')
		)
			return
		const selection = document?.getSelection()
		const range = selection?.rangeCount ? selection.getRangeAt(0) : undefined
		const container = range?.startContainer.parentElement?.closest('.source-line-selection')
		if (!range || selection?.isCollapsed || !container || !panel.value?.contains(container)) return
		event.preventDefault()
		event.stopImmediatePropagation()
		if (event.repeat) return
		if (codeSelection.value) {
			dismissedCodeSelection = range.cloneRange()
			codeSelection.value = undefined
		} else {
			dismissedCodeSelection = undefined
			selection?.removeAllRanges()
		}
	},
	{ capture: true },
)

useReviewEventListener('copy', (event) => {
	if (event.defaultPrevented || !event.clipboardData) return
	if ((event.target as Node | null)?.ownerDocument !== panel.value?.ownerDocument) return
	const code = getSelectedCode()
	if (!code || !panel.value) return
	event.clipboardData.setData('text/plain', code)
	event.clipboardData.setData(
		'text/html',
		codeClipboardHtml(panel.value.ownerDocument, code, 'java'),
	)
	event.preventDefault()
})

const client = injectModrinthClient()
const route = useRoute()
const router = useRouter()
const sourceLink = shallowRef<SourceLinkTarget>()
let localSourceQuery: string | undefined
watch(
	() => sourceQueryKey(route.query),
	(key) => {
		if (key === localSourceQuery) return
		localSourceQuery = undefined
		const reportId = route.query.tech_review_report
		const path = route.query.tech_review_path
		sourceLink.value =
			typeof reportId === 'string' && reportId && typeof path === 'string' && path
				? {
						reportId,
						path,
						jar:
							typeof route.query.tech_review_jar === 'string'
								? route.query.tech_review_jar
								: undefined,
						lines: parseSourceLines(route.query.lines),
					}
				: undefined
	},
	{ immediate: true },
)

function syncSourceView(view: SourceView) {
	const query = {
		...route.query,
		tech_review_report: view.reportId,
		tech_review_path: view.path,
		tech_review_jar: view.path ? view.jar : undefined,
		lines: view.lines ? `${view.lines.start}-${view.lines.end}` : undefined,
	}
	const key = sourceQueryKey(query)
	if (key === sourceQueryKey(route.query)) return
	localSourceQuery = key
	void router.replace({ query })
}

function finishSourceLink(target: SourceLinkTarget) {
	if (sourceLink.value === target) sourceLink.value = undefined
}
const { openTab } = injectProjectReviewContext()
/** Falls back to `detail` for backward compatibility with existing review links. */
const focusedTraceId = computed(() =>
	typeof route.query.highlightTrace === 'string'
		? route.query.highlightTrace
		: typeof route.query.detail === 'string'
			? route.query.detail
			: null,
)
const {
	selection,
	projectId,
	project,
	isLoading,
	error,
	refresh,
	reportsQuery,
	reports,
	pendingReports,
	selectedArchiveId: selectedFileId,
	decisions,
	refreshReports,
} = injectProjectReviewPageContext()
provide(TECH_REVIEW_DECISIONS_KEY, decisions)
const archiveMenu = ref<InstanceType<typeof FloatingMenu>>()
const allArchivesMenu = ref<InstanceType<typeof FloatingMenu>>()
provideTechReviewMenus()
const selectedFile = computed(
	() =>
		reports.value.find((file) => file.id === selectedFileId.value) ??
		pendingReports.value[0] ??
		reports.value[0],
)
watch(
	() => selectedFile.value?.id,
	(reportId) => {
		if (reportId) syncSourceView({ reportId })
	},
	{ immediate: true },
)
const showArchiveReview = computed(
	() =>
		!!selection.value &&
		!error.value &&
		!reportsQuery.isError.value &&
		!isLoading.value &&
		!reportsQuery.isPending.value &&
		reports.value.length > 0 &&
		!!project.value &&
		!!selectedFile.value,
)
if (import.meta.dev) {
	watch(
		[showArchiveReview, isLoading, reportsQuery.isPending, error, reportsQuery.isError],
		([archiveReviewVisible, loading, reportsPending, projectError, reportsError]) => {
			console.debug('[tech-review layout]', {
				archiveReviewVisible,
				loading,
				reportsPending,
				projectError: !!projectError,
				reportsError,
			})
		},
		{ immediate: true },
	)
}
const selectedQueueIndex = computed(() =>
	pendingReports.value.findIndex((file) => file.id === selectedFile.value?.id),
)
const previousFile = computed(() => pendingReports.value[selectedQueueIndex.value - 1])
const nextFile = computed(() => pendingReports.value[selectedQueueIndex.value + 1])
let traceLinkHandled = false
let handledSourceLink: typeof sourceLink.value
watch(
	[sourceLink, reports],
	async ([target, files]) => {
		if (!target || handledSourceLink === target) return
		const file = files.find((file) => file.id === target.reportId)
		if (!file) return
		handledSourceLink = target
		selectedFileId.value = file.id
		await nextTick()
		if (sourceLink.value === target) openTab('tech-review')
	},
	{ immediate: true },
)
watch(
	[focusedTraceId, reports],
	async ([detailId, files], [previousDetailId]) => {
		if (detailId !== previousDetailId) traceLinkHandled = false
		if (!detailId || traceLinkHandled || sourceLink.value) return
		const file = files.find((file) =>
			file.issues.some((issue) => issue.details.some((detail) => detail.id === detailId)),
		)
		if (!file) return
		traceLinkHandled = true
		selectedFileId.value = file.id
		await nextTick()
		openTab('tech-review')
	},
	{ immediate: true },
)
const requestSource = createSourceRequestQueue()
const issueIds = computed(() => [
	...new Set(selectedFile.value?.issues.map((issue) => issue.id) ?? []),
])
const sourceQueries = useQueries({
	queries: computed(() =>
		issueIds.value.map((id) => ({
			queryKey: ['tech-review', 'issue', id, { includeHidden: true }] as const,
			queryFn: ({ signal }: QueryFunctionContext) =>
				requestSource(
					() =>
						client.labrinth.tech_review_internal.getIssue(id, {
							include_hidden: true,
						}),
					signal,
				),
			staleTime: 24 * 60 * 60 * 1000,
		})),
	),
})
const loadingIssues = computed(
	() => new Set(issueIds.value.filter((_id, index) => sourceQueries.value[index]?.isFetching)),
)
const settledIssues = computed(
	() =>
		new Set(
			issueIds.value.filter(
				(_id, index) =>
					sourceQueries.value[index]?.isSuccess || sourceQueries.value[index]?.isError,
			),
		),
)

const decompiledSources = computed(
	() =>
		new Map(
			sourceQueries.value.flatMap((query) =>
				(query.data?.details ?? []).flatMap((detail) =>
					detail.decompiled_source ? [[detail.id, detail.decompiled_source] as const] : [],
				),
			),
		),
)
watch(
	() =>
		!isLoading.value &&
		!reportsQuery.isPending.value &&
		issueIds.value.every(
			(_id, index) => sourceQueries.value[index]?.isSuccess || sourceQueries.value[index]?.isError,
		),
	(ready) => {
		if (ready) emit('ready', projectId.value)
	},
	{ immediate: true, flush: 'post' },
)
function selectFile(file: FlattenedFileReport) {
	sourceLink.value = undefined
	archiveMenu.value?.hide()
	allArchivesMenu.value?.hide()
	selectedFileId.value = file.id
}

const navigationActions = {
	'pass-file': {
		available: () => archiveReview.value?.canMarkCurrentFile() ?? false,
		run: () => archiveReview.value?.markCurrentFile('safe'),
	},
	'fail-file': {
		available: () => archiveReview.value?.canMarkCurrentFile() ?? false,
		run: () => archiveReview.value?.markCurrentFile('unsafe'),
	},
	'global-pass-file': {
		available: () => archiveReview.value?.canMarkCurrentFile('global') ?? false,
		run: () => archiveReview.value?.markCurrentFile('safe', 'global'),
	},
	'global-fail-file': {
		available: () => archiveReview.value?.canMarkCurrentFile('global') ?? false,
		run: () => archiveReview.value?.markCurrentFile('unsafe', 'global'),
	},
	'previous-highlight': {
		available: () => archiveReview.value?.canNavigateHighlight() ?? false,
		run: () => archiveReview.value?.navigateHighlight(-1),
	},
	'next-highlight': {
		available: () => archiveReview.value?.canNavigateHighlight() ?? false,
		run: () => archiveReview.value?.navigateHighlight(1),
	},
	'previous-file': {
		available: () => archiveReview.value?.canNavigateFile(-1) ?? false,
		run: () => archiveReview.value?.navigateFile(-1),
	},
	'next-file': {
		available: () => archiveReview.value?.canNavigateFile(1) ?? false,
		run: () => archiveReview.value?.navigateFile(1),
	},
	'previous-archive': {
		available: () => !!previousFile.value,
		run: () => previousFile.value && selectFile(previousFile.value),
	},
	'next-archive': {
		available: () => !!nextFile.value,
		run: () => nextFile.value && selectFile(nextFile.value),
	},
}

useReviewEventListener('keydown', (event) => {
	const target = event.target as HTMLElement | null
	const element = panel.value
	if (
		!panelVisible.value ||
		!element?.getClientRects().length ||
		target?.ownerDocument !== element.ownerDocument ||
		reviewShortcutBlocked(event, true) ||
		target?.isContentEditable ||
		target?.closest(
			'input, textarea, select, [role="textbox"], [role="dialog"], [role="menu"], [role="listbox"], [role="separator"], .cm-editor',
		)
	)
		return
	keybinds.value.handle(event, {
		scope: 'tech-review',
		actions: {
			available: (action) =>
				(!event.repeat || action === 'previous-highlight' || action === 'next-highlight') &&
				navigationActions[action].available(),
			run: (action) => navigationActions[action].run(),
		},
	})
})
async function retrySources() {
	await Promise.all(
		sourceQueries.value.filter((query) => query.isError).map((query) => query.refetch()),
	)
}
async function retry() {
	await Promise.all([refresh(), reportsQuery.refetch()])
}
</script>

<style scoped>
.fallback-review-workspace {
	grid-template-columns: minmax(0, 1fr) 8px min(var(--traces-width), max(220px, calc(100% - 208px)));
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

.review-scrollbars:hover {
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
</style>
