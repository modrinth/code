import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthReviewsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_reviews_v3'
	}

	public async create(
		data: Labrinth.Reviews.v3.CreateReviewRequest,
	): Promise<Labrinth.Reviews.v3.Review> {
		return this.client.request<Labrinth.Reviews.v3.Review>(`/review`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}

	public async list(
		projectId: string,
		params?: Labrinth.Reviews.v3.ListReviewsParams,
	): Promise<Labrinth.Reviews.v3.ReviewsResponse> {
		const queryParams: Record<string, string> = { project_id: projectId }
		if (params?.count != null) queryParams.count = String(params.count)
		if (params?.offset != null) queryParams.offset = String(params.offset)

		return this.client.request<Labrinth.Reviews.v3.ReviewsResponse>(`/reviews`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: queryParams,
		})
	}

	public async get(id: string): Promise<Labrinth.Reviews.v3.Review> {
		return this.client.request<Labrinth.Reviews.v3.Review>(`/review/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	public async getOwnForProject(projectId: string): Promise<Labrinth.Reviews.v3.Review> {
		return this.client.request<Labrinth.Reviews.v3.Review>(`/review/project/${projectId}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	public async edit(id: string, data: Labrinth.Reviews.v3.EditReviewRequest): Promise<void> {
		return this.client.request(`/review/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	public async delete(id: string): Promise<void> {
		return this.client.request(`/review/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
