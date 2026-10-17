<template>
	<div
		v-if="ui.inWorkbench"
		class="flex h-full min-h-0 flex-col bg-surface-1"
		@pointerdown.capture="activate"
		@dragenter.capture="activate"
	>
		<div class="shrink-0 border-0 border-b border-solid border-surface-5 bg-surface-2 px-2 py-1">
			<FileNavbar
				:sidebar-open="ui.sidebarOpen.value"
				:active-location="view.location.value"
				:is-editor-find-open="fileEditorRef?.isFindOpen ?? false"
				:search-query="view.searchQuery.value"
				:show-refresh-button="ui.showRefreshButton.value"
				:show-install-from-url="ui.showInstallFromUrl"
				:base-id="`${ui.baseId}-${tabId}`"
				:disabled="ui.isBusy.value"
				:disabled-tooltip="ui.busyTooltip.value"
				:can-go-back="ui.fileTabs.canGoBackIn(tabId)"
				:can-go-forward="ui.fileTabs.canGoForwardIn(tabId)"
				:sidebar-toggleable="true"
				:compact="ui.compactNavigation.value"
				@back="ui.fileTabs.goIn(tabId, -1)"
				@forward="ui.fileTabs.goIn(tabId, 1)"
				@navigate="(depth) => navigate(parentInfoFrom(view.location.value, depth))"
				@navigate-home="navigate(HOME)"
				@prefetch-home="ui.handleHomePrefetch"
				@update:search-query="(value) => (view.searchQuery.value = value)"
				@create="ui.showCreateModal"
				@upload="ui.initiateFileUpload"
				@upload-zip="() => {}"
				@unzip-from-url="ui.showUnzipFromUrlModal"
				@refresh="ui.refresh"
				@share="fileEditorRef?.shareToMclogs()"
				@find="fileEditorRef?.toggleFind()"
				@toggle-sidebar="ui.setSidebarOpen(!ui.sidebarOpen.value)"
				@contextmenu="openBreadcrumbMenu"
			/>
		</div>
		<FileEditor
			v-if="location"
			:key="location.path"
			ref="fileEditorRef"
			class="min-h-0 flex-1"
			:file="location"
			:editor-component="ui.editorComponent.value"
			fill-height
			@close="() => ui.fileTabs.closeFile(tabId)"
		/>
		<div
			v-else
			class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-1 [--files-table-header-top:0px]"
		>
			<FileListing :view="view" @navigate="navigate" />
		</div>
	</div>
	<FileEditor
		v-else-if="location"
		:key="location.path"
		ref="fileEditorRef"
		:file="location"
		:editor-component="ui.editorComponent.value"
		fill-height
		@close="() => ui.fileTabs.closeFile(tabId)"
	/>
</template>

<script setup lang="ts">
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import type { FileInfo } from '#ui/layouts/shared/files-tab/providers/file-manager.ts'

import { currentLocation, type FileTabPanelParams } from '../../composables/file-tabs'
import { useEntryMenu } from '../../composables/entry-menu'
import { type FileEditorBridge, injectFileBrowserUI } from '../../providers/file-browser-ui'
import { infoFrom, parentInfoFrom } from '../../utils'
import FileEditor from '../editor/FileEditor.vue'
import FileListing from '../FileListing.vue'
import FileNavbar from '../FileNavbar.vue'

const props = defineProps<{
	params: {
		params: FileTabPanelParams
		api: DockviewPanelApi
	}
}>()

const HOME = infoFrom('/')

const ui = injectFileBrowserUI()
const { openMenu } = useEntryMenu()

const tabId = props.params.params.tabId
const view = ui.viewFor(tabId)

/** The open file, when the tab is on one rather than a directory. */
const location = computed(() => {
	const tab = ui.fileTabs.getTab(tabId)
	if (tab == null) return null
	const location = currentLocation(tab)
	return location.type == 'file' ? (location as FileInfo<'file'>) : null
})

const fileEditorRef = ref<InstanceType<typeof FileEditor>>()

/** Operations (creating, uploading, ...) act on the active tab, so working in a tab focuses it. */
function activate() {
	if (ui.fileTabs.activeTabId.value !== tabId) ui.fileTabs.activateTab(tabId)
}

function navigate(file: FileInfo) {
	ui.fileTabs.navigateIn(tabId, file)
}

/** Right-click on a breadcrumb, `depth` segments deep (the last one is the tab's location). */
function openBreadcrumbMenu(event: MouseEvent, depth: number) {
	const current = view.location.value
	const segments = current.path.split('/').filter(Boolean)
	openMenu(event, depth >= segments.length ? current : parentInfoFrom(current, depth))
}

const bridge: FileEditorBridge = {
	hasUnsavedChanges: computed(() => fileEditorRef.value?.hasUnsavedChanges ?? false),
	isFindOpen: computed(() => fileEditorRef.value?.isFindOpen ?? false),
	saveFileContent: async (exit = false) => {
		await fileEditorRef.value?.saveFileContent(exit)
	},
	revertChanges: () => fileEditorRef.value?.revertChanges(),
	shareToMclogs: async () => {
		await fileEditorRef.value?.shareToMclogs()
	},
	toggleFind: () => fileEditorRef.value?.toggleFind(),
}

onMounted(() => ui.fileTabs.registerEditor(tabId, bridge))
onUnmounted(() => ui.fileTabs.unregisterEditor(tabId, bridge))
</script>
