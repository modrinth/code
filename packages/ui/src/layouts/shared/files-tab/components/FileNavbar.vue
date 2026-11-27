<template>
	<header class="flex select-none flex-col" :aria-label="formatMessage(messages.fileNavigation)">
		<div class="flex min-h-[42px] items-center justify-between gap-2">
			<nav
				:aria-label="formatMessage(messages.breadcrumbNavigation)"
				class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
			>
				<ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
					<li class="mr-4 flex-shrink-0">
						<IconButton
							v-tooltip="formatMessage(messages.backToHome)"
							:label="formatMessage(messages.backToHome)"
							native-type="button"
							class="bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
							@click="$emit('navigateHome')"
							@mouseenter="$emit('prefetchHome')"
						>
							<HomeIcon />
							<span class="sr-only">{{ formatMessage(messages.home) }}</span>
						</IconButton>
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
										<Button
											type="quiet"
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
										</Button>
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

			<Transition name="top-action" mode="out-in">
				<SplitButton
					v-if="!isEditing && !browseOnly"
					v-tooltip="disabled ? disabledTooltip : undefined"
					type="colored"
					color="brand"
					size="lg"
					class="ml-auto shrink-0"
					:options="createOptions"
					:menu-label="formatMessage(messages.moreCreateOptions)"
					:disabled="disabled"
					@click="$emit('upload')"
				>
					<UploadIcon aria-hidden="true" />
					{{ formatMessage(messages.uploadFiles) }}
				</SplitButton>

				<div v-else-if="isEditing && !isEditingImage" class="flex gap-2">
					<IconButton
						v-if="isLogFile && !browseOnly"
						v-tooltip="formatMessage(messages.shareToMclogs)"
						type="quiet"
						:label="formatMessage(messages.shareToMclogs)"
						@click="$emit('share')"
					>
						<ShareIcon />
					</IconButton>
					<IconButton
						v-tooltip="formatMessage(messages.findInFile)"
						:type="isEditorFindOpen ? 'colored' : 'quiet'"
						:color="isEditorFindOpen ? 'brand' : undefined"
						:label="formatMessage(messages.findInFile)"
						:aria-pressed="isEditorFindOpen"
						@click="$emit('find')"
					>
						<SearchIcon />
					</IconButton>
				</div>
			</Transition>
		</div>

		<Collapsible :collapsed="isEditing" :duration-ms="200" overflow-visible fade>
			<ListToolbar
				class="pt-4"
				:search="searchQuery"
				:search-placeholder="formatMessage(messages.searchFiles)"
				@update:search="$emit('update:searchQuery', $event ?? '')"
			>
				<template v-if="showRefreshButton" #actions>
					<IconButton
						v-tooltip="refreshLabel"
						type="quiet"
						size="lg"
						:label="refreshLabel"
						:disabled="refreshing"
						@click="handleRefresh"
					>
						<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
					</IconButton>
				</template>
			</ListToolbar>
		</Collapsible>
	</header>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	ChevronRightIcon,
	FolderOpenIcon,
	HomeIcon,
	LinkIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	UploadIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { Button, type ButtonMenuOption, IconButton, SplitButton } from '#ui/components/base/buttons'
import Collapsible from '#ui/components/base/Collapsible.vue'
import ListToolbar from '#ui/components/base/ListToolbar.vue'
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
	moreCreateOptions: {
		id: 'files.navbar.more-create-options',
		defaultMessage: 'More ways to add files',
	},
	newFile: {
		id: 'files.navbar.new-file',
		defaultMessage: 'New file',
	},
	newFolder: {
		id: 'files.navbar.new-folder',
		defaultMessage: 'New folder',
	},
	uploadFiles: {
		id: 'files.navbar.upload-files',
		defaultMessage: 'Upload files',
	},
	uploadFromZipUrl: {
		id: 'files.navbar.upload-from-zip-url',
		defaultMessage: 'Upload from .zip URL',
	},
	shareToMclogs: {
		id: 'files.navbar.share-to-mclogs',
		defaultMessage: 'Share to mclo.gs',
	},
	findInFile: {
		id: 'files.navbar.find-in-file',
		defaultMessage: 'Find in file',
	},
	refreshing: {
		id: 'files.navbar.refreshing',
		defaultMessage: 'Refreshing...',
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
	disabled?: boolean
	disabledTooltip?: string
	browseOnly?: boolean
}>()

const emit = defineEmits<{
	navigate: [index: number]
	navigateHome: []
	prefetchHome: []
	'update:searchQuery': [value: string]
	create: [type: 'file' | 'directory']
	upload: []
	unzipFromUrl: []
	refresh: []
	share: []
	find: []
}>()

const refreshing = ref(false)

const refreshLabel = computed(() =>
	formatMessage(refreshing.value ? messages.refreshing : commonMessages.refreshButton),
)

const createOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'file',
		label: formatMessage(messages.newFile),
		icon: BoxIcon,
		action: () => emit('create', 'file'),
	},
	{
		id: 'directory',
		label: formatMessage(messages.newFolder),
		icon: FolderOpenIcon,
		action: () => emit('create', 'directory'),
	},
	{ type: 'divider', shown: props.showInstallFromUrl ?? false },
	{
		id: 'install-from-url',
		label: formatMessage(messages.uploadFromZipUrl),
		icon: LinkIcon,
		shown: props.showInstallFromUrl ?? false,
		action: () => emit('unzipFromUrl'),
	},
])

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
.top-action-enter-active,
.top-action-leave-active {
	transition: opacity 150ms ease-in-out;
}

.top-action-enter-from,
.top-action-leave-to {
	opacity: 0;
}

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
