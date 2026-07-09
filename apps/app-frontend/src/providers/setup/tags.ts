import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { provideTags } from '@modrinth/ui'
import { ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'
import type { CacheBehaviour } from '@/helpers/types'

export function setupTagsProvider(notificationManager: AbstractWebNotificationManager) {
	const { handleError } = notificationManager

	const gameVersions = ref([])
	const loaders = ref([])

	async function loadGameVersions(cacheBehaviour?: CacheBehaviour) {
		const versions = await get_game_versions(cacheBehaviour)
		if (versions) {
			gameVersions.value = versions
		}
	}

	// load game versions from cache
	void loadGameVersions().catch(handleError)
	void get_loaders()
		.then((v) => {
			loaders.value = v
		})
		.catch(handleError)

	provideTags({ gameVersions, loaders })

	return {
		loadGameVersions,
	}
}
