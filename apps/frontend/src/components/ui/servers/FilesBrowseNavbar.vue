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
							@click="$emit('navigate', -1)"
							@mouseenter="$emit('prefetch-home')"
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
								v-for="(segment, index) in breadcrumbSegments"
								:key="`${segment || index}-group`"
								class="relative flex min-w-0 flex-shrink items-center text-sm"
							>
								<div class="flex min-w-0 flex-shrink items-center">
									<ButtonStyled type="transparent">
										<button
											class="cursor-pointer truncate focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
											:aria-current="
												index === breadcrumbSegments.length - 1 ? 'location' : undefined
											"
											:class="{
												'!text-contrast': index === breadcrumbSegments.length - 1,
											}"
											@click="$emit('navigate', index)"
										>
											{{ segment || '' }}
										</button>
									</ButtonStyled>
									<ChevronRightIcon
										v-if="index < breadcrumbSegments.length - 1"
										class="size-4 flex-shrink-0 text-secondary"
										aria-hidden="true"
									/>
								</div>
							</li>
						</TransitionGroup>
					</ol>
				</li>
			</ol>
		</nav>

		<div class="flex flex-shrink-0 items-center gap-2">
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
						{ id: 'upload-zip', shown: false, action: () => $emit('upload-zip') },
						{ id: 'install-from-url', action: () => $emit('unzip-from-url', false) },
						{ id: 'install-cf-pack', action: () => $emit('unzip-from-url', true) },
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
	SearchIcon,
	UploadIcon,
} from '@modrinth/assets'
import { ButtonStyled, OverflowMenu } from '@modrinth/ui'

defineProps<{
	breadcrumbSegments: string[]
	searchQuery: string
	currentFilter: string
	baseId: string
}>()

defineEmits<{
	(e: 'navigate', index: number): void
	(e: 'create', type: 'file' | 'directory'): void
	(e: 'upload' | 'upload-zip' | 'prefetch-home'): void
	(e: 'unzip-from-url', cf: boolean): void
	(e: 'update:searchQuery' | 'filter', value: string): void
}>()
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
