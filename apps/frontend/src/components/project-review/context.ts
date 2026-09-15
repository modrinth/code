import { createContext } from '@modrinth/ui'

import type { ProjectReviewSlots } from './types'
import type { useProjectReviewLayout } from './use-layout'

type ProjectReviewContext = ReturnType<typeof useProjectReviewLayout> & {
	slots: ProjectReviewSlots
}

export const [injectProjectReviewContext, provideProjectReviewContext] =
	createContext<ProjectReviewContext>('ProjectReview')
