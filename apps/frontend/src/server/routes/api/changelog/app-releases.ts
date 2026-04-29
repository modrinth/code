const CACHE_MAX_AGE = 60 * 10
const GITHUB_RELEASES_URL = 'https://api.github.com/repos/modrinth/code/releases?per_page=100'

interface GitHubRelease {
	tag_name: string
	html_url: string
	published_at: string | null
	created_at: string
}

export interface AppRelease {
	version: string
	publishedAt: string
	url: string
}

export default defineCachedEventHandler(
	async (): Promise<AppRelease[]> => {
		const response = await fetch(GITHUB_RELEASES_URL, {
			headers: {
				Accept: 'application/vnd.github+json',
				'User-Agent': 'modrinth-changelog',
			},
		})

		if (!response.ok) {
			throw createError({
				statusCode: 502,
				message: `GitHub releases request failed with ${response.status}`,
			})
		}

		const releases = (await response.json()) as GitHubRelease[]
		if (!Array.isArray(releases)) {
			throw createError({ statusCode: 502, message: 'Invalid GitHub releases response' })
		}

		return releases
			.filter((release) => release.tag_name.startsWith('v'))
			.map((release) => ({
				version: release.tag_name.replace(/^v/, ''),
				publishedAt: release.published_at ?? release.created_at,
				url: release.html_url,
			}))
	},
	{
		maxAge: CACHE_MAX_AGE,
		name: 'changelog-app-releases',
		getKey: () => 'changelog-app-releases',
	},
)
