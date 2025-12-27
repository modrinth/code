import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonSettingsV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_settings_v0'
	}

	/**
	 * Update server name
	 * POST /modrinth/v0/servers/:server_id/name
	 */
	async updateName(serverId: string, name: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/name`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: { name },
		})
	}

	/**
	 * Check if a subdomain is available
	 * GET /modrinth/v0/subdomains/:subdomain/isavailable
	 */
	async checkSubdomainAvailability(subdomain: string): Promise<boolean> {
		const res = await this.client.request<Archon.Settings.v0.SubdomainAvailability>(
			`/subdomains/${encodeURIComponent(subdomain)}/isavailable`,
			{
				api: 'archon',
				version: 'modrinth/v0',
				method: 'GET',
			},
		)
		return res.available
	}

	/**
	 * Update server subdomain
	 * POST /modrinth/v0/servers/:server_id/subdomain
	 */
	async updateSubdomain(serverId: string, subdomain: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/subdomain`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: { subdomain },
		})
	}

	/**
	 * Get startup settings (runtime options)
	 * GET /modrinth/v0/servers/:server_id/startup
	 */
	async getStartupSettings(serverId: string): Promise<Archon.Settings.v0.StartupSettings> {
		return this.client.request<Archon.Settings.v0.StartupSettings>(`/servers/${serverId}/startup`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/**
	 * Update startup settings (runtime options)
	 * POST /modrinth/v0/servers/:server_id/startup
	 */
	async updateStartupSettings(
		serverId: string,
		options: Archon.Settings.v0.PostStartupRequest,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/startup`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: options,
		})
	}

	/**
	 * Get all allocations for a server
	 * GET /modrinth/v0/servers/:server_id/allocations
	 */
	async getAllocations(serverId: string): Promise<Archon.Settings.v0.Allocation[]> {
		return this.client.request<Archon.Settings.v0.Allocation[]>(
			`/servers/${serverId}/allocations`,
			{
				api: 'archon',
				version: 'modrinth/v0',
				method: 'GET',
			},
		)
	}

	/**
	 * Reserve a new allocation
	 * POST /modrinth/v0/servers/:server_id/allocations?name=...
	 */
	async reserveAllocation(serverId: string, name: string): Promise<Archon.Settings.v0.Allocation> {
		return this.client.request<Archon.Settings.v0.Allocation>(
			`/servers/${serverId}/allocations?name=${encodeURIComponent(name)}`,
			{
				api: 'archon',
				version: 'modrinth/v0',
				method: 'POST',
			},
		)
	}

	/**
	 * Update an allocation's name
	 * PUT /modrinth/v0/servers/:server_id/allocations/:port?name=...
	 */
	async updateAllocation(serverId: string, port: number, name: string): Promise<void> {
		await this.client.request(
			`/servers/${serverId}/allocations/${port}?name=${encodeURIComponent(name)}`,
			{
				api: 'archon',
				version: 'modrinth/v0',
				method: 'PUT',
			},
		)
	}

	/**
	 * Delete an allocation
	 * DELETE /modrinth/v0/servers/:server_id/allocations/:port
	 */
	async deleteAllocation(serverId: string, port: number): Promise<void> {
		await this.client.request(`/servers/${serverId}/allocations/${port}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'DELETE',
		})
	}
}
