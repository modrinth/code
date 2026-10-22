import { createContext } from '@modrinth/ui'

import type { createReviewSubmission } from './review-submission'

export const [injectReviewSubmission, provideReviewSubmission] =
	createContext<ReturnType<typeof createReviewSubmission>>('ReviewSubmission')
