
<template>
	<div
		ref="panelRoot"
		class="flex w-full scroll-mt-[var(--files-sticky-top,0px)] flex-col gap-3 p-1 pl-2"
		:class="{ 'snap-start': isFileActive }"
		:style="{ '--files-navbar-height': `${navbarHeight}px` }"
	>
		<div ref="navbarWrapper">
		<FileNavbar
			:sidebar-open="sidebarOpen"
			:breadcrumbs="ui.breadcrumbSegments.value"
			:is-editing="ui.isEditing.value"
			:editing-file-name="ctx.editingFile.value?.name"
			:editing-file-path="ctx.editingFile.value?.path"
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
			@navigate-home="() => {
				ui.navigateToSegment(-1)
			}"
			@prefetch-home="ui.handlePrefetchHome"
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
						? 'h-[calc(var(--files-viewport-height,100dvh)_-_var(--files-sticky-top,0px)_-_var(--files-navbar-height,3rem)_-_1.25rem_-_2px)] min-h-[24rem]'
						: 'h-10'
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
				@home="() => ui.navigateToSegment(-1)"
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
						:show-details="!ui.sidebarOpen.value"
						@sort="ui.handleSort"
						@toggle-all="ui.toggleSelectAll"
					/>
					<div
						v-if="filteredItems.length > 0"
						ref="virtualListContainer"
						class="relative w-full"
						:style="{ minHeight: `${totalHeight}px`, overflowAnchor: 'none' }"
					>
						<div class="absolute w-full" :style="{ top: `${visibleTop}px` }">
							<FileTableRow
								v-for="(item, idx) in visibleItems"
								:key="item.path"
								:count="item.count"
								:created="item.created"
								:modified="item.modified"
								:name="item.name"
								:path="item.path"
								:type="item.type"
								:size="item.size"
								:index="visibleRange.start + idx"
								:is-last="visibleRange.start + idx === filteredItems.length - 1"
								:selected="ui.selectedItems.value.has(item.path)"
								:write-disabled="ui.isBusy.value || !!ctx.isReadOnly?.(item.path)"
								:write-disabled-tooltip="ctx.isReadOnly?.(item.path) ? ctx.readOnlyReason?.value : ui.busyTooltip.value"
								:show-details="!ui.sidebarOpen.value"
								:container-width="ui.containerWidth.value"
								@extract="() => ui.handleExtractItem(item)"
								@delete="() => ui.showDeleteModal(item)"
								@rename="() => ui.showRenameModal(item)"
								@download="() => ui.handleDownload(item)"
								@zip="() => ui.handleZip(item)"
								@move="() => ui.showMoveModal(item)"
								@move-direct-to="ui.handleDirectMove"
								@edit="() => ui.handleEditFile(item)"
								@navigate="() => ui.handleNavigateToFolder(item)"
								@open-in-new-tab="() => ui.handleOpenInNewTab(item)"
								@hover="() => ui.handleItemHover(item)"
								@contextmenu="ui.handleContextMenu"
								@toggle-select="() => ui.toggleItemSelection(item)"
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
				</FileUploadDragAndDrop>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FolderOpenIcon } from '@modrinth/assets'
import { useElementSize } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { findScrollableAncestor, useVirtualScroll } from '#ui/composables/virtual-scroll.ts'
import { injectFileManager } from '#ui/layouts'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileManagerError from './FileManagerError.vue'
import FileNavbar from './FileNavbar.vue'
import FileTableHeader from './FileTableHeader.vue'
import FileTableRow from './FileTableRow.vue'
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
const isFileActive = computed(() => ui.fileTabs.activeLocation.value.kind === 'file')

const filteredItems = computed(() => ui.filteredItems.value)

// Virtual scroll
const {
	listContainer: virtualListContainer,
	totalHeight,
	visibleRange,
	visibleTop,
	visibleItems,
} = useVirtualScroll(filteredItems, {
	itemHeight: 52.8,
	bufferSize: 5,
})

// Sticky observer for the table header
const fileUploadRef = ref<InstanceType<typeof FileUploadDragAndDrop>>()
const fileUploadEl = computed(() => fileUploadRef.value?.$el as HTMLElement | null)
const { isStuck: isLabelBarStuck } = useStickyObserver(fileUploadEl)

const panelRoot = ref<HTMLElement | null>(null)
const navbarWrapper = ref<HTMLElement | null>(null)
const { height: navbarHeight } = useElementSize(navbarWrapper, undefined, { box: 'border-box' })

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

watch(activeFileKey, (key) => {
	if (key) nextTick(scrollPanelIntoView)
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
	if (isFileActive.value) nextTick(scrollPanelIntoView)
})
onBeforeUnmount(() => setViewerSnapping(false))
</script>
