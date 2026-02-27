import type { Labrinth } from '@modrinth/api-client'
import type { Ref } from 'vue'

import { createContext } from './index'

export interface TagsContext {
	gameVersions: Ref<Labrinth.Tags.v2.GameVersion[]>
	loaders: Ref<Labrinth.Tags.v2.Loader[]>
}

export const [injectTags, provideTags] = createContext<TagsContext>('root', 'tags')
