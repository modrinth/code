import type { Labrinth } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'

export const [injectVersionsContext, provideVersionsContext] = createContext<{
	project: Labrinth.Projects.v2.Project
	versions: Labrinth.Versions.v3.Version[]
}>('ProjectVersions')
