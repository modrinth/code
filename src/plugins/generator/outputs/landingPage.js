import { fetch } from '../fetch.js'
import { promises as fs } from 'fs'
import cliProgress from 'cli-progress'

export async function landingPage() {
	const progressBar = new cliProgress.SingleBar({
		format: 'Generating landing page   | {bar} | {percentage}%',
		barCompleteChar: '\u2588',
		barIncompleteChar: '\u2591',
		hideCursor: true,
	})
	progressBar.start(111, 0)

	/* MOD STACKS */
	// Fetch top 100 mods
	const mods = await (await fetch('search?limit=100&facets=[["project_type:mod"]]')).json()

	// Simplified array with the format: ['id', 'slug', 'icon_extension']
	const compressedMods = mods.hits
		.filter((project) => project.icon_url)
		.map((project) => {
			progressBar.increment()
			return [
				project.project_id,
				project.slug || '',
				project.icon_url.match(/\.[0-9a-z]+$/i)[0].substring(1),
			]
		})

	/* STATISTICS */
	const statistics = {
		downloads: 0,
		projects: 0,
		authors: 0,
	}

	// Get total number of projects
	const projectCount = (await (await fetch('search?limit=0')).json()).total_hits
	statistics.projects = projectCount
	progressBar.increment()

	const authors = new Set()

	// Number of pages through search to fetch
	const requestCount = Math.ceil(projectCount / 100)
	await Promise.allSettled(
		Array.from({ length: requestCount }, async (_, index) => {
			const response = await fetch(`search?limit=100&offset=${index * 100}`)
			if (!response.ok) {
				throw new Error(`Failed to fetch projects: ${response.statusText}`)
			}
			// Get project hits & use map to get rid of extra data
			const { hits } = await response.json()

			for (const hit of hits) {
				authors.add(hit.author)
				statistics.downloads += hit.downloads
			}
		})
	)

	statistics.authors = authors.size
	progressBar.increment()

	/* CONTRIBUTORS */
	const contributorCounts = new Map()

	const repoNames = [
		'knossos',
		'labrinth',
		'theseus',
		'minotaur',
		'hydra',
		'daedalus',
		'omorphia',
		'sisyphus',
		'ariadne',
	]

	const repos = await Promise.all(
		repoNames.map(async (repo) => {
			const response = await fetch(`https://api.github.com/repos/modrinth/${repo}/contributors`)

			if (!response.ok) {
				console.error(await response.json())
				throw new Error('Could not fetch repository from GitHub')
			}

			progressBar.increment()

			return await response.json()
		})
	)

	for (const repo of repos) {
		for (const user of repo) {
			if (!user.login.includes('[bot]')) {
				contributorCounts.set(user.login, {
					avatar_url: user.avatar_url,
					contributions:
						(contributorCounts.get(user.login)?.contributions || 0) + user.contributions,
				})
			}
		}
	}

	const contributors = Array.from(contributorCounts, ([name, data]) => ({ name, ...data })).sort(
		(a, b) => b.contributions - a.contributions
	)

	// Write JSON file
	await fs.writeFile(
		'./generated/landingPage.json',
		JSON.stringify({
			mods: compressedMods,
			statistics,
			contributors,
		})
	)
	progressBar.stop()
}
