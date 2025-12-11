import type { ISO3166, Labrinth } from '@modrinth/api-client'
import type { DisplayProjectType } from '@modrinth/utils'

import generatedState from '~/generated/state.json'
import type { DisplayMode } from '~/plugins/cosmetics'

export interface ProjectType {
	actual: string
	id: DisplayProjectType
	display: string
}

export interface LoaderData {
	pluginLoaders: string[]
	pluginPlatformLoaders: string[]
	allPluginLoaders: string[]
	dataPackLoaders: string[]
	modLoaders: string[]
	hiddenModLoaders: string[]
}

// Re-export types from api-client for convenience
export type Country = ISO3166.Country
export type Subdivision = ISO3166.Subdivision

export interface GeneratedState extends Labrinth.State.GeneratedState {
	// Additional runtime-defined fields not from the API
	projectTypes: ProjectType[]
	loaderData: LoaderData
	projectViewModes: DisplayMode[]
	approvedStatuses: string[]
	rejectedStatuses: string[]
	staffRoles: string[]

	// Metadata
	lastGenerated?: string
	apiUrl?: string
}

/**
 * Composable for accessing the complete generated state.
 * This includes both fetched data and runtime-defined constants.
 */
export const useGeneratedState = () =>
	useState<GeneratedState>('generatedState', () => ({
		// Cast JSON data to typed API responses
		categories: (generatedState.categories ?? []) as Labrinth.Tags.v2.Category[],
		loaders: (generatedState.loaders ?? []) as Labrinth.Tags.v2.Loader[],
		gameVersions: (generatedState.gameVersions ?? []) as Labrinth.Tags.v2.GameVersion[],
		donationPlatforms: (generatedState.donationPlatforms ??
			[]) as Labrinth.Tags.v2.DonationPlatform[],
		reportTypes: (generatedState.reportTypes ?? []) as string[],
		muralBankDetails: generatedState.muralBankDetails as
			| Record<string, { bankNames: string[] }>
			| undefined,
		countries: (generatedState.countries ?? []) as ISO3166.Country[],
		subdivisions: (generatedState.subdivisions ?? {}) as Record<string, ISO3166.Subdivision[]>,

		projectTypes: [
			{
				actual: 'mod',
				id: 'mod',
				display: 'mod',
			},
			{
				actual: 'mod',
				id: 'plugin',
				display: 'plugin',
			},
			{
				actual: 'mod',
				id: 'datapack',
				display: 'data pack',
			},
			{
				actual: 'shader',
				id: 'shader',
				display: 'shader',
			},
			{
				actual: 'resourcepack',
				id: 'resourcepack',
				display: 'resource pack',
			},
			{
				actual: 'modpack',
				id: 'modpack',
				display: 'modpack',
			},
		],
		loaderData: {
			pluginLoaders: ['bukkit', 'spigot', 'paper', 'purpur', 'sponge', 'folia'],
			pluginPlatformLoaders: ['bungeecord', 'waterfall', 'velocity'],
			allPluginLoaders: [
				'bukkit',
				'spigot',
				'paper',
				'purpur',
				'sponge',
				'bungeecord',
				'waterfall',
				'velocity',
				'folia',
			],
			dataPackLoaders: ['datapack'],
			modLoaders: ['forge', 'fabric', 'quilt', 'liteloader', 'modloader', 'rift', 'neoforge'],
			hiddenModLoaders: ['liteloader', 'modloader', 'rift'],
		},
		projectViewModes: ['list', 'grid', 'gallery'],
		approvedStatuses: ['approved', 'archived', 'unlisted', 'private'],
		rejectedStatuses: ['rejected', 'withheld'],
		staffRoles: ['moderator', 'admin'],

		homePageProjects: generatedState.homePageProjects as unknown as
			| Labrinth.Projects.v2.Project[]
			| undefined,
		homePageSearch: generatedState.homePageSearch as Labrinth.Search.v2.SearchResults | undefined,
		homePageNotifs: generatedState.homePageNotifs as Labrinth.Search.v2.SearchResults | undefined,
		products: generatedState.products as Labrinth.Billing.Internal.Product[] | undefined,

		lastGenerated: generatedState.lastGenerated,
		apiUrl: generatedState.apiUrl,
		errors: generatedState.errors,
	}))
