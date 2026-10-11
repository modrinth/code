import { AbstractModule } from '../../../core/abstract-module'
import { ModrinthApiError } from '../../../core/errors'
import type { RequestOptions } from '../../../types/request'
import type { Archon } from '../types'

const SERVER_ID = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i

const PAIRING_MESSAGES: Record<Archon.Cores.v1.PairingFailure, string> = {
	wrong_code: 'Code invalid. Please check your Core and try again.',
	expired: 'That code has expired. Use the new code shown on your Core.',
	too_many_attempts: 'Too many wrong codes. Wait a few minutes and try again.',
	already_paired: 'That Core is already paired to another account.',
	unreachable: 'Could not reach that Core. Make sure it is running and on a network you can reach.',
	failed: 'Pairing failed. Please try again.',
}

/** Why pairing did not finish. `message` is ready to show. */
export class CorePairingError extends Error {
	constructor(
		public readonly reason: Archon.Cores.v1.PairingFailure,
		message = PAIRING_MESSAGES[reason],
	) {
		super(message)
		this.name = 'CorePairingError'
	}
}

/**
 * The account's paired Cores. The backend keeps the list and each Core's address; hosting calls go
 * to a Core directly. Several Cores are treated as one: their servers form one list, and a request
 * for a server is sent to the Core that has it.
 */
export class ArchonCoresV1Module extends AbstractModule {
	private cores: Promise<Archon.Cores.v1.Core[]> | null = null
	/** Addresses that may receive the account token: paired Cores and the one being paired. */
	private addresses = new Set<string>()
	/** Server id to the address of the Core that has it. */
	private serverAddresses = new Map<string, string>()

	public getModuleID(): string {
		return 'archon_cores_v1'
	}

	/** Fetch the account's paired Cores from the backend. */
	public list(): Promise<Archon.Cores.v1.Core[]> {
		const cores = this.client
			.request<Archon.Cores.v1.Core[]>('/cores', { api: 'labrinth', version: 3, method: 'GET' })
			.then((list) => {
				this.addresses = new Set(list.map((core) => core.address))
				return list
			})
		cores.catch(() => {
			if (this.cores === cores) this.cores = null
		})
		this.cores = cores
		return cores
	}

	/**
	 * Pair the Core showing `code` to the signed-in account. The backend says where that Core is,
	 * then the Core itself accepts the code and asks the backend to confirm the claim.
	 */
	public async pair(code: string): Promise<Archon.Cores.v1.Core> {
		let claim: { core_id: string; address: string }
		try {
			claim = await this.client.request('/cores/pair', {
				api: 'labrinth',
				version: 3,
				method: 'POST',
				body: { code },
				retry: false,
			})
		} catch (error) {
			throw pairingError(error, 'failed')
		}
		this.addresses.add(claim.address)
		try {
			await this.client.request('', {
				api: claim.address,
				version: 'pairing',
				method: 'POST',
				body: { code },
				retry: false,
				circuitBreaker: false,
			})
		} catch (error) {
			throw pairingError(error, 'unreachable')
		}
		const core = (await this.list()).find((paired) => paired.id === claim.core_id)
		if (!core) throw new CorePairingError('failed')
		return core
	}

	/**
	 * Unpair a Core. It leaves the account's list first, so this works even when the Core is off;
	 * a Core that missed it finds out the next time it starts. Its servers and files stay.
	 */
	public async unpair(core: Archon.Cores.v1.Core): Promise<void> {
		await this.client.request(`/cores/${encodeURIComponent(core.id)}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
		await this.client
			.request('', {
				api: core.address,
				version: 'pairing',
				method: 'DELETE',
				retry: false,
				circuitBreaker: false,
			})
			.catch(() => undefined)
		for (const [serverId, address] of this.serverAddresses) {
			if (address === core.address) this.serverAddresses.delete(serverId)
		}
		await this.list()
	}

	/** Whether `url` is on a paired Core, which is allowed to receive the account token. */
	public isCoreUrl(url: string): boolean {
		for (const address of this.addresses) {
			if (url.startsWith(address)) return true
		}
		return false
	}

	/** The hosting API base URL for a request: the Core that has the server the request names. */
	public async hostingUrl(path: string, params?: RequestOptions['params']): Promise<string> {
		const cores = await this.known()
		if (!cores.length) throw new ModrinthApiError('No Core is paired', { statusCode: 404 })
		const serverId = cores.length > 1 && `${path} ${JSON.stringify(params ?? {})}`.match(SERVER_ID)
		if (serverId) {
			if (!this.serverAddresses.has(serverId[0])) await this.servers()
			const address = this.serverAddresses.get(serverId[0])
			if (address) return `${address}/hosting`
		}
		return `${cores[0].address}/hosting`
	}

	/** The servers of every paired Core as one list. A Core that cannot be reached is left out. */
	public async servers(
		options?: Archon.Servers.v0.GetServersOptions,
	): Promise<Archon.Servers.v0.ServerGetResponse> {
		const cores = await this.known()
		const results = await Promise.allSettled(
			cores.map((core) =>
				this.client.request<Archon.Servers.v0.ServerGetResponse>('/servers', {
					api: `${core.address}/hosting`,
					version: 'modrinth/v0',
					method: 'GET',
					params: { limit: options?.limit ?? 100, offset: options?.offset ?? 0 },
				}),
			),
		)
		const merged: Archon.Servers.v0.ServerGetResponse = {
			servers: [],
			pagination: { current_page: 1, page_size: 0, total_pages: 1, total_items: 0 },
			users: {},
		}
		results.forEach((result, index) => {
			if (result.status !== 'fulfilled') return
			for (const server of result.value.servers) {
				this.serverAddresses.set(server.server_id, cores[index].address)
				merged.servers.push(server)
			}
			Object.assign(merged.users, result.value.users)
		})
		const failure = results.find((result) => result.status === 'rejected')
		if (failure && !results.some((result) => result.status === 'fulfilled')) throw failure.reason
		merged.pagination.page_size = merged.pagination.total_items = merged.servers.length
		return merged
	}

	/**
	 * Create an unconfigured server on a Core. Pass `instance` when it is created from an App
	 * instance: the server is linked right away and waits for the instance to be uploaded.
	 */
	public async createServer(
		name: string,
		core: Archon.Cores.v1.Core,
		instance?: { id: string; name: string },
	): Promise<{ id: string }> {
		const server = await this.client.request<{ id: string }>('/servers', {
			api: `${core.address}/hosting`,
			version: 'local',
			method: 'POST',
			body: { name, instance },
		})
		this.serverAddresses.set(server.id, core.address)
		return server
	}

	private known(): Promise<Archon.Cores.v1.Core[]> {
		return this.cores ?? this.list()
	}
}

function pairingError(error: unknown, fallback: Archon.Cores.v1.PairingFailure): CorePairingError {
	const status = error instanceof ModrinthApiError ? error.statusCode : undefined
	if (status === 404) return new CorePairingError('wrong_code')
	if (status === 410) return new CorePairingError('expired')
	if (status === 429) return new CorePairingError('too_many_attempts')
	if (status === 409 || status === 403) return new CorePairingError('already_paired')
	if (status === 401) return new CorePairingError('failed', 'Sign in to pair a Core.')
	if (status === undefined) return new CorePairingError(fallback)
	return new CorePairingError('failed')
}
