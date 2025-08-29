import type { Project, ProjectV3Partial, TeamMember } from '@modrinth/utils'
import type { Ref } from 'vue'

import { createContext } from '.'

export interface ProjectPageContext {
	projectV2: Ref<Project>
	projectV3: Ref<ProjectV3Partial>
	refreshProject: () => Promise<void>
	currentMember: Ref<TeamMember>
}

export const [injectProjectPageContext, provideProjectPageContext] =
	createContext<ProjectPageContext>('root', 'projectPageContext')
