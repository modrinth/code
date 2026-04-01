export type DisplayMode = 'list' | 'gallery' | 'grid'

export type DisplayLocation =
	| 'mod'
	| 'plugin'
	| 'resourcepack'
	| 'modpack'
	| 'shader'
	| 'datapack'
	| 'user'
	| 'collection'

export interface Cosmetics {
	rightSearchLayout: boolean
	leftContentLayout: boolean
	advancedRendering: boolean
	externalLinksNewTab: boolean
	notUsingBlockers: boolean
	hideModrinthAppPromos: boolean
	searchDisplayMode: Record<DisplayLocation, DisplayMode>
	hideStagingBanner: boolean
}

export default defineNuxtPlugin({
	name: 'cosmetics',
	setup() {
		const cosmetics = useCookie<Cosmetics>('cosmetics', {
			maxAge: 60 * 60 * 24 * 365 * 10,
			sameSite: 'lax',
			secure: true,
			httpOnly: false,
			path: '/',
			default: () => ({
				rightSearchLayout: false,
				leftContentLayout: false,
				advancedRendering: true,
				externalLinksNewTab: true,
				notUsingBlockers: false,
				hideModrinthAppPromos: false,
				searchDisplayMode: {
					mod: 'list',
					plugin: 'list',
					resourcepack: 'gallery',
					modpack: 'list',
					shader: 'gallery',
					datapack: 'list',
					server: 'list',
					user: 'list',
					collection: 'list',
				},
				hideStagingBanner: false,
			}),
		})

		return { provide: { cosmetics } }
	},
})
