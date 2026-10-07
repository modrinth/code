<template>
	<div class="flex flex-col items-center gap-6 bg-surface-2 px-6 py-16 text-center">
		<div class="flex flex-col items-center gap-2">
			<FileIcon class="size-12 text-secondary" aria-hidden="true" />
			<h3 class="m-0 text-xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h3>
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.description) }}</p>
		</div>
		<div class="flex flex-wrap justify-center gap-2">
			<Button type="colored" color="brand" @click="openHome">
				<HomeIcon aria-hidden="true" />
				{{ formatMessage(messages.openHome) }}
			</Button>
			<Button :disabled="ui.isBusy.value" @click="ui.showCreateModal('file')">
				<FilePlusIcon aria-hidden="true" />
				{{ formatMessage(messages.newFile) }}
			</Button>
			<Button :disabled="ui.isBusy.value" @click="ui.showCreateModal('directory')">
				<FolderIcon aria-hidden="true" />
				{{ formatMessage(messages.newFolder) }}
			</Button>
			<Button :disabled="ui.isBusy.value" @click="ui.initiateFileUpload()">
				<UploadIcon aria-hidden="true" />
				{{ formatMessage(messages.upload) }}
			</Button>
		</div>
		<div v-if="recentlyClosed.length > 0" class="flex w-full max-w-md flex-col gap-1 text-left">
			<h4 class="m-0 mb-1 text-sm font-semibold text-secondary">
				{{ formatMessage(messages.recentlyClosed) }}
			</h4>
			<button
				v-for="entry in recentlyClosed"
				:key="entry.tab.id"
				type="button"
				class="flex items-center gap-2 rounded-lg border-none bg-transparent px-2 py-1.5 text-left text-primary hover:bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
				@click="ui.fileTabs.reopenTab(entry.tab)"
			>
				<component
					:is="entry.icon.icon"
					class="size-4 shrink-0"
					:class="ui.coloredIcons.value ? entry.icon.color : 'text-secondary'"
					aria-hidden="true"
				/>
				<span class="truncate font-medium">{{ entry.label }}</span>
				<span class="ml-auto truncate text-xs text-secondary">{{ entry.location.path }}</span>
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, FilePlusIcon, FolderIcon, HomeIcon, UploadIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { currentLocation } from '../composables/file-tabs'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import { fileIconFor, infoFrom } from '../utils'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'files.empty-workspace.title',
		defaultMessage: 'No open tabs',
	},
	description: {
		id: 'files.empty-workspace.description',
		defaultMessage: 'Open a folder or file to get started.',
	},
	openHome: {
		id: 'files.empty-workspace.open-home',
		defaultMessage: 'Open home folder',
	},
	newFile: {
		id: 'files.navbar.new-file',
		defaultMessage: 'New file',
	},
	newFolder: {
		id: 'files.navbar.new-folder',
		defaultMessage: 'New folder',
	},
	upload: {
		id: 'files.navbar.upload-file',
		defaultMessage: 'Upload file',
	},
	recentlyClosed: {
		id: 'files.empty-workspace.recently-closed',
		defaultMessage: 'Recently closed',
	},
	home: {
		id: 'files.tabs.home',
		defaultMessage: 'Home',
	},
})

const ui = injectFileBrowserUI()

const recentlyClosed = computed(() =>
	ui.fileTabs.recentlyClosed.value.map((tab) => {
		const location = currentLocation(tab)
		const isHome = location.path === '/'
		return {
			tab,
			location,
			label: isHome ? formatMessage(messages.home) : location.name,
			icon: isHome ? { icon: HomeIcon, color: 'text-secondary' } : fileIconFor(location),
		}
	}),
)

function openHome() {
	ui.fileTabs.openTab(infoFrom('/'))
}
</script>
