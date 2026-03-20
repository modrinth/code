<template>
	<header
		class="flex select-none flex-col justify-between gap-2 sm:flex-row sm:items-center"
		:aria-label="formatMessage(messages.fileNavigation)"
	>
		<nav
			:aria-label="formatMessage(messages.breadcrumbNavigation)"
			class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
		>
			<ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
				<li class="mr-4 flex-shrink-0">
					<ButtonStyled circular>
						<button
							v-tooltip="formatMessage(messages.backToHome)"
							type="button"
							class="!size-10 bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
							@click="$emit('navigateHome')"
							@mouseenter="$emit('prefetchHome')"
						>
							<HomeIcon />
							<span class="sr-only">{{ formatMessage(messages.home) }}</span>
						</button>
					</ButtonStyled>
				</li>
				<li class="m-0 -ml-2 min-w-0 flex-shrink p-0">
					<ol class="m-0 flex min-w-0 flex-shrink items-center overflow-hidden p-0">
						<TransitionGroup
							name="breadcrumb"
							tag="span"
							class="relative flex min-w-0 flex-shrink items-center"
						>
							<li
								v-for="(segment, index) in breadcrumbs"
								:key="`${segment || index}-group`"
								class="relative flex min-w-0 flex-shrink items-center text-sm"
							>
								<div class="flex min-w-0 flex-shrink items-center">
									<ButtonStyled type="transparent">
										<button
											class="cursor-pointer truncate focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
											:aria-current="
												!isEditing && index === breadcrumbs.length - 1 ? 'location' : undefined
											"
											:class="{
												'!text-contrast': !isEditing && index === breadcrumbs.length - 1,
											}"
											@click="$emit('navigate', index)"
										>
											{{ segment || '' }}
										</button>
									</ButtonStyled>
									<ChevronRightIcon
										v-if="index < breadcrumbs.length - 1 || isEditing"
										class="size-4 flex-shrink-0 text-secondary"
										aria-hidden="true"
									/>
								</div>
							</li>
						</TransitionGroup>
						<li v-if="isEditing && editingFileName" class="flex items-center px-3 text-base">
							<span class="font-semibold !text-contrast" aria-current="location">
								{{ editingFileName }}
							</span>
						</li>
					</ol>
				</li>
			</ol>
		</nav>

		<div v-if="!isEditing" class="flex flex-shrink-0 items-center gap-2">
			<StyledInput
				id="search-folder"
				:model-value="searchQuery"
				:icon="SearchIcon"
				type="search"
				name="search"
				autocomplete="off"
				:placeholder="formatMessage(messages.searchFiles)"
				class="!h-10"
				input-class="!h-10"
				wrapper-class="w-full sm:w-[280px]"
				@update:model-value="$emit('update:searchQuery', $event)"
			/>

			<ButtonStyled v-if="showRefreshButton" type="outlined">
				<button
					type="button"
					class="flex !h-10 items-center gap-2 !border-[1px] !border-surface-5"
					@click="$emit('refresh')"
				>
					<RefreshCwIcon aria-hidden="true" class="h-5 w-5" />
					{{ formatMessage(commonMessages.refreshButton) }}
				</button>
			</ButtonStyled>

			<ButtonStyled type="outlined">
				<OverflowMenu
					:dropdown-id="`create-new-${baseId}`"
					position="bottom"
					direction="left"
					:aria-label="formatMessage(messages.createNew)"
					:disabled="disabled"
					:tooltip="disabled ? disabledTooltip : undefined"
					class="!h-10 justify-center gap-2 !border-[1px] !border-surface-5"
					:options="[
						{ id: 'file', action: () => $emit('create', 'file') },
						{ id: 'directory', action: () => $emit('create', 'directory') },
						{ id: 'upload', action: () => $emit('upload') },
						{ divider: true, shown: showInstallFromUrl ?? false },
						{ id: 'upload-zip', shown: false, action: () => $emit('uploadZip') },
						{
							id: 'install-from-url',
							shown: showInstallFromUrl ?? false,
							action: () => $emit('unzipFromUrl', false),
						},
						{
							id: 'install-cf-pack',
							shown: showInstallFromUrl ?? false,
							action: () => $emit('unzipFromUrl', true),
						},
					]"
				>
					<PlusIcon aria-hidden="true" class="h-5 w-5" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5" />
					<template #file>
						<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.newFile) }}
					</template>
					<template #directory>
						<FolderOpenIcon aria-hidden="true" /> {{ formatMessage(messages.newFolder) }}
					</template>
					<template #upload>
						<UploadIcon aria-hidden="true" /> {{ formatMessage(messages.uploadFile) }}
					</template>
					<template #upload-zip>
						<FileArchiveIcon aria-hidden="true" /> {{ formatMessage(messages.uploadFromZip) }}
					</template>
					<template #install-from-url>
						<LinkIcon aria-hidden="true" /> {{ formatMessage(messages.uploadFromZipUrl) }}
					</template>
					<template #install-cf-pack>
						<CurseForgeIcon aria-hidden="true" />
						{{ formatMessage(messages.installCurseForgePack) }}
					</template>
				</OverflowMenu>
			</ButtonStyled>
		</div>

		<div v-else-if="!isEditingImage" class="flex gap-2">
			<Button
				v-if="isLogFile"
				v-tooltip="formatMessage(messages.shareToMclogs)"
				icon-only
				transparent
				:aria-label="formatMessage(messages.shareToMclogs)"
				@click="$emit('share')"
			>
				<ShareIcon />
			</Button>
			<ButtonStyled type="transparent">
				<TeleportOverflowMenu
					:aria-label="formatMessage(messages.saveFile)"
					:options="[
						{ id: 'save', action: () => $emit('save') },
						{ id: 'save-as', action: () => $emit('saveAs') },
						{ id: 'save-restart', action: () => $emit('saveRestart') },
					]"
				>
					<SaveIcon aria-hidden="true" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
					<template #save>
						<SaveIcon aria-hidden="true" /> {{ formatMessage(messages.save) }}
					</template>
					<template #save-as>
						<SaveIcon aria-hidden="true" /> {{ formatMessage(messages.saveAs) }}
					</template>
					<template #save-restart>
						<RefreshCwIcon aria-hidden="true" />
						{{ formatMessage(messages.saveAndRestart) }}
					</template>
				</TeleportOverflowMenu>
			</ButtonStyled>
		</div>
	</header>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	ChevronRightIcon,
	CurseForgeIcon,
	DropdownIcon,
	FileArchiveIcon,
	FolderOpenIcon,
	HomeIcon,
	LinkIcon,
	PlusIcon,
	RefreshCwIcon,
	SaveIcon,
	SearchIcon,
	ShareIcon,
	UploadIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import Button from '#ui/components/base/Button.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import OverflowMenu from '#ui/components/base/OverflowMenu.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import TeleportOverflowMenu from '#ui/components/base/TeleportOverflowMenu.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

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
	saveFile: {
		id: 'files.navbar.save-file',
		defaultMessage: 'Save file',
	},
	save: {
		id: 'files.navbar.save',
		defaultMessage: 'Save',
	},
	saveAs: {
		id: 'files.navbar.save-as',
		defaultMessage: 'Save as...',
	},
	saveAndRestart: {
		id: 'files.navbar.save-and-restart',
		defaultMessage: 'Save & restart',
	},
})

const props = defineProps<{
	breadcrumbs: string[]
	isEditing: boolean
	editingFileName?: string
	editingFilePath?: string
	isEditingImage?: boolean
	searchQuery: string
	showRefreshButton?: boolean
	showInstallFromUrl?: boolean
	baseId: string
	disabled?: boolean
	disabledTooltip?: string
}>()

defineEmits<{
	navigate: [index: number]
	navigateHome: []
	prefetchHome: []
	'update:searchQuery': [value: string]
	create: [type: 'file' | 'directory']
	upload: []
	uploadZip: []
	unzipFromUrl: [cf: boolean]
	refresh: []
	save: []
	saveAs: []
	saveRestart: []
	share: []
}>()

const isLogFile = computed(() => {
	return (
		props.editingFilePath?.startsWith('logs') ||
		props.editingFilePath?.startsWith('crash-reports') ||
		props.editingFilePath?.endsWith('.log')
	)
})
</script>

<style scoped>
.breadcrumb-move,
.breadcrumb-enter-active,
.breadcrumb-leave-active {
	transition: all 0.2s ease;
}

.breadcrumb-enter-from {
	opacity: 0;
	transform: translateX(-10px) scale(0.9);
}

.breadcrumb-leave-to {
	opacity: 0;
	transform: translateX(-10px) scale(0.8);
	filter: blur(4px);
}

.breadcrumb-leave-active {
	position: relative;
	pointer-events: none;
}

.breadcrumb-move {
	z-index: 1;
}
</style>
