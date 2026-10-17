<template>
	<div class="file-tabs dockview-theme-dark h-full w-full">
		<DockviewVue
			class="h-full w-full"
			:theme="theme"
			:components="components"
			:tab-components="tabComponents"
			:right-header-actions-component="FileTabsActions"
			default-renderer="always"
			disable-floating-groups
			disable-tabs-overflow-list
			no-panels-overlay="emptyGroup"
			@ready="ui.fileTabs.onReady"
		/>
	</div>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'
import '#ui/styles/dockview-theme.css'

import { type DockviewTheme, DockviewVue } from 'dockview-vue'
import { onBeforeUnmount } from 'vue'

import { FILE_TAB_COMPONENT, FILE_TAB_PANEL_COMPONENT } from '../../composables/file-tabs'
import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import FileTab from './FileTab.vue'
import FileTabPanel from './FileTabPanel.vue'
import FileTabsActions from './FileTabsActions.vue'

const ui = injectFileBrowserUI()

const theme: DockviewTheme = {
	name: 'modrinth',
	className: 'dockview-theme-modrinth',
	dndTabIndicator: 'line',
}

const components = { [FILE_TAB_PANEL_COMPONENT]: FileTabPanel }
const tabComponents = { [FILE_TAB_COMPONENT]: FileTab }

onBeforeUnmount(() => ui.fileTabs.onDispose())
</script>
