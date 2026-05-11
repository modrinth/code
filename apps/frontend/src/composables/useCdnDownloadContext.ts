import type { FilterValue } from '@modrinth/ui'
import { LOADER_FILTER_TYPES } from '@modrinth/ui'

const TEN_MINUTES = 600

export type DownloadContext = {
	gameVersion?: string
	loader?: string
	reason?: 'standalone' | 'dependency' | 'modpack' | 'update'
}

export type FilterSelection = {
	gameVersion?: string
	loader?: string
}

const cookieDefaults = {
	maxAge: TEN_MINUTES,
	sameSite: 'lax' as const,
	secure: true,
	path: '/',
	httpOnly: false,
}

function readCookieValue(value: string | null | undefined): string | undefined {
	if (typeof value !== 'string' || !value) {
		return undefined
	}
	return value
}

function newFilterSelection(
	gameVersion: string | undefined,
	loader: string | undefined,
): FilterSelection | null {
	if (!gameVersion && !loader) {
		return null
	} else if (!gameVersion) {
		return {
			loader,
		}
	} else if (!loader) {
		return {
			gameVersion,
		}
	} else {
		return {
			gameVersion,
			loader,
		}
	}
}

export function useCdnDownloadContext() {
	const filterGameVersionCookie = useCookie<string | null>('mr_download_filter_game_version', {
		...cookieDefaults,
		default: () => null,
	})

	const filterLoaderCookie = useCookie<string | null>('mr_download_filter_loader', {
		...cookieDefaults,
		default: () => null,
	})

	function createProjectDownloadUrl(originalUrl: string, context?: DownloadContext): string {
		if (!originalUrl.startsWith('https://cdn.modrinth.com')) {
			return originalUrl
		}

		const reason = context?.reason
		const gameVersion = context?.gameVersion ?? readCookieValue(filterGameVersionCookie.value)
		const loader = context?.loader ?? readCookieValue(filterLoaderCookie.value)

		try {
			const url = new URL(originalUrl)

			if (reason) {
				url.searchParams.set('mr_download_reason', reason)
			}
			if (gameVersion) {
				url.searchParams.set('mr_game_version', gameVersion)
			} else {
				url.searchParams.delete('mr_game_version')
			}

			if (loader) {
				url.searchParams.set('mr_loader', loader)
			} else {
				url.searchParams.delete('mr_loader')
			}
			return url.toString()
		} catch {
			return originalUrl
		}
	}

	function persistFilterSelection(selection: FilterSelection | null) {
		if (!selection) {
			filterGameVersionCookie.value = null
			filterLoaderCookie.value = null
			return
		}
		filterGameVersionCookie.value = selection.gameVersion ?? null
		filterLoaderCookie.value = selection.loader ?? null
	}

	function updateDiscoverFilterContext(filters: FilterValue[]) {
		if (!import.meta.client) {
			return
		}
		const versionFilters = [
			...new Set(filters.filter((f) => f.type === 'game_version').map((f) => f.option)),
		]
		const loaderFilters = [
			...new Set(
				filters
					.filter((f) => (LOADER_FILTER_TYPES as readonly string[]).includes(f.type))
					.map((f) => f.option),
			),
		]
		const gameVersion = versionFilters.length === 1 ? versionFilters[0] : undefined
		const loader = loaderFilters.length === 1 ? loaderFilters[0] : undefined
		persistFilterSelection(newFilterSelection(gameVersion, loader))
	}

	function updateVersionsFilterContext(gameVersions: string[], loaders: string[]) {
		if (!import.meta.client) {
			return
		}
		const gameVersion = gameVersions.length === 1 ? gameVersions[0] : undefined
		const loader = loaders.length === 1 ? loaders[0] : undefined
		persistFilterSelection(newFilterSelection(gameVersion, loader))
	}

	return {
		createProjectDownloadUrl,
		updateDiscoverFilterContext,
		updateVersionsFilterContext,
	}
}
