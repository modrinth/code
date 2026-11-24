import { useQueryClient } from '@tanstack/vue-query'
import { type MaybeRefOrGetter, onScopeDispose, toValue } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'

import { worldKeys } from './queries'

const REFRESH_DELAY_MS = 250

export function useRecentWorldsRefresh(enabled: MaybeRefOrGetter<boolean>) {
	const queryClient = useQueryClient()
	let refreshTimer: ReturnType<typeof setTimeout> | undefined
	let refreshRequested = false
	let refreshing = false
	let disposed = false

	const schedule = () => {
		if (disposed || !toValue(enabled)) return
		refreshRequested = true
		if (refreshTimer || refreshing) return
		refreshTimer = setTimeout(() => {
			refreshTimer = undefined
			void refresh()
		}, REFRESH_DELAY_MS)
	}

	const refresh = async () => {
		refreshRequested = false
		refreshing = true
		try {
			await queryClient.invalidateQueries(
				{ queryKey: worldKeys.recent() },
				{ cancelRefetch: false },
			)
		} finally {
			refreshing = false
			if (refreshRequested) schedule()
		}
	}

	useAppEvent('process', schedule)
	useAppEvent('instance', (event) => {
		if (event.event !== 'screenshots_updated' && event.event !== 'synced') schedule()
	})

	onScopeDispose(() => {
		disposed = true
		clearTimeout(refreshTimer)
	})
}
