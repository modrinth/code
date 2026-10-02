import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { ProjectReviewSlot, ProjectReviewSlots } from './types'
import type { useProjectReviewLayout } from './use-layout'

type ProjectReviewContext = ReturnType<typeof useProjectReviewLayout> & {
	slots: ProjectReviewSlots
}

export const [injectProjectReviewContext, provideProjectReviewContext] =
	createContext<ProjectReviewContext>('ProjectReview')

export const [injectReviewSlot, provideReviewSlot] =
	createContext<Ref<ProjectReviewSlot>>('ProjectReviewSlot')
