<template>
	<div
		v-if="ui.inWorkbench"
		class="flex h-full min-h-0 flex-col bg-surface-1"
		@pointerdown.capture="activate"
		@dragenter.capture="activate"
	>
		<div
			class="shrink-0 border-0 border-solid border-surface-5 px-2"
			:class="rounded ? 'border-y bg-surface-1 pt-2 pb-2' : 'border-b bg-surface-2 pt-1 pb-1'"
		>
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
				:sidebar-toggleable="toggleLocation != null"
				:sidebar-toggle-always="toggleLocation === 'navigation'"
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
			:variant="variant"
			:interpretation="interpretation"
			:tab-id="tabId"
			fill-height
			@close="() => ui.fileTabs.closeFile(tabId)"
		/>
		<div
			v-else
			class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-1 [--files-table-header-top:0px]"
		>
			<FileTree
				v-if="ui.browsing.value === 'tree' && !view.searchQuery.value"
				:root="view.location.value.path"
			/>
			<FileListing v-else :view="view" @navigate="navigate" />
		</div>
	</div>
	<FileEditor
		v-else-if="location"
		:key="location.path"
		ref="fileEditorRef"
		:file="location"
		:variant="variant"
		:interpretation="interpretation"
		:tab-id="tabId"
		fill-height
		@close="() => ui.fileTabs.closeFile(tabId)"
	/>
</template>

<script setup lang="ts">
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import {
	type FileInfo,
	RAW_CONTENT_VARIANT,
} from '#ui/layouts/shared/files-tab/providers/file-manager.ts'
import { injectWorkbench } from '#ui/layouts/shared/workbench/providers/workbench'

import { useEntryMenu } from '../../composables/entry-menu'
import { currentLocation, type FileTabPanelParams } from '../../composables/file-tabs'
import { useSidebarToggleLocation } from '../../composables/sidebar-toggle'
import { type FileEditorBridge, injectFileBrowserUI } from '../../providers/file-browser-ui'
import { infoFrom, parentInfoFrom } from '../../utils'
import FileEditor from '../editor/FileEditor.vue'
import FileListing from '../FileListing.vue'
import FileNavbar from '../FileNavbar.vue'
import FileTree from '../FileTree.vue'

const props = defineProps<{
	params: {
		params: FileTabPanelParams
		api: DockviewPanelApi
	}
}>()

const HOME = infoFrom('/')

const ui = injectFileBrowserUI()
const toggleLocation = useSidebarToggleLocation()
const workbench = injectWorkbench(null)

/** Rounded workbenches sit their tab bars on the base surface, which the navigation then continues. */
const rounded = computed(() => workbench?.variant.value === 'rounded')
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

const interpretation = computed(() =>
	location.value ? ui.interpretationOf(tabId, location.value) : 'auto',
)

/** Other interpretations show the stored bytes; the default one shows the picked variant, if any. */
const variant = computed(() => {
	if (!location.value) return null
	if (interpretation.value !== 'auto') return RAW_CONTENT_VARIANT
	return ui.contentVariantOf(tabId, location.value)?.selected ?? null
})

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
