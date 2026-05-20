import { ref } from 'vue'

export function useChangingItems() {
	const changingItems = ref(new Set<string>())

	function markChanging(id: string) {
		changingItems.value = new Set([...changingItems.value, id])
	}

	function unmarkChanging(id: string) {
		const next = new Set(changingItems.value)
		next.delete(id)
		changingItems.value = next
	}

	function isChanging(id: string): boolean {
		return changingItems.value.has(id)
	}

	return { changingItems, markChanging, unmarkChanging, isChanging }
}
