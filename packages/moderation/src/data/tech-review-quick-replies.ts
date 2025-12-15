import type { Labrinth } from '@modrinth/api-client'

import type { QuickReply } from '../types/quick-reply'

export interface TechReviewContext {
	project: Labrinth.Projects.v3.Project
	project_owner: Labrinth.TechReview.Internal.Ownership
	reports: Labrinth.TechReview.Internal.FileReport[]
}

export default [] as ReadonlyArray<QuickReply<TechReviewContext>>
