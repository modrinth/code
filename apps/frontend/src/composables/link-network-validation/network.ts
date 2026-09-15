import { type AbstractModrinthClient, type Labrinth, ModrinthApiError } from '@modrinth/api-client'

import { checkDiscordInvite, discordInviteCode } from './discord.ts'
import { githubRepositoryPath, probeGithubRepository } from './github.ts'
import type { LinkTarget } from './targets.ts'

type ProjectNag = Labrinth.Projects.v3.ProjectNag

const sourceDomains = [
	'github.com',
	'gitlab.com',
	'bitbucket.org',
	'codeberg.org',
	'git.sr.ht',
	'tangled.org',
	'git.gay',
	'gitee.com',
]

interface Probe {
	url: string
	/** null means the request could not establish whether the link is accessible. */
	accessible: boolean | null
	contentType?: string
	disposition?: string
}

export function linkNag(target: LinkTarget, reason: string): ProjectNag {
	return {
		kind: 'link_validation',
		severity: 'required',
		details: { field: target.field, url: target.url, reason },
	}
}

export function fetchable(value: string): boolean {
	try {
		const url = new URL(value)
		const host = url.hostname.replace(/\.$/, '')
		return (
			url.protocol === 'https:' &&
			!url.username &&
			!url.password &&
			!url.port &&
			host.includes('.') &&
			!/^[\d.]+$/.test(host) &&
			!host.includes(':') &&
			!['localhost', 'local', 'internal', 'test', 'invalid'].some(
				(suffix) => host === suffix || host.endsWith(`.${suffix}`),
			)
		)
	} catch {
		return false
	}
}

/** Document probes read response headers and the final URL. */
async function probe(url: string, signal: AbortSignal): Promise<Probe> {
	const unavailable = {
		url,
		accessible: null,
		contentType: '',
		disposition: '',
	}
	if (!fetchable(url)) return unavailable
	try {
		const options: RequestInit = {
			signal: AbortSignal.any([signal, AbortSignal.timeout(5000)]),
			credentials: 'omit',
			referrerPolicy: 'no-referrer',
			mode: 'cors',
		}
		let response = await fetch(url, { ...options, method: 'HEAD' })
		if (response.status === 405 || response.status === 501) {
			response = await fetch(url, { ...options, method: 'GET' })
		}
		const result = {
			url: response.url || url,
			accessible: response.ok ? true : response.status >= 400 && response.status < 500 ? false : null,
			contentType: response.headers.get('content-type') ?? '',
			disposition: response.headers.get('content-disposition') ?? '',
		}
		await response.body?.cancel()
		return result
	} catch {
		signal.throwIfAborted()
		return unavailable
	}
}

export function descriptionResponseReason(
	image: boolean,
	contentType: string,
	disposition: string,
): 'download' | 'unverifiable' | null {
	const mime = contentType.split(';')[0].trim().toLowerCase()
	if (image && ['', 'application/octet-stream', 'text/html'].includes(mime)) return 'unverifiable'
	if (image && mime.startsWith('image/')) return null
	return disposition.split(';')[0].trim().toLowerCase() === 'attachment' ||
		(mime !== '' && !['text/html', 'application/xhtml+xml'].includes(mime))
		? 'download'
		: null
}

export function repositoryApi(value: string): URL | null {
	const url = new URL(value)
	const parts = url.pathname.split('/').filter(Boolean)
	if (parts.length < 2) return null
	url.pathname = `/api/v1/repos/${parts[0]}/${parts[1].replace(/\.git$/, '')}`
	url.search = ''
	url.hash = ''
	return url
}

async function checkRepository(
	client: AbstractModrinthClient,
	target: LinkTarget,
	signal: AbortSignal,
): Promise<ProjectNag[]> {
	if (['description', 'discord'].includes(target.field) || !fetchable(target.url)) return []
	const host = new URL(target.url).hostname.replace(/\.$/, '')
	if (sourceDomains.some((domain) => host === domain || host.endsWith(`.${domain}`))) return []
	const api = repositoryApi(target.url)
	if (!api) return target.field === 'source' ? [linkNag(target, 'source_repository')] : []
	try {
		const repository = await client.request<Record<string, unknown> | null>(
			api.pathname.replace('/api/v1', ''),
			{
				api: `${api.origin}/api`,
				version: 1,
				skipAuth: true,
				headers: { 'Content-Type': '', Accept: 'application/json' },
				retry: false,
				timeout: 5000,
				signal,
			},
		)
		if (typeof repository !== 'object' || repository === null) {
			return target.field === 'source' ? [linkNag(target, 'unverifiable')] : []
		}
		const isRepository =
			typeof repository?.full_name === 'string' && typeof repository?.clone_url === 'string'
		if (target.field === 'source') {
			return isRepository && repository?.private === false
				? []
				: [linkNag(target, 'source_repository')]
		}
		const parts = new URL(target.url).pathname.split('/').filter(Boolean)
		const matchingSection = ['wiki', 'issues'].includes(target.field) && parts[2] === target.field
		return isRepository && !matchingSection ? [linkNag(target, 'wrong_field')] : []
	} catch (error) {
		signal.throwIfAborted()
		if (
			target.field !== 'source' ||
			!(error instanceof ModrinthApiError) ||
			error.statusCode === undefined ||
			error.statusCode < 400 ||
			error.statusCode >= 500
		)
			return []
		return [linkNag(target, error.statusCode === 404 ? 'source_repository' : 'unverifiable')]
	}
}

async function checkInvite(
	client: AbstractModrinthClient,
	target: LinkTarget,
	signal: AbortSignal,
): Promise<ProjectNag[]> {
	let code = discordInviteCode(target.url)
	if (!code) {
		const observed = await probe(target.url, signal)
		if (observed.accessible === null) return []
		if (!observed.accessible || !fetchable(observed.url)) {
			return [linkNag(target, 'unverifiable')]
		}
		code = discordInviteCode(observed.url)
		if (!code) return []
	}
	try {
		return (await checkDiscordInvite(client, code, signal))
			? [linkNag(target, 'discord_invite')]
			: []
	} catch (error) {
		signal.throwIfAborted()
		return error instanceof ModrinthApiError &&
			error.statusCode !== undefined &&
			error.statusCode >= 400 &&
			error.statusCode < 500
			? [linkNag(target, 'unverifiable')]
			: []
	}
}

export async function mapConcurrent<T, R>(items: T[], run: (item: T) => Promise<R>): Promise<R[]> {
	let next = 0
	const results = new Array<R>(items.length)
	await Promise.all(
		Array.from({ length: Math.min(8, items.length) }, async () => {
			while (next < items.length) {
				const index = next++
				results[index] = await run(items[index])
			}
		}),
	)
	return results
}

export async function validateLinkNetwork(
	client: AbstractModrinthClient,
	targets: LinkTarget[],
	signal: AbortSignal,
): Promise<ProjectNag[]> {
	const deadline = AbortSignal.any([signal, AbortSignal.timeout(25_000)])
	return (
		await mapConcurrent(targets, async (target) => {
			try {
				deadline.throwIfAborted()
				if (target.field === 'discord') {
					return await checkInvite(client, target, deadline)
				}
				const githubRepository = !target.image && githubRepositoryPath(target.url, true)
				const observed: Probe = githubRepository
					? await probeGithubRepository(client, target.url, githubRepository, deadline)
					: await probe(target.url, deadline)
				if (observed.accessible === null) return []
				let destination = target
				if (observed.url !== target.url) {
					const originalUrl = new URL(target.url)
					const finalUrl = new URL(observed.url)
					if (
						!fetchable(observed.url) ||
						(!githubRepository && originalUrl.hostname !== finalUrl.hostname) ||
						[
							'/login',
							'/signin',
							'/sign-in',
							'/users/sign_in',
							'/user/login',
							'/session/new',
						].includes(finalUrl.pathname.replace(/\/$/, ''))
					)
						return [linkNag(target, 'unverifiable')]
					destination = { ...target, url: observed.url }
				}
				if (!observed.accessible) {
					const repositoryNags =
						target.field === 'source' && fetchable(target.url)
							? await checkRepository(client, target, deadline)
							: []
					return repositoryNags.length ? repositoryNags : [linkNag(target, 'unverifiable')]
				}
				if (target.field === 'description' && observed.contentType !== undefined) {
					const reason = descriptionResponseReason(
						target.image,
						observed.contentType,
						observed.disposition ?? '',
					)
					return reason ? [linkNag(target, reason)] : []
				}
				const nags = await checkRepository(client, destination, deadline)
				return nags.map((nag) => ({
					...nag,
					details: { ...nag.details, url: target.url },
				}))
			} catch {
				signal.throwIfAborted()
				return []
			}
		})
	).flat()
}
