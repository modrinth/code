<template>
	<header
		class="flex select-none flex-col justify-between gap-2 sm:flex-row sm:items-center"
		aria-label="File navigation"
	>
		<nav
			aria-label="Breadcrumb navigation"
			class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
		>
			<ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
				<li class="mr-4 flex-shrink-0">
					<ButtonStyled circular>
						<button
							v-tooltip="'Back to home'"
							type="button"
							class="!size-10 bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
							@click="$emit('navigateHome')"
							@mouseenter="$emit('prefetchHome')"
						>
							<HomeIcon />
							<span class="sr-only">Home</span>
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
						<li v-if="isEditing && editingFileName" class="flex items-center px-3 text-sm">
							<span class="font-semibold !text-contrast" aria-current="location">
								{{ editingFileName }}
							</span>
						</li>
					</ol>
				</li>
			</ol>
		</nav>

		<div v-if="!isEditing" class="flex flex-shrink-0 items-center gap-2">
			<div class="iconified-input w-full sm:w-[280px]">
				<SearchIcon aria-hidden="true" class="!text-secondary" />
				<input
					id="search-folder"
					:value="searchQuery"
					type="search"
					name="search"
					autocomplete="off"
					class="h-10 w-full rounded-[14px] border-0 bg-surface-4 text-sm"
					placeholder="Search files"
					@input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
				/>
			</div>

			<ButtonStyled type="outlined">
				<OverflowMenu
					:dropdown-id="`create-new-${baseId}`"
					position="bottom"
					direction="left"
					aria-label="Create new..."
					class="!h-10 justify-center gap-2 !border-[1px] !border-surface-5"
					:options="[
						{ id: 'file', action: () => $emit('create', 'file') },
						{ id: 'directory', action: () => $emit('create', 'directory') },
						{ id: 'upload', action: () => $emit('upload') },
						{ divider: true },
						{ id: 'upload-zip', shown: false, action: () => $emit('uploadZip') },
						{ id: 'install-from-url', action: () => $emit('unzipFromUrl', false) },
						{ id: 'install-cf-pack', action: () => $emit('unzipFromUrl', true) },
					]"
				>
					<PlusIcon aria-hidden="true" class="h-5 w-5" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5" />
					<template #file> <BoxIcon aria-hidden="true" /> New file </template>
					<template #directory> <FolderOpenIcon aria-hidden="true" /> New folder </template>
					<template #upload> <UploadIcon aria-hidden="true" /> Upload file </template>
					<template #upload-zip>
						<FileArchiveIcon aria-hidden="true" /> Upload from .zip file
					</template>
					<template #install-from-url>
						<LinkIcon aria-hidden="true" /> Upload from .zip URL
					</template>
					<template #install-cf-pack>
						<CurseForgeIcon aria-hidden="true" /> Install CurseForge pack
					</template>
				</OverflowMenu>
			</ButtonStyled>
		</div>

		<div v-else-if="!isEditingImage" class="flex gap-2">
			<Button
				v-if="isLogFile"
				v-tooltip="'Share to mclo.gs'"
				icon-only
				transparent
				aria-label="Share to mclo.gs"
				@click="$emit('share')"
			>
				<ShareIcon />
			</Button>
			<ButtonStyled type="transparent">
				<TeleportOverflowMenu
					aria-label="Save file"
					:options="[
						{ id: 'save', action: () => $emit('save') },
						{ id: 'save-as', action: () => $emit('saveAs') },
						{ id: 'save-restart', action: () => $emit('saveRestart') },
					]"
				>
					<SaveIcon aria-hidden="true" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
					<template #save> <SaveIcon aria-hidden="true" /> Save </template>
					<template #save-as> <SaveIcon aria-hidden="true" /> Save as... </template>
					<template #save-restart>
						<RefreshCwIcon aria-hidden="true" />
						Save & restart
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
import { Button, ButtonStyled, OverflowMenu } from '@modrinth/ui'
import { computed } from 'vue'

import TeleportOverflowMenu from './explorer/TeleportOverflowMenu.vue'

const props = defineProps<{
	breadcrumbs: string[]
	isEditing: boolean
	editingFileName?: string
	editingFilePath?: string
	isEditingImage?: boolean
	searchQuery: string
	baseId: string
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
