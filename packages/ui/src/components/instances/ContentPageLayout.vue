<script setup lang="ts">
import {
	ArrowUpDownIcon,
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	EmptyIllustration,
	FileIcon,
	FilterIcon,
	FolderOpenIcon,
	LinkIcon,
	PowerIcon,
	PowerOffIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	SpinnerIcon,
	TextCursorInputIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import { formatProjectType } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { useBulkOperation } from '../../composables/content/bulk-operations'
import { useChangingItems } from '../../composables/content/changing-items'
import { useContentFilters } from '../../composables/content/content-filtering'
import { useContentSearch } from '../../composables/content/content-search'
import { useContentSelection } from '../../composables/content/content-selection'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { useStickyObserver } from '../../composables/sticky-observer'
import { injectContentManager } from '../../providers/content-manager'
import { commonMessages } from '../../utils/common-messages'
import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Collapsible from '../base/Collapsible.vue'
import FloatingActionBar from '../base/FloatingActionBar.vue'
import OverflowMenu from '../base/OverflowMenu.vue'
import ProgressBar from '../base/ProgressBar.vue'
import StyledInput from '../base/StyledInput.vue'
import ContentCardTable from './ContentCardTable.vue'
import ContentModpackCard from './ContentModpackCard.vue'
import ConfirmBulkUpdateModal from './modals/ConfirmBulkUpdateModal.vue'
import ConfirmDeletionModal from './modals/ConfirmDeletionModal.vue'
import ConfirmUnlinkModal from './modals/ConfirmUnlinkModal.vue'
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
	sortDateAdded: {
		id: 'content.page-layout.sort.date-added',
		defaultMessage: 'Date added',
	},
	updateAll: {
		id: 'content.page-layout.update-all',
		defaultMessage: 'Update all',
	},
	noContentFound: {
		id: 'content.page-layout.no-content-found',
		defaultMessage: 'No content found.',
	},
	noExtraContentAdded: {
		id: 'content.page-layout.empty.no-extra-content',
		defaultMessage: 'No extra content added',
	},
	noContentInstalled: {
		id: 'content.page-layout.empty.no-content-installed',
		defaultMessage: 'No content installed',
	},
	emptyModpackHint: {
		id: 'content.page-layout.empty.modpack-hint',
		defaultMessage: 'You can add content on top of a modpack!',
	},
	emptyHint: {
		id: 'content.page-layout.empty.hint',
		defaultMessage: 'Browse or upload {contentType} to get started',
	},
	selectedCount: {
		id: 'content.page-layout.selected-count',
		defaultMessage: '{count} {contentType} selected',
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
	enable: {
		id: 'content.page-layout.enable',
		defaultMessage: 'Enable',
	},
	disable: {
		id: 'content.page-layout.disable',
		defaultMessage: 'Disable',
	},
	bulkEnabling: {
		id: 'content.page-layout.bulk.enabling',
		defaultMessage: 'Enabling content... ({progress}/{total})',
	},
	bulkDisabling: {
		id: 'content.page-layout.bulk.disabling',
		defaultMessage: 'Disabling content... ({progress}/{total})',
	},
	bulkUpdating: {
		id: 'content.page-layout.bulk.updating',
		defaultMessage: 'Updating content... ({progress}/{total})',
	},
	bulkDeleting: {
		id: 'content.page-layout.bulk.deleting',
		defaultMessage: 'Deleting content... ({progress}/{total})',
	},
	uploadingFiles: {
		id: 'content.page-layout.uploading-files',
		defaultMessage: 'Uploading files ({completed}/{total})',
	},
})

const ctx = injectContentManager()

const uploadOverallProgress = computed(() => {
	const state = ctx.uploadState?.value
	if (!state || !state.isUploading || state.totalFiles === 0) return 0
	return (state.completedFiles + state.currentFileProgress) / state.totalFiles
})

type SortMode = 'alphabetical' | 'date-added'
const sortMode = ref<SortMode>('alphabetical')

const sortLabels: Record<SortMode, () => string> = {
	alphabetical: () => formatMessage(messages.sortAlphabetical),
	'date-added': () => formatMessage(messages.sortDateAdded),
}

function cycleSortMode() {
	const modes: SortMode[] = ['alphabetical', 'date-added']
	const idx = modes.indexOf(sortMode.value)
	sortMode.value = modes[(idx + 1) % modes.length]
}

const sortedItems = computed(() => {
	const items = [...ctx.items.value]
	if (sortMode.value === 'date-added') {
		return items.sort((a, b) => {
			const dateA = a.date_added ?? ''
			const dateB = b.date_added ?? ''
			return dateB.localeCompare(dateA)
		})
	}
	return items.sort((a, b) => {
		const nameA = a.project?.title ?? a.file_name
		const nameB = b.project?.title ?? b.file_name
		return nameA.toLowerCase().localeCompare(nameB.toLowerCase())
	})
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
		isPackLocked: ctx.isPackLocked,
		formatProjectType,
	},
)

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	ctx.getItemId,
)

const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk } = useBulkOperation()

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const contentTableRef = ref<InstanceType<typeof ContentCardTable> | null>(null)
const contentTableEl = computed(() => contentTableRef.value?.$el as HTMLElement | null)
const { isStuck: isTableStuck } = useStickyObserver(contentTableEl)

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

const filteredItems = computed(() => applyFilters(search(sortedItems.value)))
const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		return {
			...base,
			disabled: isChanging(base.id) || ctx.isBusy.value,
			hasUpdate: !ctx.isPackLocked.value && item.has_update,
			overflowOptions: ctx.getOverflowOptions?.(item),
		}
	}),
)

const hasOutdatedProjects = computed(() => ctx.items.value.some((p) => p.has_update))

//  Deletion
const pendingDeletionItems = ref<ContentItem[]>([])
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()

function handleDeleteById(id: string) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (item) {
		pendingDeletionItems.value = [item]
		confirmDeletionModal.value?.show()
	}
}

function showBulkDeleteModal() {
	pendingDeletionItems.value = [...selectedItems.value]
	confirmDeletionModal.value?.show()
}

async function confirmDelete() {
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []
	if (itemsToDelete.length === 0) return

	if (ctx.bulkDeleteItems && itemsToDelete.length > 1) {
		isBulkOperating.value = true
		bulkOperation.value = 'delete'
		try {
			await ctx.bulkDeleteItems(itemsToDelete)
		} finally {
			isBulkOperating.value = false
			bulkOperation.value = null
		}
		clearSelection()
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

	await runBulk('delete', itemsToDelete, async (item) => {
		await ctx.deleteItem(item)
		removeFromSelection(ctx.getItemId(item))
	})
	clearSelection()
}

async function handleToggleEnabledById(id: string, _value: boolean) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (!item) return
	markChanging(id)
	await ctx.toggleEnabled(item)
	unmarkChanging(id)
}

async function bulkEnable() {
	const items = selectedItems.value.filter((item) => !item.enabled)
	if (items.length === 0) return
	if (ctx.bulkEnableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'enable'
		try {
			await ctx.bulkEnableItems(items)
		} finally {
			isBulkOperating.value = false
			bulkOperation.value = null
		}
		clearSelection()
		return
	}
	await runBulk('enable', items, (item) => ctx.toggleEnabled(item))
	clearSelection()
}

async function bulkDisable() {
	const items = selectedItems.value.filter((item) => item.enabled)
	if (items.length === 0) return
	if (ctx.bulkDisableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'disable'
		try {
			await ctx.bulkDisableItems(items)
		} finally {
			isBulkOperating.value = false
			bulkOperation.value = null
		}
		clearSelection()
		return
	}
	await runBulk('disable', items, (item) => ctx.toggleEnabled(item))
	clearSelection()
}

function handleUpdateById(id: string) {
	ctx.updateItem?.(id)
}

// Bulk updating
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal>>()
const pendingBulkUpdateItems = ref<ContentItem[]>([])

function promptUpdateAll() {
	if (!ctx.bulkUpdateItem) return
	const items = ctx.items.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	confirmBulkUpdateModal.value?.show()
}

function promptUpdateSelected() {
	if (!ctx.bulkUpdateItem) return
	const items = selectedItems.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	confirmBulkUpdateModal.value?.show()
}

async function confirmBulkUpdate() {
	const items = pendingBulkUpdateItems.value
	if (items.length === 0 || !ctx.bulkUpdateItem) return

	await runBulk('update', items, ctx.bulkUpdateItem)
	clearSelection()
	pendingBulkUpdateItems.value = []
}

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>

<template>
	<div class="flex flex-col gap-4">
		<div
			v-if="ctx.loading.value"
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
				:disabled-text="ctx.modpack.value.disabledText"
				@update="ctx.updateModpack?.()"
				@content="ctx.viewModpackContent?.()"
				@unlink="ctx.unlinkModpack ? confirmUnlinkModal?.show() : undefined"
			/>

			<Collapsible :collapsed="!ctx.uploadState?.value?.isUploading">
				<Admonition type="info" show-actions-underneath>
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
					{{ ctx.uploadState?.value?.currentFileName }}
					<template #actions>
						<ProgressBar :progress="uploadOverallProgress" :max="1" color="blue" full-width />
					</template>
				</Admonition>
			</Collapsible>

			<template v-if="ctx.items.value.length > 0">
				<span v-if="ctx.modpack.value" class="text-xl font-semibold text-contrast">
					{{ formatMessage(messages.additionalContent) }}
				</span>

				<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
					<StyledInput
						v-model="searchQuery"
						:icon="SearchIcon"
						type="text"
						autocomplete="off"
						:spellcheck="false"
						input-class="!h-10"
						wrapper-class="flex-1"
						clearable
						:placeholder="
							formatMessage(messages.searchPlaceholder, {
								count: ctx.items.value.length,
								contentType: `${ctx.contentTypeLabel.value}${ctx.items.value.length === 1 ? '' : 's'}`,
							})
						"
					/>

					<div class="flex gap-2">
						<ButtonStyled color="brand">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 flex items-center gap-2"
								@click="ctx.browse"
							>
								<CompassIcon class="size-5" />
								<span>{{ formatMessage(messages.browseContent) }}</span>
							</button>
						</ButtonStyled>
						<ButtonStyled type="outlined">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 !border-button-bg !border-[1px]"
								@click="ctx.uploadFiles"
							>
								<FolderOpenIcon class="size-5" />
								{{ formatMessage(messages.uploadFiles) }}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<div
						class="flex flex-col justify-between gap-2 min-[1200px]:flex-row min-[1200px]:items-center"
					>
						<div class="flex flex-wrap items-center gap-1.5">
							<FilterIcon class="size-5 text-secondary" />
							<button
								class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
								:class="
									selectedFilters.length === 0
										? 'border-green bg-brand-highlight text-brand'
										: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
								"
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
								@click="toggleFilter(option.id)"
							>
								{{ option.label }}
							</button>
							<div class="ml-4 mx-0.5 h-5 w-px bg-surface-5" />

							<ButtonStyled type="transparent" hover-color-fill="none">
								<button @click="cycleSortMode">
									<ArrowUpDownIcon />
									{{ sortLabels[sortMode]() }}
								</button>
							</ButtonStyled>
						</div>

						<div class="flex items-center gap-2">
							<ButtonStyled
								v-if="ctx.bulkUpdateItem && !ctx.isPackLocked.value && hasOutdatedProjects"
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
						ref="contentTableRef"
						v-model:selected-ids="selectedIds"
						:items="tableItems"
						:show-selection="true"
						:is-stuck="isTableStuck"
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

			<div v-else class="mx-auto flex flex-col justify-center gap-8 p-6 text-center">
				<EmptyIllustration class="h-[80px] w-auto" />
				<div class="flex flex-col gap-4" :class="ctx.modpack.value ? '' : '-mt-4'">
					<div class="flex flex-col items-center gap-1.5">
						<span class="text-2xl font-semibold text-contrast">
							{{
								formatMessage(
									ctx.modpack.value ? messages.noExtraContentAdded : messages.noContentInstalled,
								)
							}}
						</span>
						<span class="text-primary">
							{{
								ctx.modpack.value
									? formatMessage(messages.emptyModpackHint)
									: formatMessage(messages.emptyHint, {
											contentType: `${ctx.contentTypeLabel.value}s`,
										})
							}}
						</span>
					</div>
					<div class="mx-auto flex gap-2">
						<ButtonStyled type="outlined">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 !border-button-bg !border-[1px]"
								@click="ctx.uploadFiles"
							>
								<FolderOpenIcon class="size-5" />
								{{ formatMessage(messages.uploadFiles) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 flex items-center gap-2"
								@click="ctx.browse"
							>
								<CompassIcon class="size-5" />
								<span>{{ formatMessage(messages.browseContent) }}</span>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</template>

		<FloatingActionBar :shown="selectedItems.length > 0 || isBulkOperating">
			<template v-if="!isBulkOperating">
				<div class="flex items-center gap-0.5">
					<span class="px-4 py-2.5 text-base font-semibold text-contrast">
						{{
							formatMessage(messages.selectedCount, {
								count: selectedItems.length,
								contentType: `${ctx.contentTypeLabel.value}${selectedItems.length === 1 ? '' : 's'}`,
							})
						}}
					</span>
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled type="transparent">
						<button class="!text-primary" @click="clearSelection">
							{{ formatMessage(commonMessages.clearButton) }}
						</button>
					</ButtonStyled>
				</div>

				<div class="ml-auto flex items-center gap-0.5">
					<ButtonStyled
						v-if="
							ctx.bulkUpdateItem &&
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

					<ButtonStyled v-if="selectedItems.every((m) => !m.enabled)" type="transparent">
						<button :disabled="ctx.isBusy.value" @click="bulkEnable">
							<PowerIcon />
							{{ formatMessage(messages.enable) }}
						</button>
					</ButtonStyled>
					<ButtonStyled v-else type="transparent">
						<button :disabled="ctx.isBusy.value" @click="bulkDisable">
							<PowerOffIcon />
							{{ formatMessage(messages.disable) }}
						</button>
					</ButtonStyled>

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
				</div>
			</template>

			<template v-else>
				<div class="flex flex-1 flex-col gap-2">
					<span class="text-sm font-medium text-contrast">
						{{
							formatMessage(
								bulkOperation === 'enable'
									? messages.bulkEnabling
									: bulkOperation === 'disable'
										? messages.bulkDisabling
										: bulkOperation === 'update'
											? messages.bulkUpdating
											: messages.bulkDeleting,
								{ progress: bulkProgress, total: bulkTotal },
							)
						}}
					</span>
					<ProgressBar full-width :progress="bulkProgress" :max="bulkTotal" color="brand" />
				</div>
			</template>
		</FloatingActionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			@delete="confirmDelete"
		/>
		<ConfirmBulkUpdateModal
			v-if="ctx.bulkUpdateItem"
			ref="confirmBulkUpdateModal"
			:count="pendingBulkUpdateItems.length"
			@update="confirmBulkUpdate"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>
