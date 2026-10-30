<template>
	<div
		ref="panel"
		class="review-scrollbars flex h-full min-h-0 min-w-0 flex-col gap-3"
		:class="{ 'overflow-auto': !selectedFile }"
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
		<FileDetail
			v-else-if="project && selectedFile"
			ref="fileDetail"
			:key="selectedFile.id"
			:file="selectedFile"
			file-tree
			:focused-detail-id="focusedDetailId"
			:loading-issues="loadingIssues"
			:settled-issues="settledIssues"
			:decompiled-sources="decompiledSources"
			@refetch="refreshReports"
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
						<h3
							class="m-0 min-w-0 truncate pl-1 text-base font-semibold text-contrast"
							:title="selectedFile.file_name"
						>
							{{ selectedFile.file_name }}
						</h3>
						<FileActions :file="selectedFile" />
					</div>
					<div
						class="ml-auto flex shrink-0 items-center gap-0.5"
						role="group"
						aria-label="Archive navigation"
					>
						<Button
							v-tooltip="'Previous archive'"
							type="quiet"
							size="sm"
							icon-only
							aria-label="Previous archive"
							:disabled="!previousFile"
							@click="previousFile && selectFile(previousFile)"
						>
							<ChevronLeftIcon aria-hidden="true" />
						</Button>
						<FloatingMenu ref="archiveMenu" placement="bottom-end" :arrow="false" bare>
							<Button
								type="quiet"
								size="sm"
								class="tabular-nums"
								:aria-label="`Choose archive (${selectedFileIndex + 1}/${reports.length})`"
							>
								{{ selectedFileIndex + 1 }}/{{ reports.length }}
							</Button>
							<template #popper>
								<section
									aria-label="Archives"
									class="review-scrollbars w-[30rem] max-w-[calc(100vw-2rem)] rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
								>
									<h4 class="m-0 px-3 pb-2 pt-1.5 text-xs font-medium text-secondary">
										Archives ({{ reports.length }})
									</h4>
									<div class="flex max-h-[60vh] flex-col gap-1 overflow-y-auto">
										<FileItem
											v-for="file in reports"
											:key="file.id"
											:file="file"
											:selected="file.id === selectedFile.id"
											@view-flags="selectFile"
										/>
									</div>
								</section>
							</template>
						</FloatingMenu>
						<Button
							v-tooltip="'Next archive'"
							type="quiet"
							size="sm"
							icon-only
							aria-label="Next archive"
							:disabled="!nextFile"
							@click="nextFile && selectFile(nextFile)"
						>
							<ChevronRightIcon aria-hidden="true" />
						</Button>
					</div>
					<VerdictButtons class="shrink-0" v-bind="verdictProps" />
				</div>
				<div v-if="sourceQueries.some((query) => query.isError)" role="alert">
					<p class="m-0">Could not load source code for some files.</p>
					<Button @click="retrySources">Retry</Button>
				</div>
			</template>
		</FileDetail>
		<div
			v-if="$slots.conversation"
			v-show="!conversationTarget"
			ref="conversationFallback"
			class="flex min-h-0 flex-1 flex-col gap-2.5 border-0 border-t border-solid border-divider pt-3"
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
import { ArrowRightToLineIcon, ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import { Button, codeClipboardHtml, FloatingMenu, injectModrinthClient } from '@modrinth/ui'
import { type QueryFunctionContext, useQueries } from '@tanstack/vue-query'
import { computed, nextTick, provide, ref, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { injectProjectReviewContext, injectReviewPanelVisible } from '../layout/context'
import { injectReviewShortcuts, reviewShortcutBlocked, useReviewEventListener } from '../shortcuts'
import FileDetail from './file-detail/index.vue'
import FileActions from './files/actions.vue'
import FileItem from './files/item.vue'
import { createSourceRequestQueue } from './source-request-queue'
import type { FlattenedFileReport } from './types'
import VerdictButtons from './verdict/buttons.vue'
import { TECH_REVIEW_DECISIONS_KEY } from './verdict/use-decisions'

const emit = defineEmits<{ addCode: [code: string] }>()
const { pending } = injectReviewSubmission()
const panel = ref<HTMLElement>()
const conversationTarget = ref<HTMLElement>()
const conversationFallback = ref<HTMLElement>()
const conversation = ref<HTMLElement>()
let conversationScroll: { element: Element; top: number; left: number }[] = []

function captureConversationScroll() {
	conversationScroll = [...(conversation.value?.querySelectorAll('*') ?? [])]
		.filter((element) => element.scrollTop || element.scrollLeft)
		.map((element) => ({ element, top: element.scrollTop, left: element.scrollLeft }))
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
const fileDetail = ref<InstanceType<typeof FileDetail>>()
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
const codeSelection = ref<{ code: string; left: number; top: number }>()
let selectingCode = false

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
	const container = range?.startContainer.parentElement?.closest('.source-line-selection')
	if (!container || !panel.value?.contains(container)) return
	const code = getSelectedCode()
	const view = document?.defaultView
	if (!code?.trim() || !view || !range) return
	const bounds = range.getBoundingClientRect()
	if (!bounds.width && !bounds.height) return
	codeSelection.value = {
		code,
		left: Math.max(8, bounds.left),
		top: Math.max(8, Math.min(bounds.bottom + 8, view.innerHeight - 48)),
	}
}

function scheduleCodeSelectionUpdate() {
	void nextTick(updateCodeSelection)
}

function addSelectedCode() {
	if (!codeSelection.value || pending.value) return
	const code = codeSelection.value.code
	panel.value?.ownerDocument.getSelection()?.removeAllRanges()
	codeSelection.value = undefined
	emit('addCode', code)
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
const { openTab } = injectProjectReviewContext()
const focusedDetailId = computed(() =>
	typeof route.query.detail === 'string' ? route.query.detail : null,
)
const {
	selection,
	project,
	isLoading,
	error,
	refresh,
	reportsQuery,
	reports,
	decisions,
	refreshReports,
} = injectProjectReviewPageContext()
provide(TECH_REVIEW_DECISIONS_KEY, decisions)
const archiveMenu = ref<InstanceType<typeof FloatingMenu>>()
const selectedFileId = ref<string>()
const selectedFile = computed(
	() => reports.value.find((file) => file.id === selectedFileId.value) ?? reports.value[0],
)
const selectedFileIndex = computed(() =>
	reports.value.findIndex((file) => file.id === selectedFile.value?.id),
)
const previousFile = computed(() => reports.value[selectedFileIndex.value - 1])
const nextFile = computed(() => reports.value[selectedFileIndex.value + 1])
let detailLinkHandled = false
watch(
	[focusedDetailId, reports],
	async ([detailId, files], [previousDetailId]) => {
		if (detailId !== previousDetailId) detailLinkHandled = false
		if (!detailId || detailLinkHandled) return
		const file = files.find((file) =>
			file.issues.some((issue) => issue.details.some((detail) => detail.id === detailId)),
		)
		if (!file) return
		detailLinkHandled = true
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
function selectFile(file: FlattenedFileReport) {
	archiveMenu.value?.hide()
	selectedFileId.value = file.id
}

const navigationActions = {
	'previous-highlight': {
		available: () => fileDetail.value?.canNavigateHighlight() ?? false,
		run: () => fileDetail.value?.navigateHighlight(-1),
	},
	'next-highlight': {
		available: () => fileDetail.value?.canNavigateHighlight() ?? false,
		run: () => fileDetail.value?.navigateHighlight(1),
	},
	'previous-file': {
		available: () => fileDetail.value?.canNavigateFile(-1) ?? false,
		run: () => fileDetail.value?.navigateFile(-1),
	},
	'next-file': {
		available: () => fileDetail.value?.canNavigateFile(1) ?? false,
		run: () => fileDetail.value?.navigateFile(1),
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
