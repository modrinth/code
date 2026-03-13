<script setup lang="ts">
import {
	ArrowDownAZIcon,
	ArrowDownZAIcon,
	ClockArrowDownIcon,
	ClockArrowUpIcon,
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	FilterIcon,
	FolderOpenIcon,
	LinkIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	SpinnerIcon,
	TextCursorInputIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import { formatBytes, formatProjectType } from '@modrinth/utils'
import { computed, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import OverflowMenu from '#ui/components/base/OverflowMenu.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import ContentCardTable from './components/ContentCardTable.vue'
import ContentModpackCard from './components/ContentModpackCard.vue'
import ContentSelectionBar from './components/ContentSelectionBar.vue'
import ConfirmBulkUpdateModal from './components/modals/ConfirmBulkUpdateModal.vue'
import ConfirmDeletionModal from './components/modals/ConfirmDeletionModal.vue'
import ConfirmUnlinkModal from './components/modals/ConfirmUnlinkModal.vue'
import {
	isClientOnlyEnvironment,
	useBulkOperation,
	useChangingItems,
	useContentFilters,
	useContentSearch,
	useContentSelection,
} from './composables'
import { injectContentManager } from './providers/content-manager'
import type { ContentCardTableItem, ContentItem } from './types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	loadingContent: {
		id: 'content.page-layout.loading',
		defaultMessage: 'Loading content...',
	},
	failedToLoad: {
		id: 'content.page-layout.failed-to-load',
		defaultMessage: 'Failed to load content',
	},
	additionalContent: {
		id: 'content.page-layout.additional-content',
		defaultMessage: 'Additional content',
	},
	searchPlaceholder: {
		id: 'content.page-layout.search-placeholder',
		defaultMessage: 'Search {count} {contentType}...',
	},
	browseContent: {
		id: 'content.page-layout.browse-content',
		defaultMessage: 'Browse content',
	},
	uploadFiles: {
		id: 'content.page-layout.upload-files',
		defaultMessage: 'Upload files',
	},
	sortAlphabetical: {
		id: 'content.page-layout.sort.alphabetical',
		defaultMessage: 'Alphabetical',
	},
	sortDateAddedNewest: {
		id: 'content.page-layout.sort.date-added-newest',
		defaultMessage: 'Newest first',
	},
	sortDateAddedOldest: {
		id: 'content.page-layout.sort.date-added-oldest',
		defaultMessage: 'Oldest first',
	},
	updateAll: {
		id: 'content.page-layout.update-all',
		defaultMessage: 'Update all',
	},
	noContentFound: {
		id: 'content.page-layout.no-content-found',
		defaultMessage: 'No content found.',
	},
	noExtraContentInstalled: {
		id: 'content.page-layout.empty.no-extra-content-installed',
		defaultMessage: 'No extra content installed',
	},
	noContentInstalled: {
		id: 'content.page-layout.empty.no-content-installed',
		defaultMessage: 'No content installed',
	},
	emptyModpackHint: {
		id: 'content.page-layout.empty.modpack-hint',
		defaultMessage: 'Add additional content on top of this modpack',
	},
	emptyHint: {
		id: 'content.page-layout.empty.hint',
		defaultMessage: 'Browse or upload {contentType} to get started',
	},
	shareProjectNames: {
		id: 'content.page-layout.share.project-names',
		defaultMessage: 'Project names',
	},
	shareFileNames: {
		id: 'content.page-layout.share.file-names',
		defaultMessage: 'File names',
	},
	shareProjectLinks: {
		id: 'content.page-layout.share.project-links',
		defaultMessage: 'Project links',
	},
	shareMarkdownLinks: {
		id: 'content.page-layout.share.markdown-links',
		defaultMessage: 'Markdown links',
	},
	share: {
		id: 'content.page-layout.share.label',
		defaultMessage: 'Share',
	},
	uploadingFiles: {
		id: 'content.page-layout.uploading-files',
		defaultMessage: 'Uploading files ({completed}/{total})',
	},
	sortByLabel: {
		id: 'content.page-layout.sort.label',
		defaultMessage: 'Sort by {mode}',
	},
	busyDescription: {
		id: 'content.page-layout.busy-description',
		defaultMessage: 'Please wait for the operation to complete before editing content.',
	},
	pleaseWait: {
		id: 'content.page-layout.please-wait',
		defaultMessage: 'Please wait',
	},
})

const ctx = injectContentManager()

const uploadOverallProgress = computed(() => {
	const state = ctx.uploadState?.value
	if (!state || !state.isUploading || state.totalFiles === 0) return 0
	return Math.min((state.completedFiles + state.currentFileProgress) / state.totalFiles, 1)
})

type SortMode = 'alphabetical-asc' | 'alphabetical-desc' | 'date-added-newest' | 'date-added-oldest'
const sortMode = ref<SortMode>('alphabetical-asc')

const sortLabels: Record<SortMode, () => string> = {
	'alphabetical-asc': () => formatMessage(messages.sortAlphabetical),
	'alphabetical-desc': () => formatMessage(messages.sortAlphabetical),
	'date-added-newest': () => formatMessage(messages.sortDateAddedNewest),
	'date-added-oldest': () => formatMessage(messages.sortDateAddedOldest),
}

function cycleSortMode() {
	const modes: SortMode[] = [
		'alphabetical-asc',
		'date-added-newest',
		'alphabetical-desc',
		'date-added-oldest',
	]
	const idx = modes.indexOf(sortMode.value)
	sortMode.value = modes[(idx + 1) % modes.length]
}

const sortedItems = computed(() => {
	const items = [...ctx.items.value]
	switch (sortMode.value) {
		case 'alphabetical-desc':
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return nameB.toLowerCase().localeCompare(nameA.toLowerCase())
			})
		case 'date-added-newest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateB.localeCompare(dateA)
			})
		case 'date-added-oldest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateA.localeCompare(dateB)
			})
		default:
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return nameA.toLowerCase().localeCompare(nameB.toLowerCase())
			})
	}
})

const { searchQuery, search } = useContentSearch(sortedItems, [
	'project.title',
	'owner.name',
	'file_name',
])

const { selectedFilters, filterOptions, toggleFilter, applyFilters } = useContentFilters(
	ctx.items,
	{
		showTypeFilters: true,
		showUpdateFilter: ctx.hasUpdateSupport,
		showClientOnlyFilter: ctx.showClientOnlyFilter ?? false,
		isPackLocked: ctx.isPackLocked,
		formatProjectType,
	},
)

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	ctx.getItemId,
)

const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk } = useBulkOperation()

// Sync bulk operation state back to the content manager so providers can suppress refreshes
if (ctx.isBulkOperating) {
	watch(isBulkOperating, (val) => {
		ctx.isBulkOperating!.value = val
	})
}

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const bulkWaiting = ref(false)

const refreshing = ref(false)
async function handleRefresh() {
	if (refreshing.value) return
	refreshing.value = true
	try {
		await ctx.refresh()
	} finally {
		refreshing.value = false
	}
}

const filteredItems = computed(() => {
	const sorted = sortedItems.value
	const searched = search(sorted)
	return applyFilters(searched)
})
const tableItems = computed<ContentCardTableItem[]>(() => {
	return filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		return {
			...base,
			disabled:
				isChanging(base.id) ||
				ctx.isBusy.value ||
				isBulkOperating.value ||
				item.installing === true,
			installing: item.installing === true,
			hasUpdate: !ctx.isPackLocked.value && item.has_update,
			isClientOnly: isClientOnlyEnvironment(item.environment),
			overflowOptions: ctx.getOverflowOptions?.(item),
		}
	})
})

const hasOutdatedProjects = computed(() => ctx.items.value.some((p) => p.has_update))

//  Deletion
const pendingDeletionItems = ref<ContentItem[]>([])
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()

function handleDeleteById(id: string, event?: MouseEvent) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (item) {
		pendingDeletionItems.value = [item]
		if (event?.shiftKey) {
			confirmDelete()
		} else {
			confirmDeletionModal.value?.show()
		}
	}
}

function showBulkDeleteModal(event?: MouseEvent) {
	pendingDeletionItems.value = [...selectedItems.value]
	if (event?.shiftKey) {
		confirmDelete()
	} else {
		confirmDeletionModal.value?.show()
	}
}

async function confirmDelete() {
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []
	if (itemsToDelete.length === 0) return

	if (ctx.bulkDeleteItems && itemsToDelete.length > 1) {
		isBulkOperating.value = true
		bulkOperation.value = 'delete'
		bulkWaiting.value = true
		try {
			await ctx.bulkDeleteItems(itemsToDelete)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkWaiting.value = false
		}
		return
	}

	if (itemsToDelete.length === 1) {
		const item = itemsToDelete[0]
		const id = ctx.getItemId(item)
		markChanging(id)
		await ctx.deleteItem(item)
		removeFromSelection(id)
		unmarkChanging(id)
		return
	}

	await runBulk(
		'delete',
		itemsToDelete,
		async (item) => {
			await ctx.deleteItem(item)
			removeFromSelection(ctx.getItemId(item))
		},
		{ onComplete: clearSelection },
	)
}

async function handleToggleEnabledById(id: string, _value: boolean) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (!item) return
	markChanging(id)
	try {
		await ctx.toggleEnabled(item)
	} finally {
		unmarkChanging(id)
	}
}

async function bulkEnable() {
	const items = selectedItems.value.filter((item) => !item.enabled)
	if (items.length === 0) return
	if (ctx.bulkEnableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'enable'
		bulkWaiting.value = true
		try {
			await ctx.bulkEnableItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkWaiting.value = false
		}
		return
	}
	await runBulk('enable', items, (item) => ctx.toggleEnabled(item), { onComplete: clearSelection })
}

async function bulkDisable() {
	const items = selectedItems.value.filter((item) => item.enabled)
	if (items.length === 0) return
	if (ctx.bulkDisableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'disable'
		bulkWaiting.value = true
		try {
			await ctx.bulkDisableItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkWaiting.value = false
		}
		return
	}
	await runBulk('disable', items, (item) => ctx.toggleEnabled(item), { onComplete: clearSelection })
}

function handleUpdateById(id: string) {
	ctx.updateItem?.(id)
}

// Bulk updating
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal>>()
const pendingBulkUpdateItems = ref<ContentItem[]>([])

const hasBulkUpdateSupport = computed(() => !!(ctx.bulkUpdateItem || ctx.bulkUpdateItems))

function promptUpdateAll(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = ctx.items.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	if (event?.shiftKey) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

function promptUpdateSelected(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = selectedItems.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	if (event?.shiftKey) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

async function confirmBulkUpdate() {
	const items = pendingBulkUpdateItems.value
	if (items.length === 0 || !hasBulkUpdateSupport.value) return

	if (ctx.bulkUpdateItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'update'
		bulkWaiting.value = true
		try {
			await ctx.bulkUpdateItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkWaiting.value = false
		}
	} else if (ctx.bulkUpdateItem) {
		await runBulk('update', items, ctx.bulkUpdateItem, { onComplete: clearSelection })
	}
	pendingBulkUpdateItems.value = []
}

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>

<template>
	<div class="flex flex-col gap-4 pb-6">
		<div
			v-if="ctx.loading.value"
			role="status"
			aria-live="polite"
			class="flex min-h-[50vh] w-full flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			{{ formatMessage(messages.loadingContent) }}
		</div>

		<div
			v-else-if="ctx.error.value"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="universal-card flex flex-col items-center gap-4 p-6">
				<h2 class="m-0 text-xl font-bold">{{ formatMessage(messages.failedToLoad) }}</h2>
				<p class="text-secondary">{{ ctx.error.value.message }}</p>
				<ButtonStyled color="brand">
					<button @click="handleRefresh">{{ formatMessage(commonMessages.retryButton) }}</button>
				</ButtonStyled>
			</div>
		</div>

		<template v-else>
			<Admonition v-if="ctx.isBusy.value && ctx.busyMessage?.value" type="warning">
				<template #header>{{ ctx.busyMessage.value }}</template>
				{{ formatMessage(messages.busyDescription) }}
			</Admonition>

			<ContentModpackCard
				v-if="ctx.modpack.value"
				:project="ctx.modpack.value.project"
				:project-link="ctx.modpack.value.projectLink"
				:version="ctx.modpack.value.version"
				:version-link="ctx.modpack.value.versionLink"
				:owner="ctx.modpack.value.owner"
				:categories="ctx.modpack.value.categories"
				:has-update="ctx.modpack.value.hasUpdate"
				:disabled="ctx.modpack.value.disabled || ctx.isBusy.value"
				:disabled-text="ctx.modpack.value.disabledText ?? ctx.busyMessage?.value ?? (ctx.isBusy.value ? formatMessage(messages.pleaseWait) : undefined)"
				:show-content-hint="
					!!(ctx.showContentHint?.value && ctx.modpack.value && ctx.items.value.length === 0)
				"
				v-on="{
					...(ctx.updateModpack ? { update: () => ctx.updateModpack?.() } : {}),
					...(ctx.viewModpackContent ? { content: () => ctx.viewModpackContent?.() } : {}),
					...(ctx.unlinkModpack ? { unlink: () => confirmUnlinkModal?.show() } : {}),
					...(ctx.openSettings ? { settings: () => ctx.openSettings?.() } : {}),
				}"
				@dismiss-content-hint="ctx.dismissContentHint?.()"
			/>

			<Transition
				enter-active-class="transition-all duration-300 ease-out overflow-hidden"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-40"
				leave-active-class="transition-all duration-200 ease-in overflow-hidden"
				leave-from-class="opacity-100 max-h-40"
				leave-to-class="opacity-0 max-h-0"
				aria-live="polite"
			>
				<Admonition v-if="ctx.uploadState?.value?.isUploading" type="info" show-actions-underneath>
					<template #icon>
						<UploadIcon class="h-6 w-6 flex-none text-brand-blue" />
					</template>
					<template #header>
						{{
							formatMessage(messages.uploadingFiles, {
								completed: ctx.uploadState?.value?.completedFiles ?? 0,
								total: ctx.uploadState?.value?.totalFiles ?? 0,
							})
						}}
					</template>
					<span class="text-secondary">
						{{ formatBytes(ctx.uploadState?.value?.uploadedBytes ?? 0) }}
						/ {{ formatBytes(ctx.uploadState?.value?.totalBytes ?? 0) }} ({{
							Math.round(uploadOverallProgress * 100)
						}}%)
					</span>
					<template #actions>
						<ProgressBar :progress="uploadOverallProgress" :max="1" color="blue" full-width />
					</template>
				</Admonition>
			</Transition>

			<template v-if="ctx.items.value.length > 0">
				<div class="flex flex-col gap-4">
					<span v-if="ctx.modpack.value" class="text-xl font-semibold text-contrast">
						{{ formatMessage(messages.additionalContent) }}
					</span>

					<div class="flex flex-wrap items-center gap-2">
						<StyledInput
							v-model="searchQuery"
							:icon="SearchIcon"
							type="text"
							autocomplete="off"
							:spellcheck="false"
							input-class="!h-10"
							wrapper-class="flex-1 min-w-0"
							clearable
							:placeholder="
								formatMessage(messages.searchPlaceholder, {
									count: tableItems.length,
									contentType: `${ctx.contentTypeLabel.value}${tableItems.length === 1 ? '' : 's'}`,
								})
							"
						/>

						<div class="flex gap-2">
							<ButtonStyled color="brand">
								<button
									v-tooltip="
										ctx.busyMessage?.value ??
										(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
									"
									:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
									class="!h-10 flex items-center gap-2"
									@click="ctx.browse"
								>
									<CompassIcon class="size-5" />
									<span>{{ formatMessage(messages.browseContent) }}</span>
								</button>
							</ButtonStyled>
							<ButtonStyled type="outlined">
								<button
									v-tooltip="
										ctx.busyMessage?.value ??
										(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
									"
									:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
									class="!h-10 !border-button-bg !border-[1px]"
									@click="ctx.uploadFiles"
								>
									<FolderOpenIcon class="size-5" />
									{{ formatMessage(messages.uploadFiles) }}
								</button>
							</ButtonStyled>
						</div>
					</div>

					<div class="@container flex flex-wrap items-center justify-between gap-2">
						<div class="flex flex-wrap items-center gap-1.5">
							<FilterIcon class="size-5 text-secondary" />
							<button
								class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
								:class="
									selectedFilters.length === 0
										? 'border-green bg-brand-highlight text-brand'
										: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
								"
								:aria-pressed="selectedFilters.length === 0"
								@click="selectedFilters = []"
							>
								{{ formatMessage(commonMessages.allProjectType) }}
							</button>
							<button
								v-for="option in filterOptions"
								:key="option.id"
								class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
								:class="
									selectedFilters.includes(option.id)
										? 'border-green bg-brand-highlight text-brand'
										: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
								"
								:aria-pressed="selectedFilters.includes(option.id)"
								@click="toggleFilter(option.id)"
							>
								{{ option.label }}
							</button>
							<div class="hidden @[900px]:block">
								<ButtonStyled type="transparent" hover-color-fill="none">
									<button
										:aria-label="
											formatMessage(messages.sortByLabel, { mode: sortLabels[sortMode]() })
										"
										@click="cycleSortMode"
									>
										<ArrowDownZAIcon v-if="sortMode === 'alphabetical-desc'" /><ClockArrowDownIcon
											v-else-if="sortMode === 'date-added-newest'"
										/><ClockArrowUpIcon
											v-else-if="sortMode === 'date-added-oldest'"
										/><ArrowDownAZIcon v-else />
										{{ sortLabels[sortMode]() }}
									</button>
								</ButtonStyled>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<div class="@[900px]:hidden">
								<ButtonStyled type="transparent" hover-color-fill="none">
									<button
										:aria-label="
											formatMessage(messages.sortByLabel, { mode: sortLabels[sortMode]() })
										"
										@click="cycleSortMode"
									>
										<ArrowDownZAIcon v-if="sortMode === 'alphabetical-desc'" /><ClockArrowDownIcon
											v-else-if="sortMode === 'date-added-newest'"
										/><ClockArrowUpIcon
											v-else-if="sortMode === 'date-added-oldest'"
										/><ArrowDownAZIcon v-else />
										{{ sortLabels[sortMode]() }}
									</button>
								</ButtonStyled>
							</div>

							<ButtonStyled
								v-if="hasBulkUpdateSupport && !ctx.isPackLocked.value && hasOutdatedProjects"
								color="green"
								type="transparent"
								color-fill="text"
								hover-color-fill="background"
							>
								<button :disabled="isBulkOperating || ctx.isBusy.value" @click="promptUpdateAll">
									<DownloadIcon />
									{{ formatMessage(messages.updateAll) }}
								</button>
							</ButtonStyled>

							<ButtonStyled type="transparent" hover-color-fill="none">
								<button :disabled="refreshing || ctx.isBusy.value" @click="handleRefresh">
									<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
									{{ formatMessage(commonMessages.refreshButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>

					<ContentCardTable
						v-model:selected-ids="selectedIds"
						:items="tableItems"
						:show-selection="true"
						@update:enabled="handleToggleEnabledById"
						@delete="handleDeleteById"
						@update="handleUpdateById"
					>
						<template #empty>
							<span>{{ formatMessage(messages.noContentFound) }}</span>
						</template>
					</ContentCardTable>
				</div>
			</template>

			<EmptyState v-else type="empty-inbox">
				<template #heading>
					{{
						formatMessage(
							ctx.modpack.value ? messages.noExtraContentInstalled : messages.noContentInstalled,
						)
					}}
				</template>
				<template #description>
					{{
						ctx.modpack.value
							? formatMessage(messages.emptyModpackHint)
							: formatMessage(messages.emptyHint, {
									contentType: `${ctx.contentTypeLabel.value}s`,
								})
					}}
				</template>
				<template #actions>
					<ButtonStyled type="outlined">
						<button
							v-tooltip="
								ctx.busyMessage?.value ??
								(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
							"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							class="!h-10 !border-button-bg !border-[1px]"
							@click="ctx.uploadFiles"
						>
							<FolderOpenIcon class="size-5" />
							{{ formatMessage(messages.uploadFiles) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button
							v-tooltip="
								ctx.busyMessage?.value ??
								(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
							"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							class="!h-10 flex items-center gap-2"
							@click="ctx.browse"
						>
							<CompassIcon class="size-5" />
							<span>{{ formatMessage(messages.browseContent) }}</span>
						</button>
					</ButtonStyled>
				</template>
			</EmptyState>
		</template>

		<ContentSelectionBar
			:selected-items="selectedItems"
			:content-type-label="ctx.contentTypeLabel.value"
			:is-busy="ctx.isBusy.value"
			:is-bulk-operating="isBulkOperating"
			:bulk-operation="bulkOperation"
			:bulk-progress="bulkProgress"
			:bulk-total="bulkTotal"
			:bulk-waiting="bulkWaiting"
			:aria-label="formatMessage(commonMessages.selectionActionsLabel)"
			@clear="clearSelection"
			@enable="bulkEnable"
			@disable="bulkDisable"
		>
			<template #actions>
				<ButtonStyled
					v-if="
						hasBulkUpdateSupport &&
						!ctx.isPackLocked.value &&
						selectedItems.some((m) => m.has_update)
					"
					type="transparent"
					color="green"
					color-fill="text"
					hover-color-fill="background"
				>
					<button :disabled="ctx.isBusy.value" @click="promptUpdateSelected">
						<DownloadIcon />
						{{ formatMessage(commonMessages.updateButton) }}
					</button>
				</ButtonStyled>

				<ButtonStyled v-if="ctx.shareItems" type="transparent">
					<OverflowMenu
						:options="[
							{
								id: 'share-names',
								action: () => ctx.shareItems!(selectedItems, 'names'),
							},
							{
								id: 'share-file-names',
								action: () => ctx.shareItems!(selectedItems, 'file-names'),
							},
							{
								id: 'share-urls',
								action: () => ctx.shareItems!(selectedItems, 'urls'),
							},
							{
								id: 'share-markdown',
								action: () => ctx.shareItems!(selectedItems, 'markdown'),
							},
						]"
					>
						<ShareIcon />
						{{ formatMessage(messages.share) }}
						<DropdownIcon />
						<template #share-names>
							<TextCursorInputIcon />
							{{ formatMessage(messages.shareProjectNames) }}
						</template>
						<template #share-file-names>
							<FileIcon />
							{{ formatMessage(messages.shareFileNames) }}
						</template>
						<template #share-urls>
							<LinkIcon />
							{{ formatMessage(messages.shareProjectLinks) }}
						</template>
						<template #share-markdown>
							<CodeIcon />
							{{ formatMessage(messages.shareMarkdownLinks) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>

			<template #actions-end>
				<div class="mx-1 h-6 w-px bg-surface-5" />

				<ButtonStyled
					type="transparent"
					color="red"
					color-fill="text"
					hover-color-fill="background"
				>
					<button :disabled="ctx.isBusy.value" @click="showBulkDeleteModal">
						<TrashIcon />
						{{ formatMessage(commonMessages.deleteLabel) }}
					</button>
				</ButtonStyled>
			</template>
		</ContentSelectionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:variant="ctx.deletionContext ?? 'instance'"
			@delete="confirmDelete"
		/>
		<ConfirmBulkUpdateModal
			v-if="hasBulkUpdateSupport"
			ref="confirmBulkUpdateModal"
			:count="pendingBulkUpdateItems.length"
			:server="ctx.deletionContext === 'server'"
			@update="confirmBulkUpdate"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			:server="ctx.deletionContext === 'server'"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>
