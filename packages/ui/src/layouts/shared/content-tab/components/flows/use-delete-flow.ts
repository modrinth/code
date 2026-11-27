import { computed, nextTick, ref } from 'vue'

import type { ContentManagerContext } from '../../providers/content-manager'
import type { ContentActionWarning, ContentCardTableItem, ContentItem } from '../../types'
import type ConfirmDeletionModal from '../modals/ConfirmDeletionModal.vue'
import type ContentDependencyWarningModal from '../modals/ContentDependencyWarningModal.vue'
import type { ContentFlowState } from './types'

export interface DeleteFlowOptions {
	toDisplayRow: (item: ContentItem) => ContentCardTableItem
	disableItems: (items: ContentItem[]) => Promise<void>
}

export function useDeleteFlow(
	ctx: ContentManagerContext,
	state: ContentFlowState,
	{ toDisplayRow, disableItems }: DeleteFlowOptions,
) {
	const { bulk, markChanging, unmarkChanging, clearSelection, removeFromSelection } = state

	const skipOptionalConfirms = computed(() => ctx.skipOptionalConfirms?.value ?? false)

	const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()
	const dependencyWarningModal = ref<InstanceType<typeof ContentDependencyWarningModal>>()
	const pendingItems = ref<ContentItem[]>([])
	const pendingWarning = ref<ContentActionWarning | null>(null)
	const dependencyItems = ref<ContentCardTableItem[]>([])
	const dependencyDependents = ref<
		Array<{ item: ContentCardTableItem; dependencies: ContentCardTableItem[] }>
	>([])
	const dependentsToDisable = ref<ContentItem[]>([])

	function canRemove(item: ContentItem) {
		return ctx.permissions(item).remove
	}

	async function promptDelete(items: ContentItem[], event?: MouseEvent) {
		const removable = items.filter(canRemove)
		if (removable.length === 0) return
		pendingItems.value = removable
		pendingWarning.value = ctx.warn?.('remove', removable) ?? null
		dependencyItems.value = []
		dependencyDependents.value = []
		dependentsToDisable.value = []
		const deletingIds = new Set(removable.map((item) => item.id))

		const warning = ctx.findDependents
			? await ctx.findDependents(removable).catch(() => null)
			: null
		if (!warning) {
			await showConfirmation(event)
			return
		}

		const remainingDependents = warning.dependents.filter(
			(dependent) => !deletingIds.has(dependent.item.id),
		)
		const relevantIds = new Set(
			remainingDependents.flatMap((dependent) => dependent.dependencies.map((item) => item.id)),
		)
		const warningItems = removable.filter((item) => relevantIds.has(item.id))
		if (remainingDependents.length === 0 || warningItems.length === 0) {
			await showConfirmation(event)
			return
		}

		dependencyItems.value = warningItems.map(toDisplayRow)
		dependencyDependents.value = remainingDependents.map((dependent) => ({
			item: toDisplayRow(dependent.item),
			dependencies: dependent.dependencies
				.filter((dependency) => relevantIds.has(dependency.id))
				.map(toDisplayRow),
		}))
		dependentsToDisable.value = remainingDependents.map((dependent) => dependent.item)
		dependencyWarningModal.value?.show()
	}

	async function showConfirmation(event?: MouseEvent) {
		const result = (await ctx.confirm?.('remove', pendingItems.value)) ?? 'unhandled'
		if (result === 'cancelled') return
		if (result === 'confirmed') {
			if (pendingWarning.value) confirmDeletionModal.value?.show()
			else await confirmDelete()
			return
		}
		if (
			!pendingWarning.value &&
			(event?.shiftKey || skipOptionalConfirms.value) &&
			!ctx.busy.value
		) {
			await confirmDelete()
			return
		}
		await nextTick()
		confirmDeletionModal.value?.show()
	}

	async function confirmDependencyWarningDelete(disableDependentsAfterDeleting: boolean) {
		dependentsToDisable.value = disableDependentsAfterDeleting
			? dependentsToDisable.value.filter((item) => item.enabled)
			: []
		dependencyItems.value = []
		dependencyDependents.value = []
		if ((await ctx.confirm?.('remove', pendingItems.value)) === 'cancelled') return
		if (pendingWarning.value) {
			confirmDeletionModal.value?.show()
			return
		}
		await confirmDelete()
	}

	async function disableDependents() {
		const items = dependentsToDisable.value.filter((item) => item.enabled)
		dependentsToDisable.value = []
		if (items.length > 0) await disableItems(items)
	}

	async function confirmDelete() {
		if (ctx.busy.value) return
		const items = [...pendingItems.value]
		pendingItems.value = []
		pendingWarning.value = null
		if (items.length === 0) return

		if (items.length === 1) {
			const [item] = items
			markChanging(item.id)
			try {
				await ctx.remove(items)
				removeFromSelection(item.id)
				await disableDependents()
			} finally {
				unmarkChanging(item.id)
			}
			return
		}

		bulk.isBulkOperating.value = true
		bulk.bulkOperation.value = 'delete'
		bulk.bulkProgress.value = 0
		bulk.bulkTotal.value = items.length
		bulk.bulkWaiting.value = true
		try {
			await ctx.remove(items)
			await disableDependents()
		} finally {
			clearSelection()
			bulk.isBulkOperating.value = false
			bulk.bulkOperation.value = null
			bulk.bulkProgress.value = 0
			bulk.bulkTotal.value = 0
			bulk.bulkWaiting.value = false
		}
	}

	return {
		confirmDeletionModal,
		dependencyWarningModal,
		pendingItems,
		pendingWarning,
		dependencyItems,
		dependencyDependents,
		canRemove,
		promptDelete,
		confirmDelete,
		confirmDependencyWarningDelete,
	}
}
