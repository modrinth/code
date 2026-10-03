<template>
	<FileEditor
		v-if="location?.type === 'file'"
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

import { currentLocation, type FileTabPanelParams } from '../../composables/file-tabs'
import { type FileEditorBridge, injectFileBrowserUI } from '../../providers/file-browser-ui'
import FileEditor from '../editor/FileEditor.vue'
import type {FileInfo} from "#ui/layouts/shared/files-tab/providers/file-manager.ts";

const props = defineProps<{
	params: {
		params: FileTabPanelParams
		api: DockviewPanelApi
	}
}>()

const ui = injectFileBrowserUI()

const tabId = props.params.params.tabId
const location = computed(() => {
	const tab = ui.fileTabs.getTab(tabId)
	if (tab == null) return null;
	const location = currentLocation(tab);
	return location.type == 'file' ? location as FileInfo<'file'> : null;
})

const fileEditorRef = ref<InstanceType<typeof FileEditor>>()

const bridge: FileEditorBridge = {
	hasUnsavedChanges: computed(() => fileEditorRef.value?.hasUnsavedChanges ?? false),
	isEditingImage: computed(() => fileEditorRef.value?.isEditingImage ?? false),
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
