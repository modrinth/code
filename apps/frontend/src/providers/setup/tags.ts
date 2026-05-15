import { provideTags } from '@icarus/ui'
import { computed } from 'vue'

export function setupTagsProvider() {
	const generatedState = useGeneratedState()
	provideTags({
		gameVersions: computed(() => generatedState.value.gameVersions),
		loaders: computed(() => generatedState.value.loaders),
	})
}
