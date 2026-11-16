import { useQuery } from '@tanstack/vue-query'

import { refreshSharedInstanceLocks } from '@/platform/app-lib/shared-instances/commands'

export function useSharedInstanceLockSync() {
	useQuery({
		queryKey: ['shared-instance-lock-sync'],
		queryFn: async () => {
			await refreshSharedInstanceLocks()
			return true
		},
		retry: false,
		staleTime: 60_000,
		refetchOnWindowFocus: true,
		refetchInterval: 300_000,
	})
}
