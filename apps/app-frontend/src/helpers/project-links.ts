import type { LocationQuery, LocationQueryRaw } from 'vue-router'

const MODRINTH_HOSTNAMES = new Set(['modrinth.com', 'www.modrinth.com'])

const SUPPORTED_PROJECT_TYPES = new Set([
	'mod',
	'modpack',
	'resourcepack',
	'datapack',
	'plugin',
	'shader',
	'server',
	'project',
])

export function parseModrinthLink(
	href: string,
): { slug: string; pathSuffix: string; url: URL } | null {
	let url: URL
	try {
		url = new URL(href)
	} catch {
		return null
	}

	if (!MODRINTH_HOSTNAMES.has(url.hostname.toLowerCase())) {
		return null
	}

	const segments = url.pathname.split('/').filter((p) => p.length > 0)
	if (segments.length < 2) {
		return null
	}

	if (SUPPORTED_PROJECT_TYPES.has(segments[0].toLowerCase())) {
		const slug = segments[1]
		if (!slug) {
			return null
		}

		const rest: string[] = segments.slice(2)
		const pathSuffix = toValidAppSubpath(rest)
		if (pathSuffix === null) {
			return null
		}

		return { slug, pathSuffix, url }
	} else {
		return null
	}
}

const SUPPORTED_SUBPATHS = ['versions', 'gallery']

function toValidAppSubpath(rest: string[]): string | null {
	if (rest.length === 0) {
		return ''
	}

	const subroute = rest[0].toLowerCase()
	if (rest.length === 1 && SUPPORTED_SUBPATHS.includes(subroute)) {
		return `/${subroute}`
	}

	if (rest.length === 2 && subroute === 'version') {
		return `/version/${rest[1]}`
	}

	return null
}

export function mergeUrlQuery(routeQuery: LocationQuery, linkUrl: URL): LocationQueryRaw {
	const newQuery: LocationQueryRaw = { ...routeQuery }
	const keys = new Set<string>()
	linkUrl.searchParams.forEach((_value, key) => {
		keys.add(key)
	})
	for (const key of keys) {
		const values = linkUrl.searchParams.getAll(key)
		newQuery[key] = values.length === 1 ? values[0] : values
	}
	return newQuery
}
