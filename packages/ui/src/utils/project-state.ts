import type { Labrinth } from '@modrinth/api-client'

export type ProjectState =
  | Pick<
      Labrinth.Projects.v3.Project,
      'visibility' | 'in_review' | 'review_outcome'
    >
  | Pick<Labrinth.Projects.v2.Project, 'status'>

/** Derives a legacy badge value without adding legacy state to a v3 project. */
export function getProjectDisplayStatus(
  project: ProjectState,
): Labrinth.Projects.v2.ProjectStatus {
  if ('visibility' in project) {
    if (project.in_review) return 'processing'
    if (project.review_outcome === 'never_reviewed') return 'draft'
    if (project.review_outcome !== 'approved') return project.review_outcome
    return project.visibility === 'listed' ? 'approved' : project.visibility
  }
  return project.status
}

/** Applies the review outcome's visibility ceiling to the requested visibility. */
export function getProjectEffectiveVisibility(
  project: ProjectState,
): Labrinth.Projects.v3.ProjectVisibility {
  if ('visibility' in project) {
    if (project.review_outcome === 'approved') return project.visibility
    if (project.review_outcome === 'withheld') {
      return project.visibility === 'private' ? 'private' : 'unlisted'
    }
    return 'private'
  }
  switch (project.status) {
    case 'approved':
    case 'archived':
    case 'scheduled':
      return 'listed'
    case 'unlisted':
    case 'withheld':
      return 'unlisted'
    default:
      return 'private'
  }
}
