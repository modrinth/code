import { readonly, ref } from 'vue'

const STORAGE_KEY = 'modrinth-quick-instance-count'

export const QUICK_INSTANCE_LIMIT_MAX = 20

function normalizeLimit(limit: number | null): number | null {
	if (limit === null || !Number.isFinite(limit)) return null
	const rounded = Math.max(0, Math.round(limit))
	return rounded >= QUICK_INSTANCE_LIMIT_MAX ? null : rounded
}

const storedLimit = localStorage.getItem(STORAGE_KEY)
const quickInstanceLimit = ref(normalizeLimit(storedLimit === null ? null : Number(storedLimit)))

function setQuickInstanceLimit(limit: number | null): void {
	const normalizedLimit = normalizeLimit(limit)
	quickInstanceLimit.value = normalizedLimit

	if (normalizedLimit === null) {
		localStorage.removeItem(STORAGE_KEY)
	} else {
		localStorage.setItem(STORAGE_KEY, String(normalizedLimit))
	}
}

export function useQuickInstanceLimit() {
	return {
		limit: readonly(quickInstanceLimit),
		setLimit: setQuickInstanceLimit,
	}
}
