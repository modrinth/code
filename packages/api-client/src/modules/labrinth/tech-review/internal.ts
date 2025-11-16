import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTechReviewInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_tech_review_internal'
	}

	/**
	 * Search for projects awaiting technical review.
	 *
	 * Returns a list of projects that have been flagged for technical review,
	 * along with their associated reports, ownership information, and moderation threads.
	 *
	 * @param params - Search parameters including pagination, filters, and sorting
	 * @returns Array of projects with their technical review details
	 *
	 * @example
	 * ```typescript
	 * const reviews = await client.labrinth.tech_review_internal.searchProjects({
	 *   limit: 20,
	 *   page: 0,
	 *   sort_by: 'created_asc',
	 *   filter: {
	 *     project_type: ['mod', 'modpack']
	 *   }
	 * })
	 * ```
	 */
	public async searchProjects(
		params: Labrinth.TechReview.Internal.SearchProjectsRequest,
	): Promise<Labrinth.TechReview.Internal.ProjectReview[]> {
		return this.client.request<Labrinth.TechReview.Internal.ProjectReview[]>(
			'/moderation/tech-review/search',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: params,
			},
		)
	}

	/**
	 * Update the status of a technical review issue.
	 *
	 * Allows moderators to mark an issue as safe (false positive), unsafe (malicious),
	 * or leave it as pending for further review.
	 *
	 * @param issueId - The ID of the issue to update
	 * @param data - The new status for the issue
	 * @returns Promise that resolves when the update is complete
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.tech_review_internal.updateIssue('issue-123', {
	 *   status: 'safe'
	 * })
	 * ```
	 */
	public async updateIssue(
		issueId: string,
		data: Labrinth.TechReview.Internal.UpdateIssueRequest,
	): Promise<void> {
		return this.client.request<void>(`/moderation/tech-review/issue/${issueId}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}
}
