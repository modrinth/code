import { ref } from 'vue'

import { loading_listener } from '@/helpers/events'

export async function useDownloadProgress(version?: string) {
	const downloadProgress = ref(0)
	const downloadVersion = ref<string | undefined>()

	const unlisten = await loading_listener(
		(event: {
			event: {
				type: 'launcher_update'
				version: string
			}
			fraction?: number
		}) => {
			if (event.event.type === 'launcher_update') {
				if (version == undefined || event.event.version === version) {
					downloadProgress.value = event.fraction ?? 1.0
					downloadVersion.value = event.event.version
				}
			}
		},
	)

	return {
		downloadProgress,
		downloadVersion,
		unlisten,
	}
}
