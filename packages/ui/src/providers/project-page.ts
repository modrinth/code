import type { Labrinth } from '@modrinth/api-client/src/modules/types'
// TODO: api client this shit
import type { TeamMember } from '@modrinth/utils'
import type { Ref } from 'vue'

import { createContext } from '.'

export interface ProjectPageContext {
	projectV2: Ref<Labrinth.Projects.v2.Project>
	projectV3: Ref<Labrinth.Projects.v3.Project>
	refreshProject: () => Promise<void>
	refreshVersions: () => Promise<void>
	currentMember: Ref<TeamMember>
}

export const [injectProjectPageContext, provideProjectPageContext] =
	createContext<ProjectPageContext>('root', 'projectPageContext')
