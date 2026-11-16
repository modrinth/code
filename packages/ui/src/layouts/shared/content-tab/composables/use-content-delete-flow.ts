import type { Ref } from 'vue'
import { computed, nextTick, ref } from 'vue'

import type ConfirmDeletionModal from '../components/modals/ConfirmDeletionModal.vue'
import type ContentDependencyWarningModal from '../components/modals/ContentDependencyWarningModal.vue'
import type { ContentManagerContext } from '../providers/content-manager'
import type { ContentActionWarning, ContentCardTableItem, ContentItem } from '../types'
import type { BulkOperationState } from './bulk-operations'

export interface ContentDeleteFlowOptions {
	ctx: ContentManagerContext
	getItemId: (item: ContentItem) => string
	selectedItems: Ref<ContentItem[]>
	clearSelection: () => void
	removeFromSelection: (id: string) => void
	bulk: BulkOperationState
	markChanging: (id: string) => void
	unmarkChanging: (id: string) => void
	promptDisableItems: (items: ContentItem[]) => Promise<void>
}

export function useContentDeleteFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	removeFromSelection,
	bulk,
	markChanging,
	unmarkChanging,
	promptDisableItems,
}: ContentDeleteFlowOptions) {
	const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting, runBulk } = bulk

	const skipNonEssentialWarnings = computed(() => ctx.skipNonEssentialWarnings?.value ?? false)

	const pendingDeletionItems = ref<ContentItem[]>([])
	const pendingDeletionWarning = ref<ContentActionWarning | null>(null)
	const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()
	const contentDependencyWarningModal = ref<InstanceType<typeof ContentDependencyWarningModal>>()
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

	const deletableSelectedItems = computed(() => selectedItems.value.filter(canDeleteItem))

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
		const confirmed = await ctx.confirmDeleteItems?.(pendingDeletionItems.value)
		if (confirmed !== undefined) {
			if (!confirmed) return
			if (pendingDeletionWarning.value) {
				confirmDeletionModal.value?.show()
			} else {
				await confirmDelete()
			}
			return
		}
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
		if ((await ctx.confirmDeleteItems?.(pendingDeletionItems.value)) === false) return
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

	return {
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
	}
}
