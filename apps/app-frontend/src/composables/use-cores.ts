import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { apiClient } from '@/services/api-client'

export const coresQueryKey = ['cores']

/** The account's paired Cores. `needsPairing` is true when there is none to host a server on. */
export function useCores() {
	const query = useQuery({
		queryKey: coresQueryKey,
		queryFn: () => apiClient.archon.cores_v1.list(),
		staleTime: 30_000,
	})
	const cores = computed(() => query.data.value ?? [])
	const needsPairing = computed(() => !query.isLoading.value && cores.value.length === 0)
	return { ...query, cores, needsPairing }
}

// The one "Pair a Core" modal is mounted in App.vue and registers itself here.
let showPairCoreModal: (() => Promise<boolean>) | null = null

export function registerPairCoreModal(show: (() => Promise<boolean>) | null) {
	showPairCoreModal = show
}

/** Open "Pair a Core". Resolves true once a Core is paired, false if the modal is closed. */
export function openPairCoreModal(): Promise<boolean> {
	return showPairCoreModal?.() ?? Promise.resolve(false)
}

/**
 * Call before anything that creates a server. Resolves true right away when a Core is paired;
 * otherwise opens "Pair a Core" and resolves true only once pairing succeeds.
 */
export async function ensureCorePaired(): Promise<boolean> {
	const cores = await apiClient.archon.cores_v1.list().catch(() => [])
	return cores.length > 0 || openPairCoreModal()
}
