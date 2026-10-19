import { detail, type Provider } from '../types'

export const content: Provider = (url) => {
	const host = url.hostname.replace(/^www\./, '')
	const [type, id] = url.pathname.split('/').filter(Boolean)
	if (host === 'modrinth.com' && id) {
		if (type === 'user')
			return {
				platform: 'Modrinth',
				kind: 'user',
				name: id,
				load: async (_, client) => {
					const user = await client.labrinth.users_v2.get(encodeURIComponent(id))
					return {
						profiles: [
							{
								title: user.username,
								url: `https://modrinth.com/user/${encodeURIComponent(user.username)}`,
								avatar: user.avatar_url,
								details: detail('role', user.role === 'developer' ? undefined : user.role),
							},
						],
						warnings: [],
					}
				},
			}
		if (
			['mod', 'modpack', 'resourcepack', 'shader', 'plugin', 'datapack', 'project'].includes(type)
		)
			return {
				platform: 'Modrinth',
				kind: 'project',
				name: id,
				load: async (_, client) => {
					const project = await client.labrinth.projects_v2.get(encodeURIComponent(id))
					return {
						profiles: [
							{
								title: project.title,
								url: `https://modrinth.com/${project.project_type}/${encodeURIComponent(project.slug || project.id)}`,
								avatar: project.icon_url,
								description: project.description,
								details: [
									...detail('downloads', project.downloads),
									...detail('followers', project.followers),
									...detail('status', project.status),
								],
							},
						],
						warnings: ['rejected', 'withheld', 'draft', 'processing', 'archived'].includes(
							project.status,
						)
							? ['projectUnavailable']
							: [],
					}
				},
			}
	}
	if (host === 'youtu.be' || host === 'youtube.com') {
		const video =
			host === 'youtu.be'
				? type
				: type === 'watch'
					? url.searchParams.get('v')
					: ['shorts', 'embed', 'live'].includes(type)
						? id
						: undefined
		if (video)
			return {
				platform: 'YouTube',
				kind: 'video',
				load: async (request) => {
					const data = await request<{ title: string; author_name?: string }>(
						`https://www.youtube.com/oembed?url=${encodeURIComponent(`https://www.youtube.com/watch?v=${encodeURIComponent(video)}`)}&format=json`,
					)
					return {
						profiles: [
							{
								title: data.title,
								url: `https://www.youtube.com/watch?v=${encodeURIComponent(video)}`,
								details: detail('author', data.author_name),
							},
						],
						warnings: [],
					}
				},
			}
		return { platform: 'YouTube', kind: type === 'playlist' ? 'playlist' : 'channel' }
	}
	if (host === 'reddit.com' && id && ['r', 'u', 'user'].includes(type))
		return {
			platform: 'Reddit',
			kind: type === 'r' ? 'community' : 'user',
			name: `${type === 'r' ? 'r' : 'u'}/${id}`,
			load: async (request) => {
				const { data } = await request<{
					data: {
						display_name?: string
						name?: string
						title?: string
						subscribers?: number
						subreddit_type?: string
					}
				}>(
					`https://www.reddit.com/${type === 'r' ? 'r' : 'user'}/${encodeURIComponent(id)}/about.json`,
				)
				return {
					profiles: [
						{
							title: data.title || data.display_name || data.name || id,
							url: `https://www.reddit.com/${type === 'r' ? 'r' : 'user'}/${encodeURIComponent(id)}/`,
							details: detail('subscribers', data.subscribers),
						},
					],
					warnings: data.subreddit_type === 'private' ? ['privateCommunity'] : [],
				}
			},
		}
}
