import { ref } from 'vue'

const STORAGE_KEY = 'modrinth-quick-instance-count'

export const QUICK_INSTANCE_LIMIT_MAX = 20

const storedLimit = Number.parseInt(localStorage.getItem(STORAGE_KEY) ?? '', 10)
const quickInstanceLimit = ref<number | null>(
	Number.isFinite(storedLimit) ? Math.max(0, storedLimit) : null,
)

function setQuickInstanceLimit(limit: number | null): void {
	const normalizedLimit = limit === null ? null : Math.max(0, Math.round(limit))
	quickInstanceLimit.value = normalizedLimit

	if (normalizedLimit === null) {
		localStorage.removeItem(STORAGE_KEY)
	} else {
		localStorage.setItem(STORAGE_KEY, String(normalizedLimit))
	}
}

export function useQuickInstanceLimit() {
	return {
		limit: quickInstanceLimit,
		setLimit: setQuickInstanceLimit,
	}
}
