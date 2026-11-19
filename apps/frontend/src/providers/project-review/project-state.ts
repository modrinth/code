import type { Labrinth } from '@modrinth/api-client'

export { getProjectDisplayStatus as projectDisplayStatus } from '@modrinth/ui'

type ProjectState = Pick<
	Labrinth.Projects.v3.Project,
	'visibility' | 'in_review' | 'review_outcome'
>

export function projectHasReviewDecision(
	project: ProjectState | undefined,
	status: Labrinth.Projects.v2.ProjectStatus,
): boolean {
	if (!project) return false
	if (status === 'processing') return project.in_review
	if (project.in_review) return false
	if (status === 'approved' || status === 'unlisted' || status === 'private')
		return project.review_outcome === 'approved'
	return project.review_outcome === status
}

export function reviewDecisionUpdate(
	status: Labrinth.Projects.v2.ProjectStatus,
): Labrinth.Projects.v3.EditProjectRequest {
	if (status === 'processing') return { in_review: true }
	if (status === 'approved' || status === 'unlisted' || status === 'private')
		return { review_outcome: 'approved', in_review: false }
	if (status === 'rejected' || status === 'withheld')
		return { review_outcome: status, in_review: false }
	throw new Error(`Unsupported review decision: ${status}`)
}
