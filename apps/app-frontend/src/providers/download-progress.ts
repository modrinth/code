import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { AppEvents } from '@/providers/app-events'

export interface AppDownloadProgressContext {
	progress: Ref<number>
	version: Ref<string | undefined>
}

/* returns unlisten function */
export async function subscribeToDownloadProgress(
	events: AppEvents,
	context: AppDownloadProgressContext,
	version: string,
) {
	return events.on('loading', (event) => {
		if (event.event.type === 'launcher_update') {
			if (!version || event.event.version === version) {
				context.progress.value = event.fraction ?? 1.0
				context.version.value = event.event.version
				console.log(`Progress: ${context.progress.value} ${context.version.value}`)
			}
		}
	})
}

export const [injectAppUpdateDownloadProgress, provideAppUpdateDownloadProgress] =
	createContext<AppDownloadProgressContext>('root', 'appUpdateDownloadProgress')
