import type { Project, ProjectV3Partial } from '@modrinth/utils'
import type { Ref } from 'vue'

import { createContext } from '.'

export interface ProjectPageContext {
	projectV2: Ref<Project>
	projectV3: Ref<ProjectV3Partial>
	refreshProject: () => Promise<void>
}

export const [injectProjectPageContext, provideProjectPageContext] =
	createContext<ProjectPageContext>('root', 'projectPageContext')
