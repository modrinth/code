<template>
	<div class="flex h-full items-center gap-1 pr-3">
		<TeleportOverflowMenu
			v-if="ui.fileTabs.tabs.value.length > 1"
			v-tooltip="formatMessage(messages.allTabs)"
			type="quiet"
			:label="formatMessage(messages.allTabs)"
			:options="tabOptions"
		>
			<ChevronDownIcon class="size-5" aria-hidden="true" />
		</TeleportOverflowMenu>
		<div class="flex min-w-[51px] shrink-0 justify-end">
			<IconButton
				v-tooltip="formatMessage(messages.newTab)"
				type="quiet"
				:label="formatMessage(messages.newTab)"
				@click="openNewTab"
			>
				<PlusIcon />
			</IconButton>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, HomeIcon, PlusIcon } from '@modrinth/assets'
import { computed } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { currentLocation } from '../../composables/file-tabs'
import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import { fileIconFor, infoFrom } from '../../utils'

defineProps<{
	params?: unknown
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	newTab: {
		id: 'files.tabs.new-tab',
		defaultMessage: 'New tab',
	},
	allTabs: {
		id: 'files.tabs.all-tabs',
		defaultMessage: 'All tabs',
	},
	home: {
		id: 'files.tabs.home',
		defaultMessage: 'Home',
	},
})

const ui = injectFileBrowserUI()

/**
 * Every open tab, so tabs scrolled out of the strip stay reachable. Teleported to the body,
 * unlike dockview's own overflow list, so it isn't clipped by the files viewer.
 */
const tabOptions = computed<ButtonMenuOption[]>(() =>
	ui.fileTabs.tabs.value.map((tab) => {
		const location = currentLocation(tab)
		const isHome = location.path === '/'
		return {
			id: tab.id,
			label: isHome ? formatMessage(messages.home) : location.name,
			icon: isHome ? HomeIcon : fileIconFor(location).icon,
			selected: tab.id === ui.fileTabs.activeTabId.value,
			action: () => ui.fileTabs.activateTab(tab.id),
		}
	}),
)

function openNewTab() {
	ui.fileTabs.openTab(infoFrom('/'))
}
</script>
