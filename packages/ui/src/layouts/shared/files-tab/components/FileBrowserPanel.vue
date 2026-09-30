
<template>
	<div class="flex flex-col p-1 pl-2 gap-3 w-full">
		<FileNavbar
			:sidebar-open="sidebarOpen"
			:breadcrumbs="ui.breadcrumbSegments.value"
			:is-editing="ui.isEditing.value"
			:editing-file-name="ctx.editingFile.value?.name"
			:editing-file-path="ctx.editingFile.value?.path"
			:is-editing-image="fileEditorRef?.isEditingImage"
			:is-editor-find-open="fileEditorRef?.isFindOpen"
			:search-query="ui.searchQuery.value"
			:show-refresh-button="ui.showRefreshButton.value"
			:show-install-from-url="ctx.showInstallFromUrl"
			:base-id="ui.baseId"
			:disabled="ui.isBusy.value"
			:disabled-tooltip="ui.busyTooltip.value"
			:small-mode="props.smallMode"
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
			@share="() => fileEditorRef?.shareToMclogs()"
			@find="() => fileEditorRef?.toggleFind()"
			@toggle-sidebar="() => ui.setSidebarOpen(!sidebarOpen)"
		/>
		<div class="@container relative flex flex-col overflow-clip rounded-[20px] border border-solid border-surface-4 shadow-sm">
			<div v-if="!ui.isEditing.value">
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
								@hover="() => ui.handleItemHover(item)"
								@contextmenu="ui.handleContextMenu"
								@toggle-select="() => ui.toggleItemSelection(item.path)"
							/>
						</div>
					</div>
					<div
						v-else-if="ui.items.value.length === 0 && !ctx.error.value"
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
			<FileManagerError
				v-else-if="ctx.error.value"
				class="rounded-b-[20px]"
				:title="formatMessage(messages.errorTitle)"
				:message="formatMessage(messages.errorMessage)"
				@refetch="ctx.refresh"
				@home="() => ui.navigateToSegment(-1)"
			/>
			<FileEditor
				v-else
				ref="fileEditorRef"
				:file="ctx.editingFile.value"
				:editor-component="ui.editorComponent.value"
				@close="handleEditorClose"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FolderOpenIcon } from '@modrinth/assets'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { useVirtualScroll } from '#ui/composables/virtual-scroll.ts'
import { injectFileManager } from '#ui/layouts'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileEditor from './editor/FileEditor.vue'
import FileManagerError from './FileManagerError.vue'
import FileNavbar from './FileNavbar.vue'
import FileActionBar from './FileActionBar.vue'
import FileTableHeader from './FileTableHeader.vue'
import FileTableRow from './FileTableRow.vue'
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

const fileEditorRef = ref<InstanceType<typeof FileEditor>>()

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

async function handleEditorClose() {
	await ui.handleEditorClose()
}

const hasUnsavedChanges = computed(() => fileEditorRef.value?.hasUnsavedChanges ?? false)
const isEditingImage = computed(() => fileEditorRef.value?.isEditingImage ?? false)
const isFindOpen = computed(() => fileEditorRef.value?.isFindOpen ?? false)
const saveFileContent = async (exit = false) => {
	await fileEditorRef.value?.saveFileContent(exit)
}
const revertChanges = () => fileEditorRef.value?.revertChanges()
const shareToMclogs = async () => {
	await fileEditorRef.value?.shareToMclogs()
}
const toggleFind = () => fileEditorRef.value?.toggleFind()

// Register this panel's editor API on the shared context so layout.vue and FileSideBar
// (which can no longer hold a template ref to a dockview-hosted panel) can reach it.
const fileEditorApi = {
	hasUnsavedChanges,
	isEditingImage,
	isFindOpen,
	saveFileContent,
	revertChanges,
	shareToMclogs,
	toggleFind,
}

onMounted(() => {
	ui.fileEditorApi.value = fileEditorApi
})

onUnmounted(() => {
	if (ui.fileEditorApi.value === fileEditorApi) {
		ui.fileEditorApi.value = null
	}
})
</script>
