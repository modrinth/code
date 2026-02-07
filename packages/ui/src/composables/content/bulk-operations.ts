import { onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

export type BulkOperationType = 'enable' | 'disable' | 'delete' | 'update'

export function useBulkOperation() {
	const isBulkOperating = ref(false)
	const bulkProgress = ref(0)
	const bulkTotal = ref(0)
	const bulkOperation = ref<BulkOperationType | null>(null)

	async function runBulk<T>(
		operation: BulkOperationType,
		items: T[],
		fn: (item: T) => Promise<void>,
		delayMs = 250,
	) {
		isBulkOperating.value = true
		bulkOperation.value = operation
		bulkTotal.value = items.length
		bulkProgress.value = 0

		for (const item of items) {
			await fn(item)
			bulkProgress.value++
			if (delayMs > 0 && bulkProgress.value < items.length) {
				await new Promise((resolve) => setTimeout(resolve, delayMs))
			}
		}

		isBulkOperating.value = false
		bulkOperation.value = null
	}

	function handleBeforeUnload(e: BeforeUnloadEvent) {
		if (isBulkOperating.value) {
			e.preventDefault()
			return ''
		}
	}

	if (typeof window !== 'undefined') {
		watch(isBulkOperating, (operating) => {
			if (operating) {
				window.addEventListener('beforeunload', handleBeforeUnload)
			} else {
				window.removeEventListener('beforeunload', handleBeforeUnload)
			}
		})

		onBeforeUnmount(() => {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		})
	}

	onBeforeRouteLeave(() => {
		if (isBulkOperating.value) {
			return window.confirm('A bulk operation is in progress. Are you sure you want to leave?')
		}
		return true
	})

	return { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk }
}
