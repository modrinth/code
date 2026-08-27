import type { LinkCheckResult, MessageDescriptor } from './types.ts'

export interface GitRepoFacts {
	empty?: boolean
	archived?: boolean
	issues?: boolean
	wiki?: boolean
}

function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return descriptor
}

function warn(message: MessageDescriptor): LinkCheckResult {
	return { severity: 'warn', message }
}

function error(message: MessageDescriptor): LinkCheckResult {
	return { severity: 'error', message }
}

export async function checkDiscordInvite(inviteCode: string): Promise<LinkCheckResult> {
	const response = await fetch(
		`https://discord.com/api/v10/invites/${inviteCode}?with_expiration=true`,
	)

	if (!response.ok) {
		return error(
			defineMessage({
				id: 'nags.link.discord.invite.invalid',
				defaultMessage: 'This Discord invite is invalid or has expired.',
			}),
		)
	}

	const invite = await response.json()
	if (!invite.guild) {
		return error(
			defineMessage({
				id: 'nags.link.discord.invite.not-guild',
				defaultMessage: 'This Discord invite does not lead to a server.',
			}),
		)
	}

	if (invite.expires_at) {
		return warn(
			defineMessage({
				id: 'nags.link.discord.invite.expires',
				defaultMessage: 'This Discord invite is set to expire',
			}),
		)
	}

	return { severity: 'valid' }
}

export async function fetchGiteaRepo(
	host: string,
	path: string,
): Promise<GitRepoFacts | undefined> {
	const response = await fetch(`https://${host}/api/v1/repos/${path}`)
	if (!response.ok) return undefined
	const data = await response.json()
	return {
		empty: data.size === 0,
		archived: data.archived,
		issues: data.has_issues,
		wiki: data.has_wiki,
	}
}

export async function fetchGitHubRepo(path: string): Promise<GitRepoFacts | undefined> {
	const response = await fetch(`https://api.github.com/repos/${path}`)
	if (!response.ok) return undefined
	const data = await response.json()
	return {
		empty: data.size === 0,
		archived: data.archived,
		issues: data.has_issues,
		wiki: data.has_wiki,
	}
}

export async function fetchGitLabRepo(path: string): Promise<GitRepoFacts | undefined> {
	const response = await fetch(`https://gitlab.com/api/v4/projects/${encodeURIComponent(path)}`)
	if (!response.ok) return undefined
	return {}
}

export async function fetchBitbucketRepo(path: string): Promise<GitRepoFacts | undefined> {
	const response = await fetch(`https://api.bitbucket.org/2.0/repositories/${path}`)
	if (!response.ok) return undefined
	const data = await response.json()
	return {
		empty: data.size === 0,
		issues: data.has_issues,
		wiki: data.has_wiki,
	}
}

export async function fetchGiteeRepo(path: string): Promise<GitRepoFacts | undefined> {
	const response = await fetch(`https://gitee.com/api/v5/repos/${path}`)
	if (!response.ok) return undefined
	const data = await response.json()
	return {
		issues: data.has_issues,
		wiki: data.has_wiki,
	}
}

export async function probeGiteaHost(hostname: string): Promise<boolean> {
	try {
		const response = await fetch(`https://${hostname}/api/v1/version`)
		return response.ok
	} catch {
		return false
	}
}
