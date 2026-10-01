<template>
	<div
		class="group flex h-full max-w-[16rem] items-center gap-1.5 pl-3 text-sm"
		:class="canClose ? 'pr-1' : 'pr-3'"
		:title="location?.path"
		@auxclick.prevent="(event) => event.button === 1 && close()"
	>
		<component :is="icon" class="size-4 shrink-0 text-secondary" />
		<span class="truncate" :class="isActive ? 'font-semibold text-contrast' : 'text-secondary'">
			{{ label }}
		</span>
		<button
			v-if="canClose"
			type="button"
			class="relative flex size-6 shrink-0 items-center justify-center rounded-md border-none bg-transparent p-0 text-secondary hover:bg-surface-5 hover:text-contrast focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
			:aria-label="formatMessage(messages.closeTab, { name: label })"
			@pointerdown.stop
			@mousedown.stop
			@click.stop="close"
		>
			<span
				v-if="isDirty"
				v-tooltip="formatMessage(messages.unsavedChanges)"
				class="size-2 rounded-full bg-brand group-hover:hidden"
			/>
			<XIcon class="size-4" :class="{ 'hidden group-hover:block': isDirty }" />
		</button>
		<span
			v-else-if="isDirty"
			v-tooltip="formatMessage(messages.unsavedChanges)"
			class="size-2 shrink-0 rounded-full bg-brand"
		/>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, FolderOpenIcon, XIcon } from '@modrinth/assets'
import type { DockviewPanelApi } from 'dockview-vue'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { currentLocation, type FileTabPanelParams } from '../../composables/file-tabs'
import { injectFileBrowserUI } from '../../providers/file-browser-ui'

const props = defineProps<{
	params: {
		params: FileTabPanelParams
		api: DockviewPanelApi
	}
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	closeTab: {
		id: 'files.tabs.close-tab',
		defaultMessage: 'Close {name}',
	},
	unsavedChanges: {
		id: 'files.tabs.unsaved-changes',
		defaultMessage: 'Unsaved changes',
	},
	home: {
		id: 'files.tabs.home',
		defaultMessage: 'Home',
	},
})

const ui = injectFileBrowserUI()

const tabId = props.params.params.tabId
const location = computed(() => {
	const tab = ui.fileTabs.getTab(tabId)
	return tab ? currentLocation(tab) : null
})

const label = computed(() => {
	const path = location.value?.path ?? '/'
	return path.split('/').filter(Boolean).pop() ?? formatMessage(messages.home)
})
const icon = computed(() => (location.value?.kind === 'file' ? FileIcon : FolderOpenIcon))
const isActive = computed(() => ui.fileTabs.activeTabId.value === tabId)
const canClose = computed(() => ui.fileTabs.tabs.value.length > 1)
const isDirty = computed(() => ui.fileTabs.editors.get(tabId)?.hasUnsavedChanges.value ?? false)

function close() {
	ui.fileTabs.closeTab(tabId)
}
</script>
