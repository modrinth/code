import { createContext } from '@modrinth/ui'
import type { Project, Version } from '@modrinth/utils'

export const [injectVersionsContext, provideVersionsContext] = createContext<{
	project: Project
	versions: Version[]
}>('ProjectVersions')
