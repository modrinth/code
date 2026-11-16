<script setup lang="ts">
import {
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	FolderOpenIcon,
	LinkIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	TextCursorInputIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { Button, TeleportOverflowMenu } from '#ui/components/base/buttons'
import DropdownFilterBar from '#ui/components/base/DropdownFilterBar.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import FilterPills from '#ui/components/base/FilterPills.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import UpdateAllModal from '#ui/components/modal/update-all-modal/index.vue'
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	commonMessages,
	formatContentTypeSentence,
	normalizeProjectType,
} from '#ui/utils/common-messages'

import ContentTable from './components/content-table/index.vue'
import ContentSelectionBar from './components/ContentSelectionBar.vue'
import ContentSortMenu from './components/ContentSortMenu.vue'
import ManagedContentCard from './components/managed-content-card/index.vue'
import ConfirmDeletionModal from './components/modals/ConfirmDeletionModal.vue'
import ConfirmDisableModal from './components/modals/ConfirmDisableModal.vue'
import ConfirmUnlinkModal from './components/modals/ConfirmUnlinkModal.vue'
import ContentDependencyWarningModal from './components/modals/ContentDependencyWarningModal.vue'
import {
	type ContentMetadataFilterValue,
	getClientWarningType,
	getContentWarningType,
	useBulkOperation,
	useChangingItems,
	useContentDeleteFlow,
	useContentFilters,
	useContentMetadataFilters,
	useContentSearch,
	useContentSelection,
	useContentSort,
	useContentToggleFlow,
	useContentUpdateAllFlow,
} from './composables'
import { injectContentManager } from './providers/content-manager'
import type { ContentCardTableItem, ContentItem } from './types'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ContentPageLayout')

const props = withDefaults(
	defineProps<{
		bottomPadding?: boolean
		highlightedItemId?: string
	}>(),
	{
		bottomPadding: true,
	},
)

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
		defaultMessage: 'Search {count, number} {contentType}...',
	},
	browseContent: {
		id: 'content.page-layout.browse-content',
		defaultMessage: 'Browse content',
	},
	uploadFiles: {
		id: 'content.page-layout.upload-files',
		defaultMessage: 'Upload files',
	},
	filter: {
		id: 'content.page-layout.filter.add',
		defaultMessage: 'Filter',
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
	pleaseWait: {
		id: 'content.page-layout.please-wait',
		defaultMessage: 'Please wait',
	},
})

const ctx = injectContentManager()

function getItemId(item: ContentItem) {
	return ctx.getItemId?.(item) ?? item.file_path ?? item.file_name ?? item.id
}

const { sortMode, sortedItems } = useContentSort(ctx.items, ctx.filterPersistKey)

const { searchQuery, search } = useContentSearch(
	sortedItems,
	['project.title', 'owner.name', 'file_name'],
	{ queryParam: ctx.searchQueryParam },
)

const retainedRowIds = ref(new Set<string>())
const { selectedFilters, filterOptions, toggleFilter, applyFilters } = useContentFilters(
	ctx.items,
	{
		showTypeFilters: true,
		showUpdateFilter: false,
		showWarningsFilter: true,
		showStatusFilters: false,
		showEnvironmentWarnings: ctx.showEnvironmentWarnings,
		retainSelectedWarnings: computed(() => retainedRowIds.value.size > 0),
		isPackLocked: ctx.isPackLocked,
		persistKey: ctx.filterPersistKey,
	},
)

const { selectedMetadataFilters, metadataFilterCategories, applyMetadataFilters } =
	useContentMetadataFilters(ctx.items, ctx.filterPersistKey, {
		showSharedContent: ctx.showSharedContentFilter,
		showEnabledFor: !!ctx.setEnabledFor,
		showEnvironmentWarnings: ctx.showEnvironmentWarnings,
	})

const visibleFilterOptions = computed(() => {
	const metadataMatches = applyMetadataFilters(ctx.items.value)
	const warningMatches = metadataMatches.filter(
		(item) => getContentWarningType(item, ctx.showEnvironmentWarnings) !== null,
	)
	const availableTypes = new Set(
		(selectedFilters.value.includes('warnings') ? warningMatches : metadataMatches).map((item) =>
			normalizeProjectType(item.project_type),
		),
	)
	for (const item of ctx.items.value) {
		if (retainedRowIds.value.has(getItemId(item))) {
			availableTypes.add(normalizeProjectType(item.project_type))
		}
	}
	return filterOptions.value.filter((option) =>
		option.id === 'warnings'
			? warningMatches.length > 0 ||
				(selectedFilters.value.includes('warnings') && retainedRowIds.value.size > 0)
			: availableTypes.has(option.id),
	)
})

watch(
	visibleFilterOptions,
	(options) => {
		const availableTypes = new Set(options.map((option) => option.id))
		const validFilters = selectedFilters.value.filter((filter) => availableTypes.has(filter))
		if (validFilters.length !== selectedFilters.value.length) selectedFilters.value = validFilters
	},
	{ immediate: true },
)

watch(searchQuery, () => retainedRowIds.value.clear())

function updateMetadataFilters(filters: ContentMetadataFilterValue) {
	retainedRowIds.value.clear()
	selectedMetadataFilters.value = filters
}

watch(
	() => props.highlightedItemId,
	(id) => {
		if (!id) return
		searchQuery.value = ''
		selectedFilters.value = []
		selectedMetadataFilters.value = {}
	},
	{ immediate: true },
)

const metadataFilterTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-surface-5 !bg-transparent !px-3 !text-sm !font-medium !text-primary !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-surface-3 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'
const metadataFilterPreviewTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-brand !bg-brand-highlight !px-3 !text-sm !font-medium !text-brand !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-brand-highlight focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5 [&>svg]:!text-brand'

const filterControlsRef = ref<HTMLElement | null>(null)
const filterPillsRef = ref<HTMLElement | null>(null)
const metadataFiltersRef = ref<HTMLElement | null>(null)
const filterPillsWrapped = ref(false)
const metadataFiltersWrapped = ref(false)
let filterControlsResizeObserver: ResizeObserver | null = null

function isAtLineStart(element: HTMLElement | null) {
	return (
		!!filterControlsRef.value &&
		!!element &&
		element.offsetLeft <= filterControlsRef.value.offsetLeft
	)
}

function updateFiltersWrapped() {
	filterPillsWrapped.value = isAtLineStart(filterPillsRef.value)
	metadataFiltersWrapped.value = isAtLineStart(metadataFiltersRef.value)
}

function observeFilterControls() {
	filterControlsResizeObserver?.disconnect()
	for (const element of [filterControlsRef.value, filterPillsRef.value, metadataFiltersRef.value]) {
		if (element) filterControlsResizeObserver?.observe(element)
	}
	updateFiltersWrapped()
}

onMounted(() => {
	if (typeof ResizeObserver === 'undefined') return
	filterControlsResizeObserver = new ResizeObserver(updateFiltersWrapped)
	observeFilterControls()
})

watch([filterControlsRef, filterPillsRef, metadataFiltersRef], observeFilterControls, {
	flush: 'post',
})

onBeforeUnmount(() => {
	filterControlsResizeObserver?.disconnect()
})

function updateFilterChips(nextFilters: string[]) {
	retainedRowIds.value.clear()
	if (nextFilters.length === 0) {
		selectedFilters.value = []
		return
	}

	const changedFilter =
		nextFilters.find((filter) => !selectedFilters.value.includes(filter)) ??
		selectedFilters.value.find((filter) => !nextFilters.includes(filter))
	if (changedFilter) toggleFilter(changedFilter)
}

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	getItemId,
)

const bulk = useBulkOperation()
const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting } = bulk

// Sync bulk operation state back to the content manager so providers can suppress refreshes
if (ctx.isBulkOperating) {
	watch(isBulkOperating, (val) => {
		ctx.isBulkOperating!.value = val
	})
}

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const bulkStatusMessage = ref<string | null>(null)

const refreshing = ref(false)
async function handleRefresh() {
	if (refreshing.value) return
	retainedRowIds.value.clear()
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
	const matching = applyMetadataFilters(applyFilters(searched))
	if (retainedRowIds.value.size === 0) return matching
	const matchingIds = new Set(matching.map(getItemId))
	return searched.filter((item) => {
		const id = getItemId(item)
		return matchingIds.has(id) || retainedRowIds.value.has(id)
	})
})
const tableItems = computed<ContentCardTableItem[]>(() => {
	const items = filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		const id = getItemId(item)
		const mutationPending =
			ctx.disableWhileMutating !== false && (isChanging(id) || isBulkOperating.value)
		const locked = base.locked ?? item.locked ?? false
		const clientWarning = base.enabledFor
			? null
			: getClientWarningType(item, ctx.showEnvironmentWarnings)
		return {
			...base,
			id,
			projectType: item.project_type,
			locked,
			disabled: mutationPending || ctx.isBusy.value || item.installing === true,
			disabledTooltip: ctx.isBusy.value
				? (ctx.busyMessage?.value ?? null)
				: mutationPending
					? formatMessage(messages.pleaseWait)
					: item.installing
						? formatMessage(commonMessages.installingLabel)
						: null,
			toggleDisabled: ctx.isBusy.value || base.toggleDisabled,
			toggleDisabledTooltip: ctx.isBusy.value
				? (ctx.busyMessage?.value ?? null)
				: base.toggleDisabledTooltip,
			installing: item.installing === true,
			installProgress: item.installProgress,
			hasUpdate: base.hasUpdate ?? item.has_update,
			clientWarning,
			hideDelete: base.hideDelete,
			hideSwitchVersion: base.hideSwitchVersion ?? !base.versionLink,
			overflowOptions: ctx.getOverflowOptions?.(item),
		}
	})

	const updatable = items.filter((i) => i.hasUpdate)
	if (updatable.length > 0) {
		debug('tableItems: items with hasUpdate=true', {
			count: updatable.length,
			ids: updatable.map((i) => i.id),
			isPackLocked: ctx.isPackLocked.value,
		})
	}

	return items
})

const hasOutdatedProjects = computed(() => {
	const outdated = ctx.items.value.filter((p) => p.has_update && !p.locked)
	if (outdated.length > 0) {
		debug('hasOutdatedProjects: raw items with has_update=true', {
			count: outdated.length,
			items: outdated.map((p) => ({
				id: p.id,
				fileName: p.file_name,
				title: p.project?.title,
				has_update: p.has_update,
				update_version_id: p.update_version_id,
			})),
		})
	}
	return outdated.length > 0
})

const {
	confirmDisableModal,
	pendingDisableItems,
	pendingDisableWarning,
	toggleableSelectedItems,
	promptDisableItems,
	confirmDisable,
	handleToggleEnabledById,
	bulkEnable,
	bulkDisable,
} = useContentToggleFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	markChanging,
	unmarkChanging,
	retainedRowIds,
})

const {
	confirmDeletionModal,
	contentDependencyWarningModal,
	pendingDeletionItems,
	pendingDeletionWarning,
	pendingDependencyWarningItems,
	pendingDependencyWarningDependents,
	deletableSelectedItems,
	handleDeleteById,
	showBulkDeleteModal,
	confirmDelete,
	confirmDependencyWarningDelete,
} = useContentDeleteFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	removeFromSelection,
	bulk,
	markChanging,
	unmarkChanging,
	promptDisableItems,
})

function handleUpdateById(id: string) {
	const item = ctx.items.value.find((item) => getItemId(item) === id)
	if (!item || item.locked) return
	ctx.updateItem?.(id)
}

function handleSwitchVersionById(id: string) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item && !item.locked) {
		ctx.switchVersion?.(item)
	}
}

async function handleSetEnabledForById(id: string, side: 'server' | 'player', enabled: boolean) {
	if (ctx.isBusy.value || !ctx.setEnabledFor) return
	const item = ctx.items.value.find((candidate) => getItemId(candidate) === id)
	if (!item) return

	retainedRowIds.value.add(id)
	await ctx.setEnabledFor(item, side, enabled)
}

const {
	updateAllModal,
	updateAllItems,
	loadingUpdateAll,
	loadingUpdateAllChangelog,
	bulkItemCount,
	bulkUpdateItems,
	hasBulkUpdateSupport,
	loadUpdateAllChangelog,
	preloadUpdateAllChangelog,
	updateAllSelected,
	promptUpdateAll,
	cancelUpdateAll,
	promptUpdateSelected,
} = useContentUpdateAllFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	updateItemById: handleUpdateById,
})

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>

<template>
	<div class="flex flex-col gap-4" :class="{ 'pb-6': props.bottomPadding }">
		<template v-if="!ctx.loading.value">
			<div
				v-if="ctx.error.value"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="universal-card flex flex-col items-center gap-4 p-6">
					<h2 class="m-0 text-xl font-bold">{{ formatMessage(messages.failedToLoad) }}</h2>
					<p class="text-secondary">{{ ctx.error.value.message }}</p>
					<Button type="colored" color="brand" @click="handleRefresh">{{
						formatMessage(commonMessages.retryButton)
					}}</Button>
				</div>
			</div>

			<template v-else>
				<ManagedContentCard
					v-if="ctx.managedContent.value"
					:data="ctx.managedContent.value.card"
					:disabled="ctx.managedContent.value.disabled"
					:disabled-text="ctx.managedContent.value.disabledText"
					:show-view-content="!!ctx.viewManagedContent"
					:show-settings="!!ctx.openManagedContentSettings"
					:show-primary-action="!!ctx.runManagedContentPrimaryAction"
					@view-content="ctx.viewManagedContent?.()"
					@settings="ctx.openManagedContentSettings?.()"
					@primary-action="ctx.runManagedContentPrimaryAction?.($event)"
				/>

				<template v-if="ctx.items.value.length > 0">
					<div class="flex flex-col gap-2">
						<span v-if="ctx.managedContent.value" class="mb-2 text-xl font-semibold text-contrast">
							{{ formatMessage(messages.additionalContent) }}
						</span>

						<div class="flex flex-wrap items-center gap-2">
							<Input
								v-model="searchQuery"
								:icon="SearchIcon"
								type="text"
								autocomplete="off"
								:spellcheck="false"
								size="medium"
								wrapper-class="flex-1 min-w-0"
								clearable
								:placeholder="
									formatMessage(messages.searchPlaceholder, {
										count: tableItems.length,
										contentType: formatContentTypeSentence(
											formatMessage,
											'project',
											tableItems.length,
										),
									})
								"
							/>

							<div class="flex gap-2">
								<Button
									v-tooltip="
										ctx.busyMessage?.value ??
										(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
									"
									type="outlined"
									size="lg"
									:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
									@click="ctx.uploadFiles"
								>
									<FolderOpenIcon class="size-5" />
									{{ formatMessage(messages.uploadFiles) }}
								</Button>
								<Button
									v-tooltip="
										ctx.busyMessage?.value ??
										(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
									"
									type="colored"
									color="brand"
									size="lg"
									:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
									@click="ctx.browse"
								>
									<CompassIcon class="size-5" />
									<span>{{ formatMessage(messages.browseContent) }}</span>
								</Button>
							</div>
						</div>

						<div class="@container flex items-start gap-2">
							<div
								ref="filterControlsRef"
								class="flex min-w-0 flex-1 flex-wrap items-center gap-x-4 gap-y-2"
							>
								<ContentSortMenu v-model="sortMode" />
								<div ref="filterPillsRef" class="relative min-w-0">
									<div
										class="absolute -left-2 top-[5px] h-6 w-px bg-surface-5"
										:class="{ invisible: filterPillsWrapped }"
									/>
									<FilterPills
										:model-value="selectedFilters"
										:options="visibleFilterOptions"
										@update:model-value="updateFilterChips"
									>
										<template #all>
											{{ formatMessage(commonMessages.allProjectType) }}
										</template>
									</FilterPills>
								</div>
								<div
									v-if="metadataFilterCategories.length > 0"
									ref="metadataFiltersRef"
									class="relative flex flex-wrap items-center gap-1.5 [&>div:last-of-type]:!h-[34px] [&>div:last-of-type]:!gap-1.5 [&_[data-button]]:!h-[34px]"
								>
									<div
										class="absolute -left-2 top-[5px] h-6 w-px bg-surface-5"
										:class="{ invisible: metadataFiltersWrapped }"
									/>
									<DropdownFilterBar
										:model-value="selectedMetadataFilters"
										:categories="metadataFilterCategories"
										:show-label="false"
										:add-label="formatMessage(messages.filter)"
										:add-button-class="metadataFilterTriggerClass"
										:preview-trigger-class="metadataFilterPreviewTriggerClass"
										add-button-size="sm"
										checkbox-position="right"
										apply-immediately
										@update:model-value="updateMetadataFilters"
									>
										<template #preview-content="{ label, summary }">
											<span class="min-w-0 flex-1 truncate">
												<span class="font-medium">{{ label }}:</span>
												<span class="ml-1 font-semibold text-contrast">{{ summary }}</span>
											</span>
										</template>
										<template #option="{ option, selected }">
											<span
												class="min-w-0 truncate font-semibold leading-tight"
												:class="selected ? 'text-contrast' : 'text-primary'"
											>
												{{ option.label }}
											</span>
										</template>
									</DropdownFilterBar>
								</div>
							</div>

							<div class="flex shrink-0 items-center gap-2">
								<Button
									v-if="hasBulkUpdateSupport && hasOutdatedProjects"
									v-tooltip="formatMessage(messages.updateAll)"
									type="quiet"
									color="green"
									:disabled="isBulkOperating || loadingUpdateAll"
									class="!text-sm !font-medium hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
									@click="promptUpdateAll"
								>
									<DownloadIcon />
									{{ formatMessage(messages.updateAll) }}
								</Button>

								<Button
									type="quiet"
									:disabled="refreshing"
									class="!text-sm !font-medium"
									@click="handleRefresh"
								>
									<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
									{{ formatMessage(commonMessages.refreshButton) }}
								</Button>
							</div>
						</div>

						<ContentTable
							v-model:selected-ids="selectedIds"
							class="mt-2"
							:sections="[{ id: 'content', items: tableItems }]"
							:columns="{
								selection: true,
								enabledFor: !!ctx.setEnabledFor,
								actions: ['update', 'switchVersion', 'toggle', 'remove'],
							}"
							:highlighted-item-id="highlightedItemId"
							@toggle="handleToggleEnabledById"
							@enabled-for="handleSetEnabledForById"
							@remove="handleDeleteById"
							@update="handleUpdateById"
							@switch-version="handleSwitchVersionById"
						>
							<template #empty>
								<span>{{ formatMessage(messages.noContentFound) }}</span>
							</template>
						</ContentTable>
					</div>
				</template>

				<EmptyState v-else type="empty-inbox">
					<template #heading>
						{{
							formatMessage(
								ctx.managedContent.value
									? messages.noExtraContentInstalled
									: messages.noContentInstalled,
							)
						}}
					</template>
					<template #description>
						{{
							ctx.managedContent.value
								? formatMessage(messages.emptyModpackHint)
								: formatMessage(messages.emptyHint, {
										contentType: formatContentTypeSentence(
											formatMessage,
											ctx.contentTypeLabel.value,
											2,
											'content',
										),
									})
						}}
					</template>
					<template #actions>
						<Button
							v-tooltip="
								ctx.busyMessage?.value ??
								(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
							"
							type="outlined"
							size="lg"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							@click="ctx.uploadFiles"
						>
							<FolderOpenIcon class="size-5" />
							{{ formatMessage(messages.uploadFiles) }}
						</Button>
						<Button
							v-tooltip="
								ctx.busyMessage?.value ??
								(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
							"
							type="colored"
							color="brand"
							size="lg"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							@click="ctx.browse"
						>
							<CompassIcon class="size-5" />
							<span>{{ formatMessage(messages.browseContent) }}</span>
						</Button>
					</template>
				</EmptyState>
			</template>
		</template>

		<ContentSelectionBar
			v-if="!ctx.bulkUpdatesInBackground || bulkOperation !== 'update'"
			:selected-items="bulkOperation === 'update' ? bulkUpdateItems : selectedItems"
			:content-type-label="ctx.contentTypeLabel.value"
			:is-busy="ctx.isBusy.value"
			:busy-tooltip="ctx.busyMessage?.value"
			:is-bulk-operating="isBulkOperating"
			:bulk-operation="bulkOperation"
			:bulk-progress="bulkProgress"
			:bulk-total="bulkTotal"
			:bulk-waiting="bulkWaiting"
			:bulk-status-message="bulkStatusMessage"
			:bulk-item-count="bulkItemCount"
			:aria-label="formatMessage(commonMessages.selectionActionsLabel)"
			:get-item-id="getItemId"
			:toggle-items="toggleableSelectedItems"
			@clear="clearSelection"
			@enable="bulkEnable"
			@disable="bulkDisable"
		>
			<template #actions>
				<Button
					v-if="hasBulkUpdateSupport && selectedItems.some((m) => m.has_update && !m.locked)"
					v-tooltip="formatMessage(commonMessages.updateButton)"
					type="quiet"
					color="green"
					:disabled="loadingUpdateAll"
					class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="promptUpdateSelected"
				>
					<DownloadIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.updateButton) }}</span>
				</Button>

				<TeleportOverflowMenu
					v-if="ctx.shareItems"
					type="quiet"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					:options="[
						{
							id: 'share-names',
							label: formatMessage(messages.shareProjectNames),
							action: () => ctx.shareItems!(selectedItems, 'names'),
						},
						{
							id: 'share-file-names',
							label: formatMessage(messages.shareFileNames),
							action: () => ctx.shareItems!(selectedItems, 'file-names'),
						},
						{
							id: 'share-urls',
							label: formatMessage(messages.shareProjectLinks),
							action: () => ctx.shareItems!(selectedItems, 'urls'),
						},
						{
							id: 'share-markdown',
							label: formatMessage(messages.shareMarkdownLinks),
							action: () => ctx.shareItems!(selectedItems, 'markdown'),
						},
					]"
					class="!w-auto !px-2.5 !rounded-xl"
				>
					<ShareIcon />
					<span class="bar-label">{{ formatMessage(messages.share) }}</span>
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
				</TeleportOverflowMenu>
			</template>

			<template #actions-end>
				<div v-if="deletableSelectedItems.length > 0" class="mx-1 h-6 w-px bg-surface-5" />

				<Button
					v-if="deletableSelectedItems.length > 0"
					v-tooltip="formatMessage(commonMessages.deleteLabel)"
					type="quiet"
					color="red"
					class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="showBulkDeleteModal"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</template>
		</ContentSelectionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:warning="pendingDeletionWarning"
			:variant="ctx.deletionContext ?? 'instance'"
			:backup-tip="pendingDeletionItems.map((i) => i.project?.title ?? i.file_name).join(', ')"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@delete="confirmDelete"
		/>
		<ConfirmDisableModal
			ref="confirmDisableModal"
			:count="pendingDisableItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:warning="pendingDisableWarning"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@disable="confirmDisable"
		/>
		<ContentDependencyWarningModal
			ref="contentDependencyWarningModal"
			:items="pendingDependencyWarningItems"
			:dependents="pendingDependencyWarningDependents"
			:item-type="ctx.contentTypeLabel.value"
			:variant="ctx.deletionContext ?? 'instance'"
			:backup-tip="pendingDeletionItems.map((i) => i.project?.title ?? i.file_name).join(', ')"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@delete="confirmDependencyWarningDelete"
		/>
		<UpdateAllModal
			v-if="hasBulkUpdateSupport"
			ref="updateAllModal"
			:items="updateAllItems"
			:server="ctx.deletionContext === 'server'"
			:loading="loadingUpdateAll"
			:loading-changelog="loadingUpdateAllChangelog"
			:action-disabled="ctx.isBusy.value"
			@changelog="loadUpdateAllChangelog"
			@preload-changelog="preloadUpdateAllChangelog"
			@cancel="cancelUpdateAll"
			@update="updateAllSelected"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			:server="ctx.deletionContext === 'server'"
			:backup-tip="ctx.managedContent.value?.card.manager.name"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>
