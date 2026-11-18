import { createContext } from '@modrinth/ui'

import type { ProjectReviewPageContext } from './types'

export const [injectProjectReviewPageContext, provideProjectReviewPageContext] =
	createContext<ProjectReviewPageContext>('ProjectReviewPage')
