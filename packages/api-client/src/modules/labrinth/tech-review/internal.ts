import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTechReviewInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_tech_review_internal'
	}

	public async getRules(): Promise<Labrinth.TechReview.Internal.DelphiRule[]> {
		return this.client.request<Labrinth.TechReview.Internal.DelphiRule[]>(
			'/moderation/tech-review/rules',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async createRule(
		rule: Labrinth.TechReview.Internal.WriteDelphiRule,
	): Promise<Labrinth.TechReview.Internal.DelphiRule> {
		return this.client.request<Labrinth.TechReview.Internal.DelphiRule>(
			'/moderation/tech-review/rules',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: rule,
			},
		)
	}

	public async updateRule(
		id: number,
		rule: Labrinth.TechReview.Internal.WriteDelphiRule,
	): Promise<Labrinth.TechReview.Internal.DelphiRule> {
		return this.client.request<Labrinth.TechReview.Internal.DelphiRule>(
			`/moderation/tech-review/rules/${id}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'PUT',
				body: rule,
			},
		)
	}

	public async deleteRule(id: number): Promise<void> {
		return this.client.request<void>(`/moderation/tech-review/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}

	/**
	 * Search for projects awaiting technical review.
	 *
	 * Returns a flat list of file reports with associated project data, ownership
	 * information, and moderation threads provided as lookup maps.
	 *
	 * @param params - Search parameters including pagination, filters, and sorting
	 * @returns Response object containing reports array and lookup maps for projects, threads, and ownership
	 *
	 * @example
	 * ```typescript
	 * const response = await client.labrinth.tech_review_internal.searchProjects({
	 *   limit: 20,
	 *   page: 0,
	 *   sort_by: 'created_asc',
	 *   filter: {
	 *     project_type: ['mod', 'modpack']
	 *   }
	 * })
	 * // Access reports: response.reports
	 * // Access project by ID: response.projects[projectId]
	 * ```
	 */
	public async searchProjects(
		params: Labrinth.TechReview.Internal.SearchProjectsRequest,
	): Promise<Labrinth.TechReview.Internal.SearchResponse> {
		return this.client.request<Labrinth.TechReview.Internal.SearchResponse>(
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
	 * Get detailed information about a specific file report.
	 *
	 * @param reportId - The Delphi report ID
	 * @returns Full report with all issues and details
	 *
	 * @example
	 * ```typescript
	 * const report = await client.labrinth.tech_review_internal.getReport('report-123')
	 * console.log(report.file_name, report.issues.length)
	 * ```
	 */
	public async getReport(reportId: string): Promise<Labrinth.TechReview.Internal.FileReport> {
		return this.client.request<Labrinth.TechReview.Internal.FileReport>(
			`/moderation/tech-review/report/${reportId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Get detailed information about a specific issue.
	 *
	 * @param issueId - The issue ID
	 * @returns Issue with all its details
	 *
	 * @example
	 * ```typescript
	 * const issue = await client.labrinth.tech_review_internal.getIssue('issue-123')
	 * console.log(issue.issue_type, issue.status)
	 * ```
	 */
	public async getIssue(issueId: string): Promise<Labrinth.TechReview.Internal.FileIssue> {
		return this.client.request<Labrinth.TechReview.Internal.FileIssue>(
			`/moderation/tech-review/issue/${issueId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Update the status of a technical review issue detail.
	 *
	 * Allows moderators to mark an individual issue detail as safe (false positive) or unsafe (malicious).
	 *
	 * @param detailId - The ID of the issue detail to update
	 * @param data - The verdict for the detail
	 * @returns Promise that resolves when the update is complete
	 */
	public async updateIssueDetail(
		detailId: string,
		data: Labrinth.TechReview.Internal.UpdateIssueDetailRequest,
	): Promise<void> {
		return this.updateIssueDetails([{ detail_id: detailId, verdict: data.verdict }])
	}

	public async updateIssueDetails(
		data: Labrinth.TechReview.Internal.UpdateIssueRequest[],
	): Promise<void> {
		return this.client.request<void>('/moderation/tech-review/issue-detail', {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: data,
		})
	}

	public async updateGlobalIssueDetails(
		data: Labrinth.TechReview.Internal.UpdateGlobalIssueRequest[],
	): Promise<void> {
		return this.client.request<void>('/moderation/tech-review/global-issue-detail', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	public async searchGlobalIssueDetails(
		params: Labrinth.TechReview.Internal.SearchGlobalIssueDetailsRequest,
	): Promise<Labrinth.TechReview.Internal.SearchGlobalIssueDetailsResponse> {
		return this.client.request<Labrinth.TechReview.Internal.SearchGlobalIssueDetailsResponse>(
			'/moderation/tech-review/global-issue-detail/search',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: params,
			},
		)
	}

	public async getGlobalIssueDetail(
		params: Labrinth.TechReview.Internal.GetGlobalIssueDetailRequest,
	): Promise<Labrinth.TechReview.Internal.GetGlobalIssueDetailResponse> {
		return this.client.request<Labrinth.TechReview.Internal.GetGlobalIssueDetailResponse>(
			'/moderation/tech-review/global-issue-detail/local-traces',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: params,
			},
		)
	}

	public async submitProject(
		projectId: string,
		data: Labrinth.TechReview.Internal.SubmitProjectRequest,
	): Promise<void> {
		return this.client.request<void>(`/moderation/tech-review/submit/${projectId}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Get the project report and thread for a specific project.
	 *
	 * @param projectId - The project ID
	 * @returns The project report (may be null if no reports exist) and the moderation thread
	 */
	public async getProjectReport(
		projectId: string,
	): Promise<Labrinth.TechReview.Internal.ProjectReportResponse> {
		return this.client.request<Labrinth.TechReview.Internal.ProjectReportResponse>(
			`/moderation/tech-review/project/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}
}
