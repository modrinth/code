import { injectNotificationManager, provideTags } from '@modrinth/ui'
import { ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

export function setupTagsProvider() {
	const { handleError } = injectNotificationManager()

	const gameVersions = ref([])
	const loaders = ref([])
	get_game_versions()
		.then((v) => {
			gameVersions.value = v
		})
		.catch(handleError)
	get_loaders()
		.then((v) => {
			loaders.value = v
		})
		.catch(handleError)

	provideTags({ gameVersions, loaders })
}
