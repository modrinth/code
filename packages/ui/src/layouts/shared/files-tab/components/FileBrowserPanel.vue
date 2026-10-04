
<template>
	<div
		ref="panelRoot"
		class="flex w-full scroll-mt-[var(--files-sticky-top,0px)] flex-col gap-3 p-1 pl-2"
		:class="{ 'snap-start': isFileActive }"
		:style="{
			'--files-navbar-height': `${navbarHeight}px`,
			'--files-trailing-space': `${trailingSpace}px`,
			'--files-table-header-top': `calc(var(--files-sticky-top, 0px) + ${navbarHeight}px + ${TAB_STRIP_HEIGHT}px)`,
		}"
	>
		<div
			ref="navbarWrapper"
			class="sticky top-[var(--files-sticky-top,0px)] z-30 bg-surface-1 py-1"
		>
		<FileNavbar
			:sidebar-open="sidebarOpen"
			:active-location="ui.activeLocation.value"
			:is-editing="ui.isEditing.value"
			:editing-file-name="ctx.currentFile.value?.name"
			:editing-file-path="ctx.currentFile.value?.path"
			:is-editing-image="ui.fileEditorApi.value?.isEditingImage.value ?? false"
			:is-editor-find-open="ui.fileEditorApi.value?.isFindOpen.value ?? false"
			:search-query="ui.searchQuery.value"
			:show-refresh-button="ui.showRefreshButton.value"
			:show-install-from-url="ctx.showInstallFromUrl"
			:base-id="ui.baseId"
			:disabled="ui.isBusy.value"
			:disabled-tooltip="ui.busyTooltip.value"
			:small-mode="props.smallMode"
			:can-go-back="ui.fileTabs.canGoBack.value"
			:can-go-forward="ui.fileTabs.canGoForward.value"
			@back="ui.fileTabs.back"
			@forward="ui.fileTabs.forward"
			@navigate="ui.navigateToSegment"
			@navigate-home="() => ui.navigateToSegment(0)"
			@prefetch-home="ui.handleHomePrefetch"
			@update:search-query="(value) => (ui.searchQuery.value = value)"
			@create="ui.showCreateModal"
			@upload="ui.initiateFileUpload"
			@upload-zip="() => {}"
			@unzip-from-url="ui.showUnzipFromUrlModal"
			@refresh="ctx.refresh"
			@share="() => ui.shareToMclogs()"
			@find="() => ui.toggleFind()"
			@toggle-sidebar="() => ui.setSidebarOpen(!sidebarOpen)"
		/>
		</div>
		<div
			class="@container relative flex flex-col overflow-clip rounded-[20px] border border-solid border-surface-4 shadow-sm"
		>
			<div
				class="shrink-0 overflow-hidden"
				:class="
					isFileActive
						? 'h-[calc(var(--files-viewport-height,100dvh)_-_var(--files-sticky-top,0px)_-_var(--files-navbar-height,3rem)_-_18px_-_var(--files-trailing-space,0px))] min-h-[24rem]'
						: 'sticky top-[calc(var(--files-sticky-top,0px)_+_var(--files-navbar-height,0px))] z-20 h-10'
				"
			>
				<FileTabs />
			</div>
			<FileManagerError
				v-if="!isFileActive && ctx.error.value"
				class="rounded-b-[20px]"
				:title="formatMessage(messages.errorTitle)"
				:message="formatMessage(messages.errorMessage)"
				@refetch="ctx.refresh"
				@home="() => ui.navigateToSegment(0)"
			/>
			<div v-else-if="!isFileActive">
				<FileUploadDragAndDrop
					ref="fileUploadRef"
					class=""
					:disabled="ui.isBusy.value"
					@drop-error="ui.handleDropError"
					@files-dropped="ui.handleDroppedFiles"
				>
					<FileTableHeader
						:sort-field="ui.sortField.value"
						:sort-desc="ui.sortDesc.value"
						:all-selected="ui.allSelected.value"
						:some-selected="ui.someSelected.value"
						:is-stuck="isLabelBarStuck"
						:columns="shownColumns"
						:enabled-columns="enabledColumns"
						:details-enabled="detailsEnabled"
						@sort="ui.handleSort"
						@toggle-all="ui.toggleSelectAll"
						@toggle-column="toggleColumn"
						@toggle-details="detailsEnabled = !detailsEnabled"
					/>
					<ReadyTransition :pending="listingPending">
					<div
						v-if="filteredItems.length > 0"
						ref="virtualListContainer"
						class="relative w-full"
						:style="{ minHeight: `${totalHeight}px`, overflowAnchor: 'none' }"
					>
						<div class="absolute w-full" :style="{ top: `${visibleTop}px` }">
							<FileRow
								v-for="(item, idx) in visibleItems"
								:key="item.path"
								:class="`h-[${itemHeight}]`"
								:file="item"
								:index="visibleRange.start + idx"
								:is-last="visibleRange.start + idx === filteredItems.length - 1"
								:selected="ui.selectedItems.value.has(item.path)"
								:write-disabled="ui.isBusy.value || !!ctx.isReadOnly?.(item)"
								:write-disabled-tooltip="ctx.isReadOnly?.(item) ? ctx.readOnlyReason?.value : ui.busyTooltip.value"
								:columns="shownColumns"
								:has-hidden-details="hasHiddenDetails"
								@extract="() => ui.handleExtractItem(item)"
								@delete="() => ui.showDeleteModal(item)"
								@rename="() => ui.showRenameModal(item)"
								@download="() => ui.handleDownload(item)"
								@zip="() => ui.handleZip(item)"
								@move="() => ui.showMoveModal(item)"
								@move-direct-to="ui.handleDirectMove"
								@edit="() => ui.handleNavigateTo(item)"
								@navigate="() => ui.handleNavigateTo(item)"
								@open-in-new-tab="() => ui.handleOpenInNewTab(item)"
								@hover="() => ui.handleItemPrefetch(item)"
								@contextmenu="ui.handleContextMenu"
								@toggle-select="() => ui.toggleItemSelection(item)"
								@create="ui.showCreateModal"
								@upload="ui.initiateFileUpload"
							/>
						</div>
					</div>
					<div
						v-else-if="ui.items.value.length === 0 && !ctx.error.value && !ctx.loading.value"
						class="flex h-full w-full items-center justify-center rounded-b-[20px] bg-surface-2 p-20"
					>
						<div class="flex flex-col items-center gap-4 text-center">
							<FolderOpenIcon class="h-16 w-16 text-secondary" />
							<h3 class="m-0 text-2xl font-bold text-contrast">
								{{ formatMessage(messages.emptyFolderTitle) }}
							</h3>
							<p class="m-0 text-sm text-secondary">
								{{ formatMessage(messages.emptyFolderDescription) }}
							</p>
						</div>
					</div>
					</ReadyTransition>
				</FileUploadDragAndDrop>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FolderOpenIcon } from '@modrinth/assets'
import { useElementSize, useEventListener } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { findScrollableAncestor, useVirtualScroll } from '#ui/composables/virtual-scroll.ts'
import { injectFileManager } from '#ui/layouts'
import { injectLoadingState } from '#ui/providers/loading-state'

import { useFileColumns } from '../composables/file-columns'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileManagerError from './FileManagerError.vue'
import FileNavbar from './FileNavbar.vue'
import FileRow from './FileRow.vue'
import FileTableHeader from './FileTableHeader.vue'
import FileTabs from './tabs/FileTabs.vue'
import FileUploadDragAndDrop from './upload/FileUploadDragAndDrop.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	emptyFolderTitle: {
		id: 'files.layout.empty-folder-title',
		defaultMessage: 'This folder is empty',
	},
	emptyFolderDescription: {
		id: 'files.layout.empty-folder-description',
		defaultMessage: 'There are no files or folders.',
	},
	errorTitle: {
		id: 'files.layout.error-title',
		defaultMessage: 'Unable to load files',
	},
	errorMessage: {
		id: 'files.layout.error-message',
		defaultMessage: 'The folder may not exist.',
	},
})

const ctx = injectFileManager()
const ui = injectFileBrowserUI()

const props = withDefaults(defineProps<{
	smallMode?: boolean
}>(), {
	smallMode: false
});

const sidebarOpen = computed(() => ui.sidebarOpen.value);
const isFileActive = computed(() => ui.fileTabs.activeLocation.value.type === 'file')

const filteredItems = computed(() => ui.filteredItems.value)

/** Only the listing fades while a directory loads for the first time; tabs, navbar and sidebar stay put. */
const listingPending = computed(() => ctx.loading.value && ui.items.value.length === 0)

const loadingState = injectLoadingState(null)
let refreshToken: symbol | null = null

function endRefreshLoading() {
	if (refreshToken) loadingState?.end(refreshToken)
	refreshToken = null
}

watch(
	() => ctx.isRefreshing.value,
	(refreshing) => {
		if (!refreshing) return endRefreshLoading()
		if (loadingState && !refreshToken) refreshToken = loadingState.begin()
	},
	{ immediate: true },
)
onBeforeUnmount(endRefreshLoading)

// Virtual scroll
const {
	listContainer: virtualListContainer,
	totalHeight,
	itemHeight,
	visibleRange,
	visibleTop,
	visibleItems,
} = useVirtualScroll(filteredItems, {
	itemHeight: 3.25,
	itemUnit: 'rem',
	bufferSize: 5,
})

const { detailsEnabled, enabledColumns, shownColumns, hasHiddenDetails, toggleColumn } =
	useFileColumns(ui.containerWidth)

const TAB_STRIP_HEIGHT = 40

const panelRoot = ref<HTMLElement | null>(null)
const navbarWrapper = ref<HTMLElement | null>(null)
const { height: navbarHeight } = useElementSize(navbarWrapper, undefined, { box: 'border-box' })

/** Where the table header sticks: below the host's sticky offset, the sticky navbar and the tab strip. */
const tableHeaderStickyTop = computed(() => {
	const hostOffset = navbarWrapper.value ? parseFloat(getComputedStyle(navbarWrapper.value).top) || 0 : 0
	return hostOffset + navbarHeight.value + TAB_STRIP_HEIGHT
})

const fileUploadRef = ref<InstanceType<typeof FileUploadDragAndDrop>>()
const fileUploadEl = computed(() => fileUploadRef.value?.$el as HTMLElement | null)
const { isStuck: isLabelBarStuck } = useStickyObserver(fileUploadEl, undefined, {
	topOffset: tableHeaderStickyTop,
})

/** Brings the navbar and file viewer to the top of the scroll area, unless they're already there. */
function scrollPanelIntoView() {
	const panel = panelRoot.value
	if (!panel) return

	const container = findScrollableAncestor(panel)
	const containerTop = container instanceof Window ? 0 : container.getBoundingClientRect().top
	const scrollMargin = parseFloat(getComputedStyle(panel).scrollMarginTop) || 0
	if (Math.abs(panel.getBoundingClientRect().top - containerTop - scrollMargin) < 2) return

	panel.scrollIntoView({ block: 'start', behavior: 'smooth' })
}

const activeFileKey = computed(() =>
	isFileActive.value
		? `${ui.fileTabs.activeTabId.value}:${ui.fileTabs.activeLocation.value.path}`
		: null,
)

/**
 * Scroll space below the file viewer within the page's content (stopping at `<main>`, so a site
 * footer doesn't count). The editor is shortened by this much so that scrolling to the end of the
 * page can never push the tab strip under the sticky navbar. Parents' `min-height` slack is
 * deliberately ignored, since it shrinks as the editor grows.
 */
const trailingSpace = ref(0)

function measureTrailingSpace() {
	const panel = panelRoot.value
	if (!panel) return

	const scrollContainer = findScrollableAncestor(panel)
	const boundary =
		panel.closest('main') ??
		(scrollContainer instanceof Window ? document.body : scrollContainer)

	let total = 0
	let element: HTMLElement = panel
	while (element !== boundary && element.parentElement) {
		const parent = element.parentElement
		const elementBottom = element.getBoundingClientRect().bottom
		let contentBottom = elementBottom + (parseFloat(getComputedStyle(element).marginBottom) || 0)

		for (let sibling = element.nextElementSibling; sibling; sibling = sibling.nextElementSibling) {
			const style = getComputedStyle(sibling)
			if (style.display === 'none' || style.position === 'absolute' || style.position === 'fixed') {
				continue
			}
			const siblingBottom = sibling.getBoundingClientRect().bottom + (parseFloat(style.marginBottom) || 0)
			contentBottom = Math.max(contentBottom, siblingBottom)
		}

		const parentStyle = getComputedStyle(parent)
		total +=
			contentBottom -
			elementBottom +
			(parseFloat(parentStyle.paddingBottom) || 0) +
			(parseFloat(parentStyle.borderBottomWidth) || 0)
		element = parent
	}

	trailingSpace.value = Math.max(0, Math.round(total))
}

watch(activeFileKey, (key) => {
	if (!key) return
	nextTick(() => {
		measureTrailingSpace()
		nextTick(scrollPanelIntoView)
	})
})

watch(navbarHeight, () => {
	if (isFileActive.value) nextTick(measureTrailingSpace)
})

useEventListener('resize', () => {
	if (isFileActive.value) measureTrailingSpace()
})

/**
 * While a file is open, the scroll container snaps (by proximity) to the file viewer, so it is
 * easy to bring the whole viewer into view without lining it up by hand.
 */
let snapContainer: HTMLElement | null = null
let previousSnapType = ''

function setViewerSnapping(enabled: boolean) {
	if (enabled && !snapContainer && panelRoot.value) {
		const container = findScrollableAncestor(panelRoot.value)
		snapContainer = container instanceof Window ? document.documentElement : container
		previousSnapType = snapContainer.style.scrollSnapType
		snapContainer.style.scrollSnapType = 'y proximity'
	} else if (!enabled && snapContainer) {
		snapContainer.style.scrollSnapType = previousSnapType
		snapContainer = null
	}
}

watch(isFileActive, setViewerSnapping, { flush: 'post' })
onMounted(() => {
	setViewerSnapping(isFileActive.value)
	if (!isFileActive.value) return
	measureTrailingSpace()
	nextTick(scrollPanelIntoView)
})
onBeforeUnmount(() => setViewerSnapping(false))
</script>
