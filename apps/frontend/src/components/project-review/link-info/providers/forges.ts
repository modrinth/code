import { detail, type LinkInfo, type LinkMatch } from '../types'

interface ForgeUser {
	login?: string
	username?: string
	name?: string
	full_name?: string
	avatar_url?: string
	followers_count?: number
	bot?: boolean
}
interface ForgeRepository {
	full_name: string
	owner: ForgeUser
	description?: string
	archived?: boolean
	language?: string
	size?: number
	parent?: { full_name: string }
	has_issues?: boolean
	has_wiki?: boolean
}
interface GitLabRepository {
	path_with_namespace: string
	description?: string
	archived?: boolean
	namespace?: { name: string; avatar_url?: string }
	forked_from_project?: { path_with_namespace: string }
	topics?: string[]
	statistics?: { repository_size: number }
	issues_access_level?: string
	wiki_access_level?: string
}
interface BitbucketRepository {
	full_name: string
	description?: string
	language?: string
	size?: number
	owner: { display_name: string; links?: { avatar?: { href: string } } }
	parent?: { full_name: string }
	has_issues?: boolean
	has_wiki?: boolean
}

export function forges(url: URL, detectedPlatform?: string): LinkMatch | undefined {
	const platforms: Record<string, string> = {
		'gitlab.com': 'GitLab',
		'bitbucket.org': 'Bitbucket',
		'gitee.com': 'Gitee',
		'codeberg.org': 'Codeberg',
	}
	const platform = detectedPlatform ?? platforms[url.hostname.replace(/^www\./, '')]
	if (!platform) return
	const parts = url.pathname.split('/').filter(Boolean)
	const [owner, repo] = parts
	if (!owner) return { platform, kind: 'website' }
	if (!repo)
		return {
			platform,
			kind: 'user',
			name: owner,
			load: async (request) => {
				if (platform === 'Bitbucket') {
					const user = await request<{ name: string; links?: { avatar?: { href: string } } }>(
						`https://api.bitbucket.org/2.0/workspaces/${encodeURIComponent(owner)}`,
					)
					return {
						profiles: [
							{
								title: user.name || owner,
								url: `${url.origin}/${encodeURIComponent(owner)}`,
								avatar: user.links?.avatar?.href,
								details: [],
							},
						],
						warnings: [],
					}
				}
				const endpoint =
					platform === 'GitLab'
						? `${url.origin}/api/v4/users?username=${encodeURIComponent(owner)}`
						: `${url.origin}/api/${platform === 'Gitee' ? 'v5' : 'v1'}/users/${encodeURIComponent(owner)}`
				const response = await request<ForgeUser | ForgeUser[]>(endpoint)
				const user = Array.isArray(response) ? response[0] : response
				if (!user) throw new Error('User not found')
				return {
					profiles: [
						{
							title: user.full_name || user.name || user.login || owner,
							url: `${url.origin}/${encodeURIComponent(user.username || user.login || owner)}`,
							avatar: user.avatar_url,
							details: [...detail('followers', user.followers_count), ...detail('bot', user.bot)],
						},
					],
					warnings: [],
				}
			},
		}
	const separator = parts.indexOf('-')
	const repoParts =
		platform === 'GitLab' ? parts.slice(0, separator < 0 ? parts.length : separator) : [owner, repo]
	const repoPath = repoParts.join('/').replace(/\.git$/, '')
	const page = platform === 'GitLab' ? (separator < 0 ? undefined : parts[separator + 1]) : parts[2]
	const kind =
		page === 'issues' ? 'issues' : page === 'wiki' || page === 'wikis' ? 'wiki' : 'repository'
	return {
		platform,
		kind,
		name: repoPath,

		load: async (request) => {
			const result: LinkInfo = { profiles: [], warnings: [] }
			const encoded = repoPath.split('/').map(encodeURIComponent).join('/')
			if (platform === 'GitLab') {
				const data = await request<GitLabRepository>(
					`${url.origin}/api/v4/projects/${encodeURIComponent(repoPath)}?statistics=true`,
				)
				if (data.archived) result.warnings.push('archived')
				if (kind === 'issues' && data.issues_access_level === 'disabled')
					result.warnings.push('issuesDisabled')
				if (kind === 'wiki' && data.wiki_access_level === 'disabled')
					result.warnings.push('wikiDisabled')
				result.profiles.push({
					title: data.path_with_namespace,
					url: `${url.origin}/${encoded}`,
					avatar: data.namespace?.avatar_url,
					description: data.description,
					details: [
						...detail('owner', data.namespace?.name),
						...detail('forkOf', data.forked_from_project?.path_with_namespace),
						...detail('topic', data.topics?.[0]),
						...detail('size', data.statistics?.repository_size, 'bytes'),
					],
				})
			} else if (platform === 'Bitbucket') {
				const data = await request<BitbucketRepository>(
					`https://api.bitbucket.org/2.0/repositories/${encoded}`,
				)
				if (kind === 'issues' && data.has_issues === false) result.warnings.push('issuesDisabled')
				if (kind === 'wiki' && data.has_wiki === false) result.warnings.push('wikiDisabled')
				result.profiles.push({
					title: data.full_name,
					url: `${url.origin}/${encoded}`,
					avatar: data.owner.links?.avatar?.href,
					description: data.description,
					details: [
						...detail('owner', data.owner.display_name),
						...detail('language', data.language),
						...detail('size', data.size, 'bytes'),
						...detail('forkOf', data.parent?.full_name),
					],
				})
			} else {
				const api = `${url.origin}/api/${platform === 'Gitee' ? 'v5' : 'v1'}`
				const data = await request<ForgeRepository>(`${api}/repos/${encoded}`)
				if (data.archived) result.warnings.push('archived')
				if (kind === 'issues' && data.has_issues === false) result.warnings.push('issuesDisabled')
				if (kind === 'wiki' && data.has_wiki === false) result.warnings.push('wikiDisabled')
				result.profiles.push({
					title: data.full_name,
					url: `${url.origin}/${encoded}`,
					avatar: data.owner.avatar_url,
					description: data.description,
					details: [
						...detail('owner', data.owner.full_name || data.owner.name || data.owner.login),
						...detail('language', data.language),
						...detail('size', data.size == null ? undefined : data.size * 1024, 'bytes'),
						...detail('forkOf', data.parent?.full_name),
					],
				})
			}
			return result
		},
	}
}
