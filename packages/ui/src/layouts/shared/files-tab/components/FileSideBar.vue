<template>
	<div
		class="flex flex-col p-1 gap-2 h-full"
		:class="[
			addBorder ? 'border-r border-0 border-solid border-surface-5 pr-2' : '',
			constrainWidth ? 'w-[24rem]' : 'w-full'
		]"
	>
		<FileActionBar
			:sidebar-open="ui.sidebarOpen.value"
			:is-editing="ui.isEditing.value"
			:editing-file-path="ctx.editingFile.value?.path"
			:is-editing-image="ui.fileEditorApi.value?.isEditingImage.value ?? false"
			:is-editor-find-open="ui.fileEditorApi.value?.isFindOpen.value ?? false"
			:search-query="ui.searchQuery.value"
			:show-refresh-button="ui.showRefreshButton.value"
			:show-install-from-url="ctx.showInstallFromUrl"
			:disabled="ui.isBusy.value"
			:disabled-tooltip="ui.busyTooltip.value"
			:is-refreshing="ctx.isRefreshing.value"
			@update:search-query="(value) => (ui.searchQuery.value = value)"
			@create="ui.showCreateModal"
			@upload="ui.initiateFileUpload"
			@upload-zip="() => {}"
			@unzip-from-url="ui.showUnzipFromUrlModal"
			@refresh="() => {
				ctx.refresh();
			}"
			@share="() => ui.shareToMclogs()"
			@find="() => ui.toggleFind()"
			@toggle-sidebar="() => ui.setSidebarOpen(!ui.sidebarOpen.value)"
		/>
		<div :class="scrollFileEntries ? 'overflow-y-auto' : ''" :style="scrollFileEntries ? 'height: calc(100dvh - 92px - 32px)' : ''">
			<div
				v-if="filteredItems.length > 0"
				class="relative h-fit"
				:style="{ overflowAnchor: 'none'}"
			>
				<div>
					<FileTableRow
						v-for="(item, idx) in filteredItems"
						:key="item.path"
						:count="item.count"
						:created="item.created"
						:modified="item.modified"
						:name="item.name"
						:path="item.path"
						:type="item.type"
						:size="item.size"
						:index="idx"
						:is-last="idx === filteredItems.length - 1"
						:selected="ui.selectedItems.value.has(item.path)"
						:write-disabled="ui.isBusy.value || !!ctx.isReadOnly?.(item.path)"
						:write-disabled-tooltip="ctx.isReadOnly?.(item.path) ? ctx.readOnlyReason?.value : ui.busyTooltip.value"
						:show-details="false"
						:selection-within-action-menu="true"
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
				class="flex h-full w-full items-center justify-center rounded-b-[20px] bg-surface-2 px-5 py-20"
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
		</div>
	</div>
</template>

<script setup lang="ts">
import { FolderOpenIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/index.ts'
import { injectFileManager } from '#ui/layouts/index.ts'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileActionBar from './FileActionBar.vue'
import FileTableRow from './FileTableRow.vue'

const ctx = injectFileManager()
const ui = injectFileBrowserUI()

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
})

const filteredItems = computed(() => ui.filteredItems.value)

withDefaults(
	defineProps<{
		constrainWidth?: boolean
		scrollFileEntries?: boolean,
		addBorder?: boolean
	}>(),
	{
		constrainWidth: false,
		scrollFileEntries: false,
		addBorder: true,
	}
)

</script>
