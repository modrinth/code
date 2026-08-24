<script setup lang="ts">
import {
	ArrowDownAZIcon,
	ArrowUpZAIcon,
	ClockArrowDownIcon,
	ClockArrowUpIcon,
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	FolderOpenIcon,
	LinkIcon,
	OrganizationIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	TextCursorInputIcon,
	TrashIcon,
	UserIcon,
} from '@modrinth/assets'
import { useSessionStorage } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button, type OverflowMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import DropdownFilterBar from '#ui/components/base/DropdownFilterBar.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import FilterPills from '#ui/components/base/FilterPills.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

import ContentCardTable from './components/ContentCardTable.vue'
import ContentSelectionBar from './components/ContentSelectionBar.vue'
import ManagedContentCard from './components/managed-content-card/index.vue'
import ConfirmBulkUpdateModal from './components/modals/ConfirmBulkUpdateModal.vue'
import ConfirmDeletionModal from './components/modals/ConfirmDeletionModal.vue'
import ConfirmDisableModal from './components/modals/ConfirmDisableModal.vue'
import ConfirmUnlinkModal from './components/modals/ConfirmUnlinkModal.vue'
import ContentDependencyWarningModal from './components/modals/ContentDependencyWarningModal.vue'
import {
	getClientWarningType,
	useBulkOperation,
	useChangingItems,
	useContentFilters,
	useContentMetadataFilters,
	useContentSearch,
	useContentSelection,
} from './composables'
import { injectContentManager } from './providers/content-manager'
import type {
	BulkOperationStatus,
	ContentActionWarning,
	ContentCardTableItem,
	ContentItem,
} from './types'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ContentPageLayout')

const props = withDefaults(
	defineProps<{
		bottomPadding?: boolean
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
	sortAlphabeticalAscending: {
		id: 'content.page-layout.sort.alphabetical-ascending',
		defaultMessage: 'Name (A-Z)',
	},
	sortAlphabeticalDescending: {
		id: 'content.page-layout.sort.alphabetical-descending',
		defaultMessage: 'Name (Z-A)',
	},
	sortDateAddedNewest: {
		id: 'content.page-layout.sort.date-added-newest',
		defaultMessage: 'Newest first',
	},
	sortDateAddedOldest: {
		id: 'content.page-layout.sort.date-added-oldest',
		defaultMessage: 'Oldest first',
	},
	filter: {
		id: 'content.page-layout.filter.add',
		defaultMessage: 'Filter',
	},
	authorCount: {
		id: 'content.page-layout.filter.author-count',
		defaultMessage: '{count, plural, one {# author} other {# authors}}',
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
	sortByLabel: {
		id: 'content.page-layout.sort.label',
		defaultMessage: 'Sort by {mode}',
	},
	pleaseWait: {
		id: 'content.page-layout.please-wait',
		defaultMessage: 'Please wait',
	},
})

const ctx = injectContentManager()
const skipNonEssentialWarnings = computed(() => ctx.skipNonEssentialWarnings?.value ?? false)

function getItemId(item: ContentItem) {
	return ctx.getItemId?.(item) ?? item.file_path ?? item.file_name ?? item.id
}

type SortMode = 'alphabetical-asc' | 'alphabetical-desc' | 'date-added-newest' | 'date-added-oldest'
const sortMode = ctx.filterPersistKey
	? useSessionStorage<SortMode>(`content-sort:${ctx.filterPersistKey}`, 'alphabetical-asc')
	: ref<SortMode>('alphabetical-asc')

const sortLabels: Record<SortMode, () => string> = {
	'alphabetical-asc': () => formatMessage(messages.sortAlphabeticalAscending),
	'alphabetical-desc': () => formatMessage(messages.sortAlphabeticalDescending),
	'date-added-newest': () => formatMessage(messages.sortDateAddedNewest),
	'date-added-oldest': () => formatMessage(messages.sortDateAddedOldest),
}

const sortOptions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'alphabetical-asc',
		label: formatMessage(messages.sortAlphabeticalAscending),
		icon: ArrowDownAZIcon,
		action: () => (sortMode.value = 'alphabetical-asc'),
	},
	{
		id: 'alphabetical-desc',
		label: formatMessage(messages.sortAlphabeticalDescending),
		icon: ArrowUpZAIcon,
		action: () => (sortMode.value = 'alphabetical-desc'),
	},
	{
		id: 'date-added-newest',
		label: formatMessage(messages.sortDateAddedNewest),
		icon: ClockArrowDownIcon,
		action: () => (sortMode.value = 'date-added-newest'),
	},
	{
		id: 'date-added-oldest',
		label: formatMessage(messages.sortDateAddedOldest),
		icon: ClockArrowUpIcon,
		action: () => (sortMode.value = 'date-added-oldest'),
	},
])

const sortedItems = computed(() => {
	const items = [...ctx.items.value]
	switch (sortMode.value) {
		case 'alphabetical-desc':
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return (
					nameB.toLowerCase().localeCompare(nameA.toLowerCase()) ||
					a.file_name.localeCompare(b.file_name)
				)
			})
		case 'date-added-newest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateB.localeCompare(dateA) || a.file_name.localeCompare(b.file_name)
			})
		case 'date-added-oldest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateA.localeCompare(dateB) || a.file_name.localeCompare(b.file_name)
			})
		default:
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return (
					nameA.toLowerCase().localeCompare(nameB.toLowerCase()) ||
					a.file_name.localeCompare(b.file_name)
				)
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
		showUpdateFilter: false,
		showWarningsFilter: false,
		showStatusFilters: false,
		showEnvironmentWarnings: ctx.showEnvironmentWarnings,
		isPackLocked: ctx.isPackLocked,
		persistKey: ctx.filterPersistKey,
	},
)

const { selectedMetadataFilters, metadataFilterCategories, applyMetadataFilters } =
	useContentMetadataFilters(ctx.items, ctx.filterPersistKey, {
		showSharedContent: ctx.showSharedContentFilter,
		showEnvironmentWarnings: ctx.showEnvironmentWarnings,
	})

const metadataFilterAuthors = computed(() => {
	const authors = new Map<string, NonNullable<ContentItem['owner']>>()
	for (const item of ctx.items.value) {
		if (!item.owner) continue
		authors.set(`${item.owner.type}:${item.owner.id}`, item.owner)
	}
	return authors
})

function getMetadataFilterAuthor(value: string) {
	return metadataFilterAuthors.value.get(value)
}

function getMetadataFilterPreviewAuthor(selectedValues: string[]) {
	const [selectedValue] = selectedValues
	return selectedValues.length === 1 && selectedValue
		? getMetadataFilterAuthor(selectedValue)
		: undefined
}

const metadataFilterPreviewAuthorLimit = 3
const metadataFilterPreviewAuthorSize = 20
const metadataFilterPreviewAuthorOffset = 14

function getMetadataFilterPreviewAuthorValues(selectedValues: string[]) {
	return selectedValues.slice(0, metadataFilterPreviewAuthorLimit)
}

function getMetadataFilterPreviewAuthorOverflow(selectedValues: string[]) {
	return Math.max(0, selectedValues.length - metadataFilterPreviewAuthorLimit)
}

function getMetadataFilterPreviewAuthorStackWidth(selectedValues: string[]) {
	const visibleCount = Math.min(selectedValues.length, metadataFilterPreviewAuthorLimit)
	if (visibleCount === 0) return 0
	return (
		metadataFilterPreviewAuthorSize +
		(visibleCount - 1 + (selectedValues.length > metadataFilterPreviewAuthorLimit ? 1 : 0)) *
			metadataFilterPreviewAuthorOffset
	)
}

function isMetadataFilterOrganization(value: string) {
	return (
		getMetadataFilterAuthor(value)?.type === 'organization' || value.startsWith('organization:')
	)
}

const metadataFilterTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-surface-5 !bg-transparent !px-3 !text-sm !font-medium !text-primary !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-surface-3 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'
const metadataFilterPreviewTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-brand !bg-brand-highlight !px-3 !text-sm !font-medium !text-brand !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-brand-highlight focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5 [&>svg]:!text-brand'

const filterControlsRef = ref<HTMLElement | null>(null)
const projectTypeFiltersRef = ref<HTMLElement | null>(null)
const metadataFiltersRef = ref<HTMLElement | null>(null)
const metadataFiltersWrapped = ref(false)
let filterControlsResizeObserver: ResizeObserver | null = null

function updateMetadataFiltersWrapped() {
	metadataFiltersWrapped.value =
		!!projectTypeFiltersRef.value &&
		!!metadataFiltersRef.value &&
		metadataFiltersRef.value.offsetTop > projectTypeFiltersRef.value.offsetTop
}

function observeFilterControls() {
	filterControlsResizeObserver?.disconnect()
	for (const element of [
		filterControlsRef.value,
		projectTypeFiltersRef.value,
		metadataFiltersRef.value,
	]) {
		if (element) filterControlsResizeObserver?.observe(element)
	}
	updateMetadataFiltersWrapped()
}

onMounted(() => {
	if (typeof ResizeObserver === 'undefined') return
	filterControlsResizeObserver = new ResizeObserver(updateMetadataFiltersWrapped)
	observeFilterControls()
})

watch([filterControlsRef, projectTypeFiltersRef, metadataFiltersRef], observeFilterControls, {
	flush: 'post',
})

onBeforeUnmount(() => {
	filterControlsResizeObserver?.disconnect()
})

function updateFilterChips(nextFilters: string[]) {
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

const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk } = useBulkOperation()

// Sync bulk operation state back to the content manager so providers can suppress refreshes
if (ctx.isBulkOperating) {
	watch(isBulkOperating, (val) => {
		ctx.isBulkOperating!.value = val
	})
}

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const bulkWaiting = ref(false)
const bulkStatusMessage = ref<string | null>(null)
const bulkItemCount = ref(0)

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
	return applyMetadataFilters(applyFilters(searched))
})
const tableItems = computed<ContentCardTableItem[]>(() => {
	const items = filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		const id = getItemId(item)
		const locked = base.locked ?? item.locked ?? false
		const clientWarning = getClientWarningType(item, ctx.showEnvironmentWarnings)
		return {
			...base,
			id,
			locked,
			disabled:
				isChanging(id) || ctx.isBusy.value || isBulkOperating.value || item.installing === true,
			disabledTooltip: ctx.isBusy.value ? (ctx.busyMessage?.value ?? null) : null,
			toggleDisabled: ctx.isBusy.value || base.toggleDisabled,
			toggleDisabledTooltip: ctx.isBusy.value
				? (ctx.busyMessage?.value ?? null)
				: base.toggleDisabledTooltip,
			installing: item.installing === true,
			installProgress: item.installProgress,
			hasUpdate: base.hasUpdate ?? item.has_update,
			isClientOnly: clientWarning !== null,
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

//  Deletion
const pendingDeletionItems = ref<ContentItem[]>([])
const pendingDeletionWarning = ref<ContentActionWarning | null>(null)
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()
const confirmDisableModal = ref<InstanceType<typeof ConfirmDisableModal>>()
const contentDependencyWarningModal = ref<InstanceType<typeof ContentDependencyWarningModal>>()
const pendingDisableItems = ref<ContentItem[]>([])
const pendingDisableWarning = ref<ContentActionWarning | null>(null)
const pendingDependencyWarningItems = ref<ContentCardTableItem[]>([])
const pendingDependencyWarningDependents = ref<
	Array<{
		item: ContentCardTableItem
		dependencies: ContentCardTableItem[]
	}>
>([])
const pendingDependencyWarningDisableTargets = ref<ContentItem[]>([])

function mapToDisplayItem(item: ContentItem) {
	return {
		...ctx.mapToTableItem(item),
		id: getItemId(item),
	}
}

function canDeleteItem(item: ContentItem) {
	return ctx.canDeleteItem?.(item) ?? true
}

function canToggleItem(item: ContentItem) {
	return ctx.canToggleItem?.(item) ?? true
}

const deletableSelectedItems = computed(() => selectedItems.value.filter(canDeleteItem))
const toggleableSelectedItems = computed(() => selectedItems.value.filter(canToggleItem))

async function promptDeleteItems(items: ContentItem[], event?: MouseEvent) {
	const deletableItems = items.filter(canDeleteItem)
	if (deletableItems.length === 0) return
	pendingDeletionItems.value = deletableItems
	pendingDeletionWarning.value = ctx.getDeleteWarning?.(deletableItems) ?? null
	pendingDependencyWarningItems.value = []
	pendingDependencyWarningDependents.value = []
	pendingDependencyWarningDisableTargets.value = []
	const deletingIds = new Set(deletableItems.map(getItemId))

	const warning = ctx.getDeleteDependencyWarning
		? await Promise.resolve()
				.then(() => ctx.getDeleteDependencyWarning!(deletableItems))
				.catch(() => null)
		: null
	if (warning) {
		const remainingDependents = warning.dependents.filter(
			(dependent) => !deletingIds.has(getItemId(dependent.item)),
		)

		if (remainingDependents.length === 0) {
			showDeletionConfirmation(event)
			return
		}

		const relevantDependencyIds = new Set(
			remainingDependents.flatMap((dependent) => dependent.dependencies.map(getItemId)),
		)
		const warningItems = deletableItems.filter((item) => relevantDependencyIds.has(getItemId(item)))
		if (warningItems.length === 0) {
			showDeletionConfirmation(event)
			return
		}

		pendingDependencyWarningItems.value = warningItems.map(mapToDisplayItem)
		pendingDependencyWarningDependents.value = remainingDependents.map((dependent) => ({
			item: mapToDisplayItem(dependent.item),
			dependencies: dependent.dependencies
				.filter((dependency) => relevantDependencyIds.has(getItemId(dependency)))
				.map(mapToDisplayItem),
		}))
		pendingDependencyWarningDisableTargets.value = remainingDependents.map(
			(dependent) => dependent.item,
		)
		contentDependencyWarningModal.value?.show()
		return
	}

	showDeletionConfirmation(event)
}

async function showDeletionConfirmation(event?: MouseEvent) {
	if (
		!pendingDeletionWarning.value &&
		(event?.shiftKey || skipNonEssentialWarnings.value) &&
		!ctx.isBusy.value
	) {
		confirmDelete()
	} else {
		await nextTick()
		confirmDeletionModal.value?.show()
	}
}

async function handleDeleteById(id: string, event?: MouseEvent) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item) {
		await promptDeleteItems([item], event)
	}
}

async function showBulkDeleteModal(event?: MouseEvent) {
	await promptDeleteItems([...selectedItems.value], event)
}

async function confirmDependencyWarningDelete(disableDependentsAfterDeleting: boolean) {
	if (disableDependentsAfterDeleting) {
		pendingDependencyWarningDisableTargets.value =
			pendingDependencyWarningDisableTargets.value.filter((item) => item.enabled)
	} else {
		pendingDependencyWarningDisableTargets.value = []
	}

	pendingDependencyWarningItems.value = []
	pendingDependencyWarningDependents.value = []
	if (pendingDeletionWarning.value) {
		confirmDeletionModal.value?.show()
		return
	}

	await confirmDelete()
}

async function disablePendingDependencyWarningDependents() {
	const items = pendingDependencyWarningDisableTargets.value.filter((item) => item.enabled)
	pendingDependencyWarningDisableTargets.value = []
	if (items.length === 0) return

	await promptDisableItems(items)
}

async function disableItemsWithoutWarning(items: ContentItem[]) {
	if (ctx.bulkDisableItems) {
		await ctx.bulkDisableItems(items)
		return
	}

	for (const item of items) {
		const id = getItemId(item)
		markChanging(id)
		try {
			await ctx.toggleEnabled(item)
		} finally {
			unmarkChanging(id)
		}
	}
}

async function confirmDelete() {
	if (ctx.isBusy.value) return
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []
	pendingDeletionWarning.value = null
	if (itemsToDelete.length === 0) return

	if (ctx.bulkDeleteItems && itemsToDelete.length > 1) {
		isBulkOperating.value = true
		bulkOperation.value = 'delete'
		bulkProgress.value = 0
		bulkTotal.value = itemsToDelete.length
		bulkWaiting.value = true
		try {
			await ctx.bulkDeleteItems(itemsToDelete)
			await disablePendingDependencyWarningDependents()
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}

	if (itemsToDelete.length === 1) {
		const item = itemsToDelete[0]
		const id = getItemId(item)
		markChanging(id)
		try {
			await ctx.deleteItem(item)
			removeFromSelection(id)
			await disablePendingDependencyWarningDependents()
		} finally {
			unmarkChanging(id)
		}
		return
	}

	await runBulk(
		'delete',
		itemsToDelete,
		async (item) => {
			await ctx.deleteItem(item)
			removeFromSelection(getItemId(item))
		},
		{ onComplete: clearSelection },
	)
	await disablePendingDependencyWarningDependents()
}

async function promptDisableItems(items: ContentItem[]) {
	const toggleableItems = items.filter(canToggleItem)
	if (toggleableItems.length === 0) return
	pendingDisableItems.value = toggleableItems
	const warning = ctx.getDisableWarning?.(toggleableItems) ?? null
	if (warning) {
		pendingDisableWarning.value = warning
		confirmDisableModal.value?.show()
		return
	}

	await confirmDisable()
}

async function confirmDisable() {
	if (ctx.isBusy.value) return
	const itemsToDisable = [...pendingDisableItems.value]
	pendingDisableItems.value = []
	pendingDisableWarning.value = null
	if (itemsToDisable.length === 0) return

	if (ctx.bulkDisableItems && itemsToDisable.length > 1) {
		isBulkOperating.value = true
		bulkOperation.value = 'disable'
		bulkProgress.value = 0
		bulkTotal.value = itemsToDisable.length
		bulkWaiting.value = true
		try {
			await disableItemsWithoutWarning(itemsToDisable)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}

	if (itemsToDisable.length === 1) {
		const item = itemsToDisable[0]
		const id = getItemId(item)
		markChanging(id)
		try {
			if (ctx.bulkDisableItems) {
				await ctx.bulkDisableItems(itemsToDisable)
			} else {
				await ctx.toggleEnabled(item)
			}
		} finally {
			unmarkChanging(id)
		}
		return
	}

	await runBulk('disable', itemsToDisable, (item) => disableItemsWithoutWarning([item]), {
		onComplete: clearSelection,
	})
}

async function handleToggleEnabledById(id: string, _value: boolean) {
	if (ctx.isBusy.value) return
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (!item) return
	if (!canToggleItem(item)) return
	if (!_value) {
		await promptDisableItems([item])
		return
	}
	markChanging(id)
	try {
		await ctx.toggleEnabled(item)
	} finally {
		unmarkChanging(id)
	}
}

async function bulkEnable() {
	if (ctx.isBusy.value) return
	const items = toggleableSelectedItems.value.filter((item) => !item.enabled)
	if (items.length === 0) return
	if (ctx.bulkEnableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'enable'
		bulkProgress.value = 0
		bulkTotal.value = items.length
		bulkWaiting.value = true
		try {
			await ctx.bulkEnableItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}
	await runBulk('enable', items, (item) => ctx.toggleEnabled(item), { onComplete: clearSelection })
}

async function bulkDisable() {
	if (ctx.isBusy.value) return
	const items = toggleableSelectedItems.value.filter((item) => item.enabled)
	if (items.length === 0) return
	await promptDisableItems(items)
}

function handleUpdateById(id: string) {
	const item = ctx.items.value.find((item) => getItemId(item) === id)
	if (item?.locked) return
	ctx.updateItem?.(id)
}

function handleSwitchVersionById(id: string) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item && !item.locked) {
		ctx.switchVersion?.(item)
	}
}

// Bulk updating
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal>>()
const pendingBulkUpdateItems = ref<ContentItem[]>([])
const pendingBulkUpdateAll = ref(false)

const hasBulkUpdateSupport = computed(
	() => !!(ctx.bulkUpdateAll || ctx.bulkUpdateItem || ctx.bulkUpdateItems),
)

function promptUpdateAll(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = ctx.items.value.filter((item) => item.has_update && !item.locked)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	pendingBulkUpdateAll.value = true
	if ((event?.shiftKey || skipNonEssentialWarnings.value) && !ctx.isBusy.value) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

function promptUpdateSelected(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = selectedItems.value.filter((item) => item.has_update && !item.locked)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	pendingBulkUpdateAll.value = false
	if ((event?.shiftKey || skipNonEssentialWarnings.value) && !ctx.isBusy.value) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

async function confirmBulkUpdate() {
	if (ctx.isBusy.value) return
	const items = pendingBulkUpdateItems.value
	if (items.length === 0 || !hasBulkUpdateSupport.value) return

	const setBulkStatus = (status: BulkOperationStatus) => {
		bulkStatusMessage.value = status.message ?? null
		bulkProgress.value = status.progress ?? bulkProgress.value
		bulkTotal.value = status.total ?? bulkTotal.value
		bulkWaiting.value = status.waiting ?? false
	}

	try {
		if (pendingBulkUpdateAll.value && ctx.bulkUpdateAll) {
			isBulkOperating.value = true
			bulkOperation.value = 'update'
			bulkProgress.value = 0
			bulkTotal.value = items.length
			bulkItemCount.value = items.length
			bulkStatusMessage.value = null
			bulkWaiting.value = true
			try {
				await ctx.bulkUpdateAll(setBulkStatus)
			} finally {
				clearSelection()
				isBulkOperating.value = false
				bulkOperation.value = null
				bulkProgress.value = 0
				bulkTotal.value = 0
				bulkItemCount.value = 0
				bulkStatusMessage.value = null
				bulkWaiting.value = false
			}
		} else if (ctx.bulkUpdateItems) {
			isBulkOperating.value = true
			bulkOperation.value = 'update'
			bulkProgress.value = 0
			bulkTotal.value = items.length
			bulkItemCount.value = items.length
			bulkStatusMessage.value = null
			bulkWaiting.value = true
			try {
				await ctx.bulkUpdateItems(items)
			} finally {
				clearSelection()
				isBulkOperating.value = false
				bulkOperation.value = null
				bulkProgress.value = 0
				bulkTotal.value = 0
				bulkItemCount.value = 0
				bulkStatusMessage.value = null
				bulkWaiting.value = false
			}
		} else if (ctx.bulkUpdateItem) {
			await runBulk('update', items, ctx.bulkUpdateItem, { onComplete: clearSelection })
		}
	} finally {
		pendingBulkUpdateItems.value = []
		pendingBulkUpdateAll.value = false
	}
}

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
					<div class="flex flex-col gap-4">
						<span v-if="ctx.managedContent.value" class="text-xl font-semibold text-contrast">
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
										contentType: formatContentTypeSentence(
											formatMessage,
											ctx.contentTypeLabel.value,
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
							<div ref="filterControlsRef" class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
								<div ref="projectTypeFiltersRef" class="flex items-center gap-2">
									<TeleportOverflowMenu
										class="!h-[34px] !text-sm !font-medium"
										:icon-only="false"
										:label="formatMessage(messages.sortByLabel, { mode: sortLabels[sortMode]() })"
										:options="sortOptions"
									>
										<ArrowUpZAIcon v-if="sortMode === 'alphabetical-desc'" />
										<ClockArrowDownIcon v-else-if="sortMode === 'date-added-newest'" />
										<ClockArrowUpIcon v-else-if="sortMode === 'date-added-oldest'" />
										<ArrowDownAZIcon v-else />
										{{ sortLabels[sortMode]() }}
										<DropdownIcon />
									</TeleportOverflowMenu>
									<div class="h-6 w-px shrink-0 bg-surface-5" />
									<FilterPills
										:model-value="selectedFilters"
										:options="filterOptions"
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
									class="flex flex-wrap items-center gap-1.5 [&>div:last-of-type]:!h-[34px] [&>div:last-of-type]:!gap-1.5 [&_[data-button]]:!h-[34px]"
								>
									<div
										class="mr-0.5 h-6 w-px shrink-0 bg-surface-5"
										:class="{ invisible: metadataFiltersWrapped }"
									/>
									<DropdownFilterBar
										v-model="selectedMetadataFilters"
										:categories="metadataFilterCategories"
										:show-label="false"
										:add-label="formatMessage(messages.filter)"
										:add-button-class="metadataFilterTriggerClass"
										:preview-trigger-class="metadataFilterPreviewTriggerClass"
										add-button-size="sm"
										checkbox-position="right"
										apply-immediately
									>
										<template #preview-content="{ category, selectedValues, label, summary }">
											<div
												v-if="category.key === 'author'"
												class="flex min-w-0 flex-1 items-center gap-2"
											>
												<template v-if="selectedValues.length === 1">
													<span
														class="flex size-5 shrink-0 items-center justify-center overflow-hidden bg-surface-2 text-brand"
														:class="
															getMetadataFilterPreviewAuthor(selectedValues)?.type ===
															'organization'
																? 'rounded'
																: 'rounded-full'
														"
													>
														<Avatar
															v-if="getMetadataFilterPreviewAuthor(selectedValues)?.avatar_url"
															:src="getMetadataFilterPreviewAuthor(selectedValues)?.avatar_url"
															size="100%"
															:circle="
																getMetadataFilterPreviewAuthor(selectedValues)?.type !==
																'organization'
															"
															no-shadow
															class="!border-0"
														/>
														<OrganizationIcon
															v-else-if="
																getMetadataFilterPreviewAuthor(selectedValues)?.type ===
																'organization'
															"
															class="size-4"
														/>
														<UserIcon v-else class="size-4" />
													</span>
													<span class="min-w-0 truncate font-semibold text-contrast">
														{{ getMetadataFilterPreviewAuthor(selectedValues)?.name ?? summary }}
													</span>
												</template>
												<template v-else>
													<span class="font-medium">{{ label }}:</span>
													<div
														class="relative h-5 shrink-0"
														:style="{
															width: `${getMetadataFilterPreviewAuthorStackWidth(selectedValues)}px`,
														}"
														aria-hidden="true"
													>
														<div
															v-for="(value, index) in getMetadataFilterPreviewAuthorValues(
																selectedValues,
															)"
															:key="value"
															class="absolute top-0 flex size-5 items-center justify-center overflow-hidden border border-solid border-surface-3 bg-surface-4 text-brand"
															:class="
																isMetadataFilterOrganization(value) ? 'rounded' : 'rounded-full'
															"
															:style="{
																left: `${index * metadataFilterPreviewAuthorOffset}px`,
																zIndex:
																	getMetadataFilterPreviewAuthorValues(selectedValues).length -
																	index,
															}"
														>
															<Avatar
																v-if="getMetadataFilterAuthor(value)?.avatar_url"
																:src="getMetadataFilterAuthor(value)?.avatar_url"
																size="100%"
																:circle="!isMetadataFilterOrganization(value)"
																no-shadow
																class="!border-0"
															/>
															<OrganizationIcon
																v-else-if="isMetadataFilterOrganization(value)"
																class="size-3.5"
															/>
															<UserIcon v-else class="size-3.5" />
														</div>
														<div
															v-if="getMetadataFilterPreviewAuthorOverflow(selectedValues) > 0"
															class="absolute top-0 flex size-5 items-center justify-center rounded-full border border-solid border-surface-3 bg-surface-4 text-[10px] font-bold text-contrast"
															:style="{
																left: `${getMetadataFilterPreviewAuthorValues(selectedValues).length * metadataFilterPreviewAuthorOffset}px`,
															}"
														>
															+{{ getMetadataFilterPreviewAuthorOverflow(selectedValues) }}
														</div>
													</div>
													<span class="min-w-0 truncate font-semibold text-contrast">
														{{
															formatMessage(messages.authorCount, { count: selectedValues.length })
														}}
													</span>
												</template>
											</div>
											<span v-else class="min-w-0 flex-1 truncate">
												<span class="font-medium">{{ label }}:</span>
												<span class="ml-1 font-semibold text-contrast">{{ summary }}</span>
											</span>
										</template>
										<template #option="{ category, option, selected }">
											<div
												v-if="category.key === 'author'"
												class="flex min-w-0 flex-1 items-center gap-2"
											>
												<span
													v-tooltip="option.label"
													class="flex size-6 shrink-0 items-center justify-center overflow-hidden bg-surface-2 text-secondary"
													:class="
														getMetadataFilterAuthor(option.value)?.type === 'organization'
															? 'rounded'
															: 'rounded-full'
													"
												>
													<img
														v-if="getMetadataFilterAuthor(option.value)?.avatar_url"
														:src="getMetadataFilterAuthor(option.value)?.avatar_url"
														:alt="option.label"
														class="size-full object-cover"
													/>
													<OrganizationIcon
														v-else-if="
															getMetadataFilterAuthor(option.value)?.type === 'organization'
														"
														class="size-5"
													/>
													<UserIcon v-else class="size-5" />
												</span>
												<span
													v-tooltip="option.label"
													class="min-w-0 truncate font-semibold leading-tight"
													:class="selected ? 'text-contrast' : 'text-primary'"
												>
													{{ option.label }}
												</span>
											</div>
											<span
												v-else
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
									:disabled="isBulkOperating"
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

						<ContentCardTable
							v-model:selected-ids="selectedIds"
							:items="tableItems"
							:show-selection="true"
							@update:enabled="handleToggleEnabledById"
							@delete="handleDeleteById"
							@update="handleUpdateById"
							@switch-version="handleSwitchVersionById"
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
			:selected-items="selectedItems"
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
		<ConfirmBulkUpdateModal
			v-if="hasBulkUpdateSupport"
			ref="confirmBulkUpdateModal"
			:count="pendingBulkUpdateItems.length"
			:server="ctx.deletionContext === 'server'"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@update="confirmBulkUpdate"
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
