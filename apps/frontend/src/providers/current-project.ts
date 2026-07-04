import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

/**
 * Informs the default layout of the current project ID, if any, because it can't be gleaned from the route which may be a slug
 */
export const [injectCurrentProjectId, provideCurrentProjectId] = createContext<
	Ref<string | undefined>
>('root', 'currentProjectId')
