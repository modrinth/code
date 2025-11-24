import { createContext } from '@modrinth/ui'
import type { Labrinth } from '@modrinth/api-client'
import type { Version } from '@modrinth/utils'

export const [injectVersionsContext, provideVersionsContext] = createContext<{
	project: Labrinth.Projects.v3.Project
	versions: Version
}>('ProjectVersions')
