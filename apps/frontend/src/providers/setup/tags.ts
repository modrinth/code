import { provideTags } from '@modrinth/ui'
import { computed } from 'vue'

export function setupTagsProvider() {
	const generatedState = useGeneratedState()
	provideTags({
		gameVersions: computed(() => generatedState.value.gameVersions),
		loaders: computed(() => generatedState.value.loaders),
	})
}
