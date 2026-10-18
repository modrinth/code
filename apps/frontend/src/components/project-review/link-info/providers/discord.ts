import { detail, type Profile, type Provider } from '../types'

interface DiscordUser {
	id: string
	username: string
	global_name?: string | null
	avatar?: string | null
	bot?: boolean
}
interface Invite {
	type?: number
	guild?: { id: string; name: string; icon?: string | null }
	channel?: { name?: string; type?: number }
	approximate_member_count?: number
	approximate_presence_count?: number
	inviter?: DiscordUser
	expires_at?: string | null
	uses?: number
	max_uses?: number
	target_type?: number
}

function userProfile(user: DiscordUser): Profile {
	return {
		title: user.global_name ? `${user.global_name} (@${user.username})` : user.username,
		label: 'inviter',
		avatar: user.avatar
			? `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64`
			: undefined,
		details: [
			...detail('bot', user.bot || undefined),
			...detail(
				'created',
				new Date(Number(BigInt(user.id) >> 22n) + 1420070400000).toISOString(),
				'date',
			),
		],
	}
}

export const discord: Provider = (url) => {
	if (
		![
			'discord.gg',
			'discord.com',
			'discordapp.com',
			'www.discord.com',
			'www.discordapp.com',
		].includes(url.hostname)
	)
		return
	const [path, id] = url.pathname.split('/').filter(Boolean)
	const code = url.hostname === 'discord.gg' ? path : path === 'invite' ? id : undefined
	if (code)
		return {
			platform: 'Discord',
			kind: 'invite',
			name: code,

			load: async (request) => {
				const data = await request<Invite>(
					`https://discord.com/api/v10/invites/${encodeURIComponent(code)}?with_counts=true&with_expiration=true`,
				)
				const guild = data.guild
				const profiles: Profile[] = [
					{
						title: guild?.name || code,
						avatar: guild?.icon
							? `https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=64`
							: undefined,
						details: [
							...detail('members', data.approximate_member_count),
							...detail('online', data.approximate_presence_count),
							...detail('channel', data.channel?.name),
							...detail(
								'expires',
								data.expires_at === null ? { message: 'noExpiry' } : data.expires_at,
								data.expires_at ? 'date' : undefined,
								data.expires_at ? 'inviteExpires' : undefined,
							),
							...detail('uses', data.uses),
							...detail(
								'maxUses',
								data.max_uses === 0 ? { message: 'unlimitedUses' } : data.max_uses,
							),
							...detail(
								'target',
								data.target_type === 1
									? { message: 'stream' }
									: data.target_type === 2
										? { message: 'activity' }
										: undefined,
							),
						],
					},
				]
				if (data.inviter) profiles.push(userProfile(data.inviter))
				return {
					profiles,
					warnings:
						data.expires_at && new Date(data.expires_at).getTime() <= Date.now()
							? ['inviteExpired']
							: [],
				}
			},
		}
	if (path === 'users' && /^\d+$/.test(id ?? ''))
		return {
			platform: 'Discord',
			kind: 'user',
			name: id,
		}
	if (path === 'channels' && id === '@me') return { platform: 'Discord', kind: 'dm' }
	const guildId =
		path === 'channels' ? id : path === 'servers' ? id?.match(/\d{17,20}$/)?.[0] : undefined
	if (guildId && /^\d+$/.test(guildId))
		return {
			platform: 'Discord',
			kind: path === 'channels' ? 'channel' : 'server',

			load: async (request) => {
				const data = await request<{ name: string; presence_count?: number }>(
					`https://discord.com/api/v10/guilds/${guildId}/widget.json`,
				)
				return {
					profiles: [{ title: data.name, details: detail('online', data.presence_count) }],
					warnings: [],
				}
			},
		}
	return { platform: 'Discord', kind: 'website' }
}
