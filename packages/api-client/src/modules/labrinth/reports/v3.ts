import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthReportsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_reports_v3'
	}

	/**
	 * Get a report by ID
	 *
	 * @param id - Report ID
	 * @returns Promise resolving to the report data
	 *
	 * @example
	 * ```typescript
	 * const report = await client.labrinth.reports_v3.get('abc123')
	 * ```
	 */
	public async get(id: string): Promise<Labrinth.Reports.v3.Report> {
		return this.client.request<Labrinth.Reports.v3.Report>(`/report/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * List reports for the current user (or all reports if moderator)
	 *
	 * @param params - Optional query parameters for count, offset, and whether to show all reports
	 * @returns Promise resolving to an array of reports
	 *
	 * @example
	 * ```typescript
	 * const reports = await client.labrinth.reports_v3.list({ count: 100 })
	 * ```
	 */
	public async list(
		params?: Labrinth.Reports.v3.ListReportsParams,
	): Promise<Labrinth.Reports.v3.Report[]> {
		const queryParams: Record<string, string> = {}
		if (params?.count != null) queryParams.count = String(params.count)
		if (params?.offset != null) queryParams.offset = String(params.offset)
		if (params?.all != null) queryParams.all = String(params.all)

		return this.client.request<Labrinth.Reports.v3.Report[]>(`/report`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: Object.keys(queryParams).length > 0 ? queryParams : undefined,
		})
	}

	/**
	 * Get multiple reports by IDs
	 *
	 * @param ids - Array of report IDs
	 * @returns Promise resolving to an array of reports
	 *
	 * @example
	 * ```typescript
	 * const reports = await client.labrinth.reports_v3.getMultiple(['id1', 'id2'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Reports.v3.Report[]> {
		return this.client.request<Labrinth.Reports.v3.Report[]>(
			`/reports?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Create a new report
	 *
	 * @param data - Report creation data
	 * @returns Promise resolving to the created report
	 *
	 * @example
	 * ```typescript
	 * const report = await client.labrinth.reports_v3.create({
	 *   report_type: 'spam',
	 *   item_id: 'project123',
	 *   item_type: 'project',
	 *   body: 'This project is spam',
	 * })
	 * ```
	 */
	public async create(
		data: Labrinth.Reports.v3.CreateReportRequest,
	): Promise<Labrinth.Reports.v3.Report> {
		return this.client.request<Labrinth.Reports.v3.Report>(`/report`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Edit a report
	 *
	 * @param id - Report ID
	 * @param data - Report edit data
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.reports_v3.edit('abc123', { closed: true })
	 * ```
	 */
	public async edit(id: string, data: Labrinth.Reports.v3.EditReportRequest): Promise<void> {
		return this.client.request(`/report/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a report (moderator only)
	 *
	 * @param id - Report ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.reports_v3.delete('abc123')
	 * ```
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/report/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
