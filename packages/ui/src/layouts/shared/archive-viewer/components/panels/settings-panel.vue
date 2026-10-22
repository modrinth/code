<template>
	<SettingsPages :pages="pages" storage-key="archive-viewer-settings:page" />
</template>

<script setup lang="ts">
import { InfoIcon, LayoutPanelLeftIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { type SettingsPage, SettingsPages } from '#ui/components/settings'
import { defineMessages } from '#ui/composables/i18n'
import {
	filesTabSettingGroups,
	injectFilesWorkspaceSettings,
	useFilesTabSettings,
} from '#ui/layouts/shared/files-tab/settings'
import { injectWorkbench } from '#ui/layouts/shared/workbench/providers/workbench'
import {
	workbenchSettingGroup,
	workbenchSettingsStore,
} from '#ui/layouts/shared/workbench/settings'

import { archiveSettingGroups, injectArchiveSettings } from '../../settings'

/**
 * The archive viewer's settings as a panel of its own, so they can be changed while watching them
 * apply, docked anywhere or popped out like any other panel.
 */
const archiveStore = injectArchiveSettings()
const workbench = injectWorkbench()
/** The files settings as the place these files are browsed sees them, when it's known. */
const filesWorkspace = injectFilesWorkspaceSettings(null)
const filesStore = filesWorkspace?.store ?? useFilesTabSettings()
const filesGroups = filesWorkspace?.groups ?? filesTabSettingGroups

const messages = defineMessages({
	details: { id: 'archive-viewer.settings.page.details', defaultMessage: 'Details' },
	layout: { id: 'archive-viewer.settings.page.layout', defaultMessage: 'Layout and files' },
})

const pages = computed<SettingsPage[]>(() => [
	{
		id: 'details',
		name: messages.details,
		icon: InfoIcon,
		groups: archiveSettingGroups.map((group) => ({ ...group, store: archiveStore })),
	},
	{
		id: 'layout',
		name: messages.layout,
		icon: LayoutPanelLeftIcon,
		groups: [
			{ ...workbenchSettingGroup(workbench), store: workbenchSettingsStore(workbench) },
			...filesGroups.map((group) => ({ ...group, store: filesStore })),
		],
	},
])
</script>
