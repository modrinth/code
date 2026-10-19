import { detail, type LinkInfo, type Profile, type Provider } from '../types'

interface User {
	login: string
	name?: string | null
	avatar_url?: string
	bio?: string | null
	company?: string | null
	followers?: number
	public_repos?: number
	type?: string
}
interface Repository {
	full_name: string
	owner: User
	description?: string | null
	stargazers_count?: number
	forks_count?: number
	size?: number
	language?: string | null
	license?: { spdx_id: string } | null
	pushed_at?: string
	archived?: boolean
	has_issues?: boolean
	has_wiki?: boolean
	has_discussions?: boolean
	parent?: { full_name: string }
}

function profile(user: User): Profile {
	return {
		title: user.name || user.login,
		url: `https://github.com/${encodeURIComponent(user.login)}`,
		avatar: user.avatar_url,
		description: user.bio,
		details: [
			...detail('status', user.type),
			...detail('followers', user.followers),
			...detail('repositories', user.public_repos),
			...detail('company', user.company),
		],
	}
}

export const github: Provider = (url) => {
	if (!['github.com', 'www.github.com'].includes(url.hostname)) return
	const [owner, rawRepo, page] = url.pathname.split('/').filter(Boolean)
	if (!owner) return { platform: 'GitHub', kind: 'website' }
	if (!rawRepo || owner === 'sponsors') {
		const username = owner === 'sponsors' ? rawRepo : owner
		if (!username) return { platform: 'GitHub', kind: 'sponsors' }
		return {
			platform: 'GitHub',
			kind: owner === 'sponsors' ? 'sponsors' : 'user',
			name: username,

			load: async (request) => ({
				profiles: [
					profile(
						await request<User>(`https://api.github.com/users/${encodeURIComponent(username)}`),
					),
				],
				warnings: [],
			}),
		}
	}
	const repo = rawRepo.replace(/\.git$/, '')
	const path = `${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`
	const kind = page === 'issues' || page === 'wiki' || page === 'discussions' ? page : 'repository'
	return {
		platform: 'GitHub',
		kind,
		name: `${owner}/${repo}`,

		load: async (request) => {
			const data = await request<Repository>(`https://api.github.com/repos/${path}`)
			const result: LinkInfo = {
				profiles: [
					{
						title: data.full_name,
						url: `https://github.com/${path}`,
						avatar: data.owner.avatar_url,
						description: data.description,
						details: [
							...detail('stars', data.stargazers_count),
							...detail('forks', data.forks_count),
							...detail('size', data.size == null ? undefined : data.size * 1024, 'bytes'),
							...detail('language', data.language),
							...detail(
								'license',
								data.license?.spdx_id === 'NOASSERTION' ? undefined : data.license?.spdx_id,
							),
							...detail('lastPush', data.pushed_at, 'date'),
							...detail('forkOf', data.parent?.full_name),
						],
					},
				],
				warnings: data.archived ? ['archived'] : [],
			}
			if (kind === 'issues' && data.has_issues === false) result.warnings.push('issuesDisabled')
			if (kind === 'wiki' && data.has_wiki === false) result.warnings.push('wikiDisabled')
			if (kind === 'discussions' && data.has_discussions === false)
				result.warnings.push('discussionsDisabled')
			const ownerProfile = request<User>(
				`https://api.github.com/users/${encodeURIComponent(data.owner.login)}`,
			)
				.then(profile)
				.catch(() => profile(data.owner))
			const counts =
				kind === 'issues' && data.has_issues && !data.archived
					? Promise.allSettled(
							['open', 'closed'].map(async (state) => {
								const query = encodeURIComponent(`repo:${owner}/${repo} is:issue state:${state}`)
								const count = await request<{ total_count: number }>(
									`https://api.github.com/search/issues?q=${query}&per_page=1`,
								)
								return detail(state === 'open' ? 'openIssues' : 'closedIssues', count.total_count)
							}),
						)
					: Promise.resolve([])
			const [ownerData, issueCounts] = await Promise.all([ownerProfile, counts])
			result.profiles.push(ownerData)
			for (const count of issueCounts) {
				if (count.status === 'fulfilled') result.profiles[0].details.push(...count.value)
			}
			return result
		},
	}
}
