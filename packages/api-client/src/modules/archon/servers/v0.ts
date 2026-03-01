import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import type { Archon } from '../types'

export class ArchonServersV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_v0'
	}

	/**
	 * Get a specific server by ID
	 * GET /modrinth/v0/servers/:id
	 */
	public async get(serverId: string): Promise<Archon.Servers.v0.Server> {
		return this.client.request<Archon.Servers.v0.Server>(`/servers/${serverId}`, {
			api: 'archon',
			method: 'GET',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Get list of servers for the authenticated user
	 * GET /modrinth/v0/servers
	 */
	public async list(
		options?: Archon.Servers.v0.GetServersOptions,
	): Promise<Archon.Servers.v0.ServerGetResponse> {
		const params = new URLSearchParams()
		if (options?.limit) params.set('limit', options.limit.toString())
		if (options?.offset) params.set('offset', options.offset.toString())

		const query = params.toString() ? `?${params.toString()}` : ''

		return this.client.request<Archon.Servers.v0.ServerGetResponse>(`servers${query}`, {
			api: 'archon',
			method: 'GET',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Check stock availability for a region
	 * POST /modrinth/v0/stock
	 */
	public async checkStock(
		region: string,
		request: Archon.Servers.v0.StockRequest,
	): Promise<Archon.Servers.v0.StockResponse> {
		return this.client.request<Archon.Servers.v0.StockResponse>(`/stock?region=${region}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: request,
		})
	}

	/**
	 * Get filesystem authentication credentials for a server
	 * Returns URL and JWT token for accessing the server's filesystem via Kyros
	 * GET /modrinth/v0/servers/:id/fs
	 */
	public async getFilesystemAuth(serverId: string): Promise<Archon.Servers.v0.JWTAuth> {
		return this.client.request<Archon.Servers.v0.JWTAuth>(`/servers/${serverId}/fs`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/**
	 * Get WebSocket authentication credentials for a server
	 * GET /modrinth/v0/servers/:id/ws
	 */
	public async getWebSocketAuth(serverId: string): Promise<Archon.Websocket.v0.WSAuth> {
		return this.client.request<Archon.Websocket.v0.WSAuth>(`/servers/${serverId}/ws`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/**
	 * Send a power action to a server (Start, Stop, Restart, Kill)
	 * POST /modrinth/v0/servers/:id/power
	 */
	public async power(
		serverId: string,
		action: 'Start' | 'Stop' | 'Restart' | 'Kill',
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/power`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			body: { action },
		})
	}

	/**
	 * Reinstall a server with a new loader or modpack
	 * POST /modrinth/v0/servers/:id/reinstall
	 */
	public async reinstall(
		serverId: string,
		request: Archon.Servers.v0.ReinstallRequest,
		hardReset: boolean = false,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/reinstall`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			params: { hard: String(hardReset) },
			body: request,
		})
	}

	/**
	 * Get authentication credentials for .mrpack file upload
	 * GET /modrinth/v0/servers/:id/reinstallFromMrpack
	 */
	public async getReinstallMrpackAuth(
		serverId: string,
	): Promise<Archon.Servers.v0.MrpackReinstallAuth> {
		return this.client.request<Archon.Servers.v0.MrpackReinstallAuth>(
			`/servers/${serverId}/reinstallFromMrpack`,
			{
				api: 'archon',
				version: 'modrinth/v0',
				method: 'GET',
			},
		)
	}

	/**
	 * Reinstall a server from a .mrpack file with progress tracking
	 *
	 * Two-step flow: fetches upload auth, then uploads the .mrpack file to the node.
	 *
	 * @param serverId - Server ID
	 * @param file - .mrpack file to upload
	 * @param hardReset - Whether to erase all server data
	 * @param options - Optional progress callback
	 * @returns Promise resolving to an UploadHandle with progress tracking and cancellation
	 */
	public async reinstallFromMrpack(
		serverId: string,
		file: File,
		hardReset: boolean = false,
		options?: {
			onProgress?: (progress: UploadProgress) => void
		},
	): Promise<UploadHandle<void>> {
		const auth = await this.getReinstallMrpackAuth(serverId)

		const formData = new FormData()
		formData.append('file', file)

		return this.client.upload<void>('', {
			api: `https://${auth.url}`,
			version: 'reinstallMrpackMultiparted',
			formData,
			params: { hard: String(hardReset) },
			headers: { Authorization: `Bearer ${auth.token}` },
			skipAuth: true,
			onProgress: options?.onProgress,
			retry: false,
		})
	}

	/**
	 * Update a server's name
	 * POST /modrinth/v0/servers/:id/name
	 */
	public async updateName(serverId: string, name: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/name`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			body: { name },
		})
	}

	/**
	 * Get allocations for a server
	 * GET /modrinth/v0/servers/:id/allocations
	 */
	public async getAllocations(serverId: string): Promise<Archon.Servers.v0.Allocation[]> {
		return this.client.request<Archon.Servers.v0.Allocation[]>(`/servers/${serverId}/allocations`, {
			api: 'archon',
			method: 'GET',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Reserve a new allocation for a server
	 * POST /modrinth/v0/servers/:id/allocations?name=...
	 */
	public async reserveAllocation(
		serverId: string,
		name: string,
	): Promise<Archon.Servers.v0.Allocation> {
		return this.client.request<Archon.Servers.v0.Allocation>(`/servers/${serverId}/allocations`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			params: { name },
		})
	}

	/**
	 * Update an allocation's name
	 * PUT /modrinth/v0/servers/:id/allocations/:port?name=...
	 */
	public async updateAllocation(serverId: string, port: number, name: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/allocations/${port}`, {
			api: 'archon',
			method: 'PUT',
			version: 'modrinth/v0',
			params: { name },
		})
	}

	/**
	 * Delete an allocation
	 * DELETE /modrinth/v0/servers/:id/allocations/:port
	 */
	public async deleteAllocation(serverId: string, port: number): Promise<void> {
		await this.client.request(`/servers/${serverId}/allocations/${port}`, {
			api: 'archon',
			method: 'DELETE',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Check if a subdomain is available
	 * GET /modrinth/v0/subdomains/:subdomain/isavailable
	 */
	public async checkSubdomainAvailability(subdomain: string): Promise<{ available: boolean }> {
		return this.client.request<{ available: boolean }>(`/subdomains/${subdomain}/isavailable`, {
			api: 'archon',
			method: 'GET',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Change a server's subdomain
	 * POST /modrinth/v0/servers/:id/subdomain
	 */
	public async changeSubdomain(serverId: string, subdomain: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/subdomain`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			body: { subdomain },
		})
	}

	/**
	 * Get startup configuration for a server
	 * GET /modrinth/v0/servers/:id/startup
	 */
	public async getStartupConfig(serverId: string): Promise<Archon.Servers.v0.StartupConfig> {
		return this.client.request<Archon.Servers.v0.StartupConfig>(`/servers/${serverId}/startup`, {
			api: 'archon',
			method: 'GET',
			version: 'modrinth/v0',
		})
	}

	/**
	 * Update startup configuration for a server
	 * POST /modrinth/v0/servers/:id/startup
	 */
	public async updateStartupConfig(
		serverId: string,
		config: {
			invocation: string | null
			jdk_version: string | null
			jdk_build: string | null
		},
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/startup`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
			body: config,
		})
	}

	/**
	 * Dismiss a server notice
	 * POST /modrinth/v0/servers/:id/notices/:noticeId/dismiss
	 */
	public async dismissNotice(serverId: string, noticeId: number): Promise<void> {
		await this.client.request(`/servers/${serverId}/notices/${noticeId}/dismiss`, {
			api: 'archon',
			method: 'POST',
			version: 'modrinth/v0',
		})
	}

}
