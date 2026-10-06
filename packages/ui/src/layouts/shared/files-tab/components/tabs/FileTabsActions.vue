<template>
	<div class="flex h-full items-center px-1">
		<button
			v-tooltip="formatMessage(messages.newTab)"
			type="button"
			class="flex size-7 items-center justify-center rounded-md border-none bg-transparent p-0 text-secondary hover:bg-surface-5 hover:text-contrast focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
			:aria-label="formatMessage(messages.newTab)"
			@click="openNewTab"
		>
			<PlusIcon class="size-4" />
		</button>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import { infoFrom } from '../../utils'

defineProps<{
	params?: unknown
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	newTab: {
		id: 'files.tabs.new-tab',
		defaultMessage: 'New tab',
	},
})

const ui = injectFileBrowserUI()

function openNewTab() {
	ui.fileTabs.openTab(infoFrom('/'))
}
</script>
