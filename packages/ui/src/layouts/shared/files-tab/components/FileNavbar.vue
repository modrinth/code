<template>
	<header
		class="@container flex select-none flex-col gap-4"
		:aria-label="formatMessage(messages.fileNavigation)"
	>
		<div v-if="!isEditing" class="flex items-center gap-2 @[800px]:hidden">
			<StyledInput
				:model-value="searchQuery"
				:icon="SearchIcon"
				type="search"
				name="search"
				autocomplete="off"
				:placeholder="formatMessage(messages.searchFiles)"
				class="!h-10"
				input-class="!h-10"
				wrapper-class="flex-1 min-w-0"
				@update:model-value="$emit('update:searchQuery', $event)"
			/>
		</div>
		<div class="flex items-center justify-between gap-2">
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
						<ol
							ref="breadcrumbOuter"
							class="m-0 flex min-w-0 flex-shrink items-center overflow-hidden p-0"
							:class="{ 'breadcrumb-fade-mask': isBreadcrumbOverflowing }"
							:style="
								isBreadcrumbOverflowing
									? { '--scroll-distance': `-${breadcrumbOverflowAmount}px` }
									: undefined
							"
							@mouseenter="onBreadcrumbMouseEnter"
							@mouseleave="onBreadcrumbMouseLeave"
						>
							<TransitionGroup
								ref="breadcrumbInner"
								name="breadcrumb"
								tag="span"
								class="relative flex w-fit items-center"
								:class="{ 'breadcrumbs-scroll': isBreadcrumbAnimating }"
								@animationiteration="onBreadcrumbAnimationIteration"
							>
								<li
									v-for="(segment, index) in breadcrumbs"
									:key="`${segment || index}-group`"
									class="relative flex shrink-0 items-center text-sm"
								>
									<div class="flex shrink-0 items-center">
										<ButtonStyled type="transparent">
											<button
												class="cursor-pointer whitespace-nowrap focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
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
					class="!h-10 hidden @[800px]:inline-flex"
					input-class="!h-10"
					wrapper-class="w-full sm:w-[280px]"
					@update:model-value="$emit('update:searchQuery', $event)"
				/>

				<ButtonStyled v-if="showRefreshButton" type="outlined">
					<button
						type="button"
						class="flex !h-10 items-center gap-2 !border-[1px] !border-surface-5"
						:disabled="refreshing"
						@click="handleRefresh"
					>
						<RefreshCwIcon
							aria-hidden="true"
							class="h-5 w-5 transition-transform"
							:class="refreshing ? 'animate-spin' : ''"
						/>
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
				<ButtonStyled v-if="isLogFile" type="transparent" circular>
					<button
						v-tooltip="formatMessage(messages.shareToMclogs)"
						:aria-label="formatMessage(messages.shareToMclogs)"
						@click="$emit('share')"
					>
						<ShareIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled
					circular
					:type="isEditorFindOpen ? 'standard' : 'transparent'"
					:color="isEditorFindOpen ? 'brand' : 'standard'"
				>
					<button
						v-tooltip="formatMessage(messages.findInFile)"
						:aria-label="formatMessage(messages.findInFile)"
						:aria-pressed="isEditorFindOpen"
						@click="$emit('find')"
					>
						<SearchIcon />
					</button>
				</ButtonStyled>
			</div>
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
	SearchIcon,
	ShareIcon,
	UploadIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import OverflowMenu from '#ui/components/base/OverflowMenu.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
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
	findInFile: {
		id: 'files.navbar.find-in-file',
		defaultMessage: 'Find in file',
	},
})

const props = defineProps<{
	breadcrumbs: string[]
	isEditing: boolean
	editingFileName?: string
	editingFilePath?: string
	isEditingImage?: boolean
	isEditorFindOpen?: boolean
	searchQuery: string
	showRefreshButton?: boolean
	showInstallFromUrl?: boolean
	baseId: string
	disabled?: boolean
	disabledTooltip?: string
}>()

const emit = defineEmits<{
	navigate: [index: number]
	navigateHome: []
	prefetchHome: []
	'update:searchQuery': [value: string]
	create: [type: 'file' | 'directory']
	upload: []
	uploadZip: []
	unzipFromUrl: [cf: boolean]
	refresh: []
	share: []
	find: []
}>()

const refreshing = ref(false)

function handleRefresh() {
	emit('refresh')
	refreshing.value = true
	setTimeout(() => {
		refreshing.value = false
	}, 1000)
}

const breadcrumbOuter = ref<HTMLElement | null>(null)
const breadcrumbInner = ref<{ $el: HTMLElement } | null>(null)
const isBreadcrumbOverflowing = ref(false)
const isBreadcrumbAnimating = ref(false)
const breadcrumbOverflowAmount = ref(0)

let bcHovered = false
let bcStopping = false

function checkBreadcrumbOverflow() {
	const inner = breadcrumbInner.value?.$el
	if (!breadcrumbOuter.value || !inner) return
	const overflow = inner.scrollWidth - breadcrumbOuter.value.clientWidth
	isBreadcrumbOverflowing.value = overflow > 0
	breadcrumbOverflowAmount.value = overflow + 12
}

function onBreadcrumbMouseEnter() {
	bcHovered = true
	bcStopping = false
	if (isBreadcrumbOverflowing.value) {
		isBreadcrumbAnimating.value = true
	}
}

function onBreadcrumbMouseLeave() {
	bcHovered = false
	if (isBreadcrumbAnimating.value) {
		bcStopping = true
	}
}

function onBreadcrumbAnimationIteration() {
	if (bcStopping && !bcHovered) {
		isBreadcrumbAnimating.value = false
		bcStopping = false
	}
}

let bcResizeObserver: ResizeObserver | null = null

onMounted(() => {
	checkBreadcrumbOverflow()
	bcResizeObserver = new ResizeObserver(checkBreadcrumbOverflow)
	if (breadcrumbOuter.value) bcResizeObserver.observe(breadcrumbOuter.value)
	const innerEl = breadcrumbInner.value?.$el
	if (innerEl) bcResizeObserver.observe(innerEl)
})

onBeforeUnmount(() => {
	bcResizeObserver?.disconnect()
})

watch(
	() => props.breadcrumbs,
	() => {
		requestAnimationFrame(checkBreadcrumbOverflow)
	},
)

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

.breadcrumb-fade-mask {
	mask-image: linear-gradient(
		to right,
		transparent,
		black 12px,
		black calc(100% - 12px),
		transparent
	);
}

.breadcrumbs-scroll {
	animation: breadcrumb-scroll 10s ease-in-out infinite;
}

@keyframes breadcrumb-scroll {
	0% {
		transform: translateX(0);
	}
	35%,
	65% {
		transform: translateX(var(--scroll-distance));
	}
	100% {
		transform: translateX(0);
	}
}
</style>
