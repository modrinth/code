import { ref } from 'vue'

import type { ContentManagerContext } from '../../providers/content-manager'
import type { ContentActionWarning, ContentItem } from '../../types'
import type ConfirmDisableModal from '../modals/ConfirmDisableModal.vue'
import type { ContentFlowState } from './types'

export function useToggleFlow(ctx: ContentManagerContext, state: ContentFlowState) {
	const { bulk, clearSelection, retain } = state

	const confirmDisableModal = ref<InstanceType<typeof ConfirmDisableModal>>()
	const pendingDisableItems = ref<ContentItem[]>([])
	const pendingDisableWarning = ref<ContentActionWarning | null>(null)

	function canToggle(item: ContentItem) {
		return ctx.permissions(item).toggle
	}

	async function runBulkToggle(items: ContentItem[], enabled: boolean) {
		bulk.isBulkOperating.value = true
		bulk.bulkOperation.value = enabled ? 'enable' : 'disable'
		bulk.bulkProgress.value = 0
		bulk.bulkTotal.value = items.length
		bulk.bulkWaiting.value = true
		try {
			await ctx.setEnabled(items, enabled)
		} finally {
			clearSelection()
			bulk.isBulkOperating.value = false
			bulk.bulkOperation.value = null
			bulk.bulkProgress.value = 0
			bulk.bulkTotal.value = 0
			bulk.bulkWaiting.value = false
		}
	}

	async function runSingleToggle(item: ContentItem, enabled: boolean) {
		retain(item.id)
		await ctx.setEnabled([item], enabled)
	}

	async function disableItems(items: ContentItem[]) {
		const toggleable = items.filter(canToggle)
		if (toggleable.length === 0) return
		if ((await ctx.confirm?.('disable', toggleable)) === 'cancelled') return
		pendingDisableItems.value = toggleable
		const warning = ctx.warn?.('disable', toggleable) ?? null
		if (warning) {
			pendingDisableWarning.value = warning
			confirmDisableModal.value?.show()
			return
		}
		await confirmDisable()
	}

	async function confirmDisable() {
		if (ctx.busy.value) return
		const items = [...pendingDisableItems.value]
		pendingDisableItems.value = []
		pendingDisableWarning.value = null
		if (items.length === 0) return
		if (items.length === 1) {
			await runSingleToggle(items[0], false)
			return
		}
		await runBulkToggle(items, false)
	}

	async function toggle(item: ContentItem, enabled: boolean) {
		if (ctx.busy.value || !canToggle(item)) return
		if (!enabled) {
			await disableItems([item])
			return
		}
		if ((await ctx.confirm?.('enable', [item])) === 'cancelled') return
		await runSingleToggle(item, true)
	}

	async function enableItems(items: ContentItem[]) {
		if (ctx.busy.value) return
		const toEnable = items.filter((item) => canToggle(item) && !item.enabled)
		if (toEnable.length === 0) return
		if ((await ctx.confirm?.('enable', toEnable)) === 'cancelled') return
		await runBulkToggle(toEnable, true)
	}

	async function disableEnabledItems(items: ContentItem[]) {
		if (ctx.busy.value) return
		const toDisable = items.filter((item) => item.enabled)
		if (toDisable.length === 0) return
		await disableItems(toDisable)
	}

	return {
		confirmDisableModal,
		pendingDisableItems,
		pendingDisableWarning,
		canToggle,
		toggle,
		disableItems,
		enableItems,
		disableEnabledItems,
		confirmDisable,
	}
}
