import { queryOptions } from '@tanstack/vue-query'

import { getSandboxUnavailableReason } from '@/platform/app-lib/settings/commands'

export function sandboxUnavailableReasonQueryOptions() {
	return queryOptions({
		queryKey: ['sandbox-unavailable-reason'],
		queryFn: getSandboxUnavailableReason,
		networkMode: 'always',
		staleTime: Infinity,
	})
}
