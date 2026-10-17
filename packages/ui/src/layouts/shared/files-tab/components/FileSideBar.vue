<template>
	<div
		class="flex flex-col p-1 gap-2 h-full"
		:class="[
			addBorder ? 'border-r border-0 border-solid border-surface-5 pr-2' : '',
			constrainWidth ? 'w-[24rem]' : 'w-full',
		]"
	>
		<FileActionBar
			:sidebar-open="ui.sidebarOpen.value"
			:active-location="ui.activeLocation.value"
			:search-query="ui.activeView.value.searchQuery.value"
			:show-refresh-button="ui.showRefreshButton.value"
			:show-install-from-url="ui.showInstallFromUrl"
			:disabled="ui.isBusy.value"
			:disabled-tooltip="ui.busyTooltip.value"
			:is-refreshing="ui.isRefreshing.value"
			@update:search-query="(value) => (ui.activeView.value.searchQuery.value = value)"
			@create="ui.showCreateModal"
			@upload="ui.initiateFileUpload"
			@upload-zip="() => {}"
			@unzip-from-url="ui.showUnzipFromUrlModal"
			@refresh="() => ui.refresh()"
			@share="() => ui.shareEditorToMclogs()"
			@find="() => ui.toggleFind()"
			@toggle-sidebar="() => ui.setSidebarOpen(!ui.sidebarOpen.value)"
		>
			<template #actions>
				<IconButton
					v-tooltip="formatMessage(messages.compactFolders)"
					:type="ui.compactFolders.value ? 'colored' : 'quiet'"
					:color="ui.compactFolders.value ? 'brand' : undefined"
					size="sm"
					:label="formatMessage(messages.compactFolders)"
					:aria-pressed="ui.compactFolders.value"
					@click="ui.setCompactFolders(!ui.compactFolders.value)"
				>
					<FoldVerticalIcon />
				</IconButton>
			</template>
		</FileActionBar>
		<FileSearch :loaded-only-notice="ui.searchLoadedOnlyNotice.value" />
		<div
			:class="[
				scrollFileEntries ? 'overflow-y-auto' : '',
				fillHeight ? 'min-h-0 flex-1 overflow-y-auto overscroll-contain' : '',
			]"
			:style="scrollFileEntries ? 'height: calc(100dvh - 92px - 32px)' : ''"
		>
			<FileTree />
		</div>
	</div>
</template>

<script setup lang="ts">
import { FoldVerticalIcon } from '@modrinth/assets'

import { IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileActionBar from './FileActionBar.vue'
import FileSearch from './FileSearch.vue'
import FileTree from './FileTree.vue'

const ui = injectFileBrowserUI()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	compactFolders: {
		id: 'files.sidebar.compact-folders',
		defaultMessage: 'Compact folders',
	},
})

withDefaults(
	defineProps<{
		constrainWidth?: boolean
		scrollFileEntries?: boolean
		addBorder?: boolean
		/** Fill the parent's height and scroll the entries within it, keeping the action bar pinned. */
		fillHeight?: boolean
	}>(),
	{
		constrainWidth: false,
		scrollFileEntries: false,
		addBorder: true,
		fillHeight: false,
	},
)
</script>
