import { createContext } from '@modrinth/ui'
import type { Labrinth } from '@modrinth/api-client'

export const [injectVersionsContext, provideVersionsContext] = createContext<{
	project: Labrinth.Projects.v3.Project
	versions: Labrinth.Versions.v3.Version[]
}>('ProjectVersions')
