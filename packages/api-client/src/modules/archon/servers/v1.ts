import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServersV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_v1'
	}

	/**
	 * Get list of servers for the authenticated user
	 * GET /v1/servers
	 */
	public async list(): Promise<Archon.Servers.v1.ServerFull[]> {
		return this.client.request<Archon.Servers.v1.ServerFull[]>('/servers', {
			api: 'archon',
			version: 1,
			method: 'GET',
		})
	}

	/**
	 * Get full server details including worlds, backups, and content
	 * GET /v1/servers/:server_id
	 */
	public async get(serverId: string): Promise<Archon.Servers.v1.ServerFull> {
		return this.client.request<Archon.Servers.v1.ServerFull>(`/servers/${serverId}`, {
			api: 'archon',
			version: 1,
			method: 'GET',
		})
	}

	/**
	 * Get available regions
	 * GET /v1/regions
	 */
	public async getRegions(): Promise<Archon.Servers.v1.Region[]> {
		return this.client.request<Archon.Servers.v1.Region[]>('/regions', {
			api: 'archon',
			version: 1,
			method: 'GET',
			skipAuth: true,
		})
	}

	/**
	 * Create a world
	 * POST /v1/servers/:id/worlds
	 */
	public async createWorld(
		serverId: string,
		request: Archon.Servers.v1.CreateWorld,
	): Promise<Archon.Servers.v1.CreateWorldResponse> {
		return this.client.request<Archon.Servers.v1.CreateWorldResponse>(
			`/servers/${serverId}/worlds`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
				body: request,
			},
		)
	}

	/**
	 * Modify a world
	 * PATCH /v1/servers/:id/worlds/:wid
	 */
	public async patchWorld(
		serverId: string,
		worldId: string,
		request: Archon.Servers.v1.PatchWorld,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/worlds/${worldId}`, {
			api: 'archon',
			version: 1,
			method: 'PATCH',
			body: request,
		})
	}

	/**
	 * End the intro flow for a server
	 * DELETE /v1/servers/:id/flows/intro
	 */
	public async endIntro(serverId: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/flows/intro`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}

	/**
	 * Run a power action for a specific world
	 * POST /v1/servers/:id/worlds/:wid/power
	 */
	public async powerWorld(
		serverId: string,
		worldId: string,
		request: Archon.Servers.v1.WorldPowerActionRequest,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/worlds/${worldId}/power`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/**
	 * Reset a world to onboarding
	 * POST /v1/servers/:id/worlds/:wid/onboard
	 */
	public async resetToOnboarding(serverId: string, worldId: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/worlds/${worldId}/onboard`, {
			api: 'archon',
			version: 1,
			method: 'POST',
		})
	}
}
