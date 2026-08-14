import type { ISO3166, Labrinth } from '@modrinth/api-client'
import type { DisplayProjectType } from '@modrinth/utils'

import {
	apiUrl,
	categories,
	donationPlatforms,
	errors,
	gameVersions,
	lastGenerated,
	loaders,
	reportTypes,
	taxComplianceThresholds,
} from '~/generated/state.json'
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

/**
 * Route-specific slices of the generated state are deliberately excluded here.
 * Anything placed on `useGeneratedState` is serialized into the SSR payload of
 * every page, so large fields must be imported directly by the pages that need
 * them instead (see `useCountries`, `useSubdivisions`, `~/generated/state.json`).
 */
type GloballyUsedState = Omit<
	Labrinth.State.GeneratedState,
	| 'countries'
	| 'subdivisions'
	| 'muralBankDetails'
	| 'tremendousIdMap'
	| 'homePageProjects'
	| 'homePageSearch'
	| 'homePageNotifs'
	| 'products'
>

export interface GeneratedState extends GloballyUsedState {
	// Additional runtime-defined fields not from the API
	projectTypes: ProjectType[]
	loaderData: LoaderData
	projectViewModes: DisplayMode[]
	approvedStatuses: string[]
	rejectedStatuses: string[]
	staffRoles: string[]
	taxComplianceThresholds?: Record<string, number>

	// Metadata
	lastGenerated?: string
	apiUrl?: string
	buildYear: number
}

/**
 * Built once per module load rather than via `useState`, because every `useState` value is
 * serialized into the SSR payload of every page. This is build-time constant data that the
 * client already has via the static import above, so putting it in the payload would ship a
 * second copy for no benefit.
 *
 * Consequence: on the server this object is shared by every request in the isolate, so it must
 * stay immutable there. Runtime refreshes are client-only and go through `setGameVersions`.
 */
const generatedState = shallowRef<GeneratedState>(
	Object.freeze({
		// Cast JSON data to typed API responses
		categories: (categories ?? []) as Labrinth.Tags.v2.Category[],
		loaders: (loaders ?? []) as Labrinth.Tags.v2.Loader[],
		gameVersions: (gameVersions ?? []) as Labrinth.Tags.v2.GameVersion[],
		donationPlatforms: (donationPlatforms ?? []) as Labrinth.Tags.v2.DonationPlatform[],
		reportTypes: (reportTypes ?? []) as string[],

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
			{
				actual: 'server',
				id: 'server',
				display: 'server',
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

		taxComplianceThresholds: (taxComplianceThresholds ?? {}) as Record<string, number>,

		lastGenerated,
		apiUrl,
		errors,

		buildYear: new Date().getFullYear(),
	}) as GeneratedState,
)

/**
 * Composable for accessing the globally used generated state.
 * This includes both fetched data and runtime-defined constants.
 */
export const useGeneratedState = () => generatedState

/**
 * Replaces the build-time game versions with a freshly fetched list. Client-only: mutating this
 * on the server would leak across every request sharing the isolate.
 */
export function setGameVersions(versions: Labrinth.Tags.v2.GameVersion[]) {
	if (import.meta.server) return

	generatedState.value = Object.freeze({
		...generatedState.value,
		gameVersions: versions,
	}) as GeneratedState
}
