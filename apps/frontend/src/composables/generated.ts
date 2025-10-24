import generatedState from '~/generated/state.json'

export interface ProjectType {
	actual: string
	id: string
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

export interface GeneratedState {
	categories: any[]
	loaders: any[]
	gameVersions: any[]
	donationPlatforms: any[]
	reportTypes: any[]
	muralBankDetails: Record<
		string,
		{
			bankNames: string[]
		}
	>

	projectTypes: ProjectType[]
	loaderData: LoaderData
	projectViewModes: string[]
	approvedStatuses: string[]
	rejectedStatuses: string[]
	staffRoles: string[]

	homePageProjects?: any[]
	homePageSearch?: any
	homePageNotifs?: any
	products?: any[]

	// Metadata
	lastGenerated?: string
	apiUrl?: string
	errors?: number[]
}

/**
 * Composable for accessing the complete generated state.
 * This includes both fetched data and runtime-defined constants.
 */
export const useGeneratedState = () =>
	useState<GeneratedState>('generatedState', () => ({
		categories: generatedState.categories ?? [],
		loaders: generatedState.loaders ?? [],
		gameVersions: generatedState.gameVersions ?? [],
		donationPlatforms: generatedState.donationPlatforms ?? [],
		reportTypes: generatedState.reportTypes ?? [],
		muralBankDetails: generatedState.muralBankDetails ?? null,

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

		homePageProjects: generatedState.homePageProjects,
		homePageSearch: generatedState.homePageSearch,
		homePageNotifs: generatedState.homePageNotifs,
		products: generatedState.products,

		lastGenerated: generatedState.lastGenerated,
		apiUrl: generatedState.apiUrl,
		errors: generatedState.errors,
	}))
