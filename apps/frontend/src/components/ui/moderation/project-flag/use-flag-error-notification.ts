import { injectNotificationManager } from '@modrinth/ui'
import { type WatchSource, watch } from 'vue'

export function useFlagErrorNotification(
	error: WatchSource<Error | string | null | undefined>,
	title: string,
) {
	const { addNotification } = injectNotificationManager()
	watch(
		error,
		(value) => {
			if (import.meta.client && value) {
				addNotification({
					type: 'error',
					title,
					text: typeof value === 'string' ? value : value.message,
				})
			}
		},
		{ immediate: true },
	)
}
