<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap gap-1" role="tablist" :aria-label="formatMessage(messages.pages)">
			<Button
				v-for="page in pages"
				:key="page.id"
				type="quiet"
				size="sm"
				role="tab"
				:aria-selected="page.id === activePage.id"
				:class="{ '!bg-surface-4 !text-contrast': page.id === activePage.id }"
				@click="pageId = page.id"
			>
				<component :is="page.icon" aria-hidden="true" />
				{{ formatMessage(page.name) }}
			</Button>
		</div>
		<SettingsPanel :key="activePage.id" :groups="activePage.groups" />
	</div>
</template>

<script setup lang="ts">
import { InfoIcon, LayoutPanelLeftIcon } from '@modrinth/assets'
import { useLocalStorage } from '@vueuse/core'
import { computed } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { SettingsPanel } from '#ui/components/settings'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { filesTabSettingGroups, useFilesTabSettings } from '#ui/layouts/shared/files-tab/settings'
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
const { formatMessage } = useVIntl()

const archiveStore = injectArchiveSettings()
const workbench = injectWorkbench()
const filesStore = useFilesTabSettings()

const messages = defineMessages({
	pages: { id: 'archive-viewer.settings.pages', defaultMessage: 'Settings pages' },
	details: { id: 'archive-viewer.settings.page.details', defaultMessage: 'Details' },
	layout: { id: 'archive-viewer.settings.page.layout', defaultMessage: 'Layout and files' },
})

const pages = computed(() => [
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
			...filesTabSettingGroups.map((group) => ({ ...group, store: filesStore })),
		],
	},
])

const pageId = useLocalStorage('archive-viewer-settings:page', 'details')
const activePage = computed(
	() => pages.value.find((page) => page.id === pageId.value) ?? pages.value[0],
)
</script>
