import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type ConfirmDisableModal from '../components/modals/ConfirmDisableModal.vue'
import type { ContentManagerContext } from '../providers/content-manager'
import type { ContentActionWarning, ContentItem } from '../types'
import type { BulkOperationState } from './bulk-operations'

export interface ContentToggleFlowOptions {
	ctx: ContentManagerContext
	getItemId: (item: ContentItem) => string
	selectedItems: Ref<ContentItem[]>
	clearSelection: () => void
	bulk: BulkOperationState
	markChanging: (id: string) => void
	unmarkChanging: (id: string) => void
	retainedRowIds: Ref<Set<string>>
}

export function useContentToggleFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	markChanging,
	unmarkChanging,
	retainedRowIds,
}: ContentToggleFlowOptions) {
	const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting, runBulk } = bulk

	const confirmDisableModal = ref<InstanceType<typeof ConfirmDisableModal>>()
	const pendingDisableItems = ref<ContentItem[]>([])
	const pendingDisableWarning = ref<ContentActionWarning | null>(null)

	function canToggleItem(item: ContentItem) {
		return (
			ctx.canToggleItem?.(item) ??
			!!(ctx.toggleEnabled || ctx.bulkEnableItems || ctx.bulkDisableItems)
		)
	}

	const toggleableSelectedItems = computed(() => selectedItems.value.filter(canToggleItem))

	async function disableItemsWithoutWarning(items: ContentItem[]) {
		if (ctx.bulkDisableItems) {
			await ctx.bulkDisableItems(items)
			return
		}
		if (!ctx.toggleEnabled) return

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

	async function promptDisableItems(items: ContentItem[]) {
		const toggleableItems = items.filter(canToggleItem)
		if (toggleableItems.length === 0) return
		if (ctx.confirmAction && !(await ctx.confirmAction('disable', toggleableItems))) return
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
				retainedRowIds.value.add(id)
				if (ctx.bulkDisableItems) {
					await ctx.bulkDisableItems(itemsToDisable)
				} else {
					await ctx.toggleEnabled?.(item)
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
		if (ctx.isBusy.value || !ctx.toggleEnabled) return
		const item = ctx.items.value.find((i) => getItemId(i) === id)
		if (!item) return
		if (!canToggleItem(item)) return
		if (!_value) {
			await promptDisableItems([item])
			return
		}
		if (ctx.confirmAction && !(await ctx.confirmAction('enable', [item]))) return
		markChanging(id)
		try {
			retainedRowIds.value.add(id)
			await ctx.toggleEnabled(item)
		} finally {
			unmarkChanging(id)
		}
	}

	async function bulkEnable() {
		if (ctx.isBusy.value) return
		const items = toggleableSelectedItems.value.filter((item) => !item.enabled)
		if (items.length === 0) return
		if (ctx.confirmAction && !(await ctx.confirmAction('enable', items))) return
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
		if (!ctx.toggleEnabled) return
		await runBulk('enable', items, (item) => ctx.toggleEnabled!(item), {
			onComplete: clearSelection,
		})
	}

	async function bulkDisable() {
		if (ctx.isBusy.value) return
		const items = toggleableSelectedItems.value.filter((item) => item.enabled)
		if (items.length === 0) return
		await promptDisableItems(items)
	}

	return {
		confirmDisableModal,
		pendingDisableItems,
		pendingDisableWarning,
		toggleableSelectedItems,
		promptDisableItems,
		confirmDisable,
		handleToggleEnabledById,
		bulkEnable,
		bulkDisable,
	}
}
