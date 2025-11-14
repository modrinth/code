import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthStateModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_state'
	}

	/**
	 * Build the complete generated state by fetching from multiple endpoints
	 *
	 * @returns Promise resolving to the generated state containing categories, loaders, products, etc.
	 *
	 * @example
	 * ```typescript
	 * const state = await client.labrinth.state.build()
	 * console.log(state.products) // Available billing products
	 * ```
	 */
	public async build(): Promise<Labrinth.State.GeneratedState> {
		const errors: unknown[] = []
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const handleError = (err: any, defaultValue: any) => {
			console.error('Error fetching state data:', err)
			errors.push(err)
			return defaultValue
		}

		// TODO: as we add new modules, move these raw requests to actual
		// abstractions
		const [
			categories,
			loaders,
			gameVersions,
			donationPlatforms,
			reportTypes,
			homePageProjects,
			homePageSearch,
			homePageNotifs,
			products,
			muralBankDetails,
			iso3166Data,
		] = await Promise.all([
			// Tag endpoints
			this.client
				.request<Labrinth.Tags.v2.Category[]>('/tag/category', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
				})
				.catch((err) => handleError(err, [])),
			this.client
				.request<Labrinth.Tags.v2.Loader[]>('/tag/loader', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
				})
				.catch((err) => handleError(err, [])),
			this.client
				.request<Labrinth.Tags.v2.GameVersion[]>('/tag/game_version', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
				})
				.catch((err) => handleError(err, [])),
			this.client
				.request<Labrinth.Tags.v2.DonationPlatform[]>('/tag/donation_platform', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
				})
				.catch((err) => handleError(err, [])),
			this.client
				.request<string[]>('/tag/report_type', { api: 'labrinth', version: 2, method: 'GET' })
				.catch((err) => handleError(err, [])),

			// Homepage data
			this.client
				.request<Labrinth.Projects.v2.Project[]>('/projects_random', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
					params: { count: '60' },
				})
				.catch((err) => handleError(err, [])),
			this.client
				.request<Labrinth.Search.v2.SearchResults>('/search', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
					params: { limit: '3', query: 'leave', index: 'relevance' },
				})
				.catch((err) => handleError(err, {} as Labrinth.Search.v2.SearchResults)),
			this.client
				.request<Labrinth.Search.v2.SearchResults>('/search', {
					api: 'labrinth',
					version: 2,
					method: 'GET',
					params: { limit: '3', query: '', index: 'updated' },
				})
				.catch((err) => handleError(err, {} as Labrinth.Search.v2.SearchResults)),

			// Internal billing/mural endpoints
			this.client.labrinth.billing_internal.getProducts().catch((err) => handleError(err, [])),
			this.client
				.request<{ bankDetails: Record<string, { bankNames: string[] }> }>('/mural/bank-details', {
					api: 'labrinth',
					version: 'internal',
					method: 'GET',
				})
				.catch((err) => handleError(err, null)),

			// ISO3166 country and subdivision data
			this.client.iso3166.data
				.build()
				.catch((err) => handleError(err, { countries: [], subdivisions: {} })),
		])

		return {
			categories,
			loaders,
			gameVersions,
			donationPlatforms,
			reportTypes,
			homePageProjects,
			homePageSearch,
			homePageNotifs,
			products,
			muralBankDetails: muralBankDetails?.bankDetails,
			countries: iso3166Data.countries,
			subdivisions: iso3166Data.subdivisions,
			errors,
		}
	}
}
