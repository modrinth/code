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
			@ready="ui.fileTabs.onReady"
		/>
	</div>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'

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

<style>
.dockview-theme-modrinth {
	--dv-group-view-background-color: var(--surface-2);
	--dv-tabs-and-actions-container-background-color: var(--surface-3);
	--dv-tabs-and-actions-container-height: 2.5rem;
	--dv-tabs-and-actions-container-font-size: 0.875rem;
	--dv-activegroup-visiblepanel-tab-background-color: var(--surface-2);
	--dv-activegroup-hiddenpanel-tab-background-color: var(--surface-3);
	--dv-inactivegroup-visiblepanel-tab-background-color: var(--surface-2);
	--dv-inactivegroup-hiddenpanel-tab-background-color: var(--surface-3);
	--dv-activegroup-visiblepanel-tab-color: var(--color-contrast);
	--dv-activegroup-hiddenpanel-tab-color: var(--color-text-default);
	--dv-inactivegroup-visiblepanel-tab-color: var(--color-text-default);
	--dv-inactivegroup-hiddenpanel-tab-color: var(--color-text-tertiary);
	--dv-tab-divider-color: var(--surface-5);
	--dv-separator-border: var(--surface-5);
	--dv-paneview-header-border-color: var(--surface-5);
	--dv-icon-hover-background-color: var(--surface-5);
	--dv-drag-over-background-color: var(--color-brand-highlight);
	--dv-active-sash-color: var(--color-brand);
	--dv-tabs-container-scrollbar-color: var(--surface-5);
}

.dv-tab {
	border-width: 0px;
	border-right-width: 1px;
	border-style: solid;
	border-color: var(--surface-5);
}
</style>
