<template>
	<header
		class="@container flex select-none flex-col"
		:class="[hasNav ? 'gap-4' : 'gap-3']"
		:aria-label="formatMessage(messages.fileNavigation)"
	>
		<div class="flex items-center justify-between gap-2">
			<div
				:aria-label="formatMessage(messages.breadcrumbNavigation)"
				class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
			>
				<div class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
					<div v-if="hasNav ? (smallMode ? true : !sidebarOpen) : true" class="mr-2 flex flex-shrink-0 gap-3">
						<IconButton
							v-tooltip="formatMessage(sidebarOpen ? messages.collapse : messages.expand)"
							:label="formatMessage(sidebarOpen ? messages.collapse : messages.expand)"
							native-type="button"
							class="bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
							@click="() => $emit('toggleSidebar')"
						>
							<PanelRightCloseIcon v-if="sidebarOpen && !smallMode" transform="rotate(180)" />
							<PanelRightOpenIcon v-else transform="rotate(180)"/>
							<span class="sr-only">{{ formatMessage(messages.expand) }}</span>
						</IconButton>
					</div>
					<div class="ml-2">
						<slot/>
					</div>
				</div>
			</div>

			<div v-if="!isEditing" class="flex flex-shrink-0 items-center gap-2">
				<Input
					v-if="hasNav ? !sidebarOpen : false"
					id="search-folder"
					:model-value="searchQuery"
					:icon="SearchIcon"
					type="search"
					name="search"
					autocomplete="off"
					:placeholder="formatMessage(messages.searchFiles)"
					:class="[
						hasNav ? 'hidden @[800px]:inline-flex' : 'hidden @[400px]:inline-flex'
					]"
					size="medium"
					wrapper-class="w-full sm:w-[280px]"
					@update:model-value="$emit('update:searchQuery', $event)"
				/>

				<TeleportOverflowMenu
					type="outlined"
					:label="formatMessage(messages.createNew)"
					:disabled="disabled"
					:tooltip="disabled ? disabledTooltip : undefined"
					size="lg"
					class="justify-center gap-2 !w-auto !px-2.5 !rounded-xl"
					:options="options"
				>
					<PlusIcon aria-hidden="true" class="h-5 w-5" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5" />
				</TeleportOverflowMenu>
			</div>

			<div v-else-if="!isEditingImage" class="flex gap-2">
				<IconButton
					v-if="isLogFile"
					v-tooltip="formatMessage(messages.shareToMclogs)"
					type="quiet"
					size="lg"
					:label="formatMessage(messages.shareToMclogs)"
					@click="$emit('share')"
				>
					<ShareIcon />
				</IconButton>
				<IconButton
					v-tooltip="formatMessage(messages.findInFile)"
					:type="isEditorFindOpen ? 'colored' : 'quiet'"
					size="lg"
					:color="isEditorFindOpen ? 'brand' : undefined"
					:label="formatMessage(messages.findInFile)"
					:aria-pressed="isEditorFindOpen"
					@click="$emit('find')"
				>
					<SearchIcon />
				</IconButton>
			</div>
		</div>
		<div
v-if="!isEditing && !hasNav" class="flex items-center gap-2"
			 :class="[

			]"
		>
			<Input
				:model-value="searchQuery"
				:icon="SearchIcon"
				type="search"
				name="search"
				autocomplete="off"
				:placeholder="formatMessage(messages.searchFiles)"
				size="medium"
				wrapper-class="flex-1 min-w-0"
				@update:model-value="$emit('update:searchQuery', $event)"
			/>
		</div>
	</header>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	CurseForgeIcon,
	DropdownIcon,
	FileArchiveIcon,
	FolderOpenIcon,
	LinkIcon,
	PanelRightCloseIcon,
	PanelRightOpenIcon,
	PlusIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	UploadIcon,
} from '@modrinth/assets'
import {computed} from 'vue'

import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n.ts'
import {useFileActions} from "#ui/layouts/shared/files-tab/composables/folder-actions.ts";
import { commonMessages } from '#ui/utils/common-messages.ts'

import Input from '../../../../components/base/inputs/Input.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	fileNavigation: {
		id: 'files.navbar.file-navigation',
		defaultMessage: 'File navigation',
	},
	breadcrumbNavigation: {
		id: 'files.navbar.breadcrumb-navigation',
		defaultMessage: 'Breadcrumb navigation',
	},
	backToHome: {
		id: 'files.navbar.back-to-home',
		defaultMessage: 'Back to home',
	},
	home: {
		id: 'files.navbar.home',
		defaultMessage: 'Home',
	},
	expand: {
		id: 'files.navbar.expand',
		defaultMessage: 'Expand',
	},
	collapse: {
		id: 'files.navbar.collapse',
		defaultMessage: 'Collapse',
	},
	searchFiles: {
		id: 'files.navbar.search-files',
		defaultMessage: 'Search files',
	},
	createNew: {
		id: 'files.navbar.create-new',
		defaultMessage: 'Create new...',
	},
	newFile: {
		id: 'files.navbar.new-file',
		defaultMessage: 'New file',
	},
	newFolder: {
		id: 'files.navbar.new-folder',
		defaultMessage: 'New folder',
	},
	uploadFile: {
		id: 'files.navbar.upload-file',
		defaultMessage: 'Upload file',
	},
	uploadFromZip: {
		id: 'files.navbar.upload-from-zip',
		defaultMessage: 'Upload from .zip file',
	},
	uploadFromZipUrl: {
		id: 'files.navbar.upload-from-zip-url',
		defaultMessage: 'Upload from .zip URL',
	},
	installCurseForgePack: {
		id: 'files.navbar.install-curseforge-pack',
		defaultMessage: 'Install CurseForge pack',
	},
	shareToMclogs: {
		id: 'files.navbar.share-to-mclogs',
		defaultMessage: 'Share to mclo.gs',
	},
	findInFile: {
		id: 'files.navbar.find-in-file',
		defaultMessage: 'Find in file',
	},
})

export type Properties = {
	isEditing: boolean
	sidebarOpen: boolean
	editingFilePath?: string
	isEditingImage?: boolean
	isEditorFindOpen?: boolean
	searchQuery: string
	showRefreshButton?: boolean
	showInstallFromUrl?: boolean
	disabled?: boolean
	disabledTooltip?: string
	hasNav?: boolean,
	smallMode?: boolean,
	isRefreshing?: boolean
};

const props = withDefaults(
	defineProps<Properties>(),
	{
		hasNav: false,
		smallMode: false,
	}
)

const smallMode = computed(() => props.smallMode)

export type EmitCallbacks = {
	toggleSidebar: []
	'update:searchQuery': [value: string]
	create: [type: 'file' | 'directory']
	upload: []
	uploadZip: []
	unzipFromUrl: [cf: boolean]
	refresh: [],
	share: []
	find: []
};

const emit = defineEmits<EmitCallbacks>()

const refreshing = computed(() => props.isRefreshing)

function handleRefresh() {
	emit('refresh')
}

const isLogFile = computed(() => {
	return (
		props.editingFilePath?.startsWith('logs') ||
		props.editingFilePath?.startsWith('crash-reports') ||
		props.editingFilePath?.endsWith('.log')
	)
})

const { options } = useFileActions(
	(type) => emit('create', type),
	(type) => type == 'file' ? emit('upload') : emit('uploadZip'),
	props.showInstallFromUrl ? (type) => emit('unzipFromUrl', type == 'cf') : undefined,
	props.showRefreshButton ? { handleRefresh, isRefreshing: computed(() => refreshing.value ?? false) } : undefined,
);
</script>
