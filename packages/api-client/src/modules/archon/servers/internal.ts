import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServersInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_internal'
	}

	/**
	 * Get the server assigned to a subdomain.
	 * GET /_internal/servers/by-subdomain/:subdomain
	 */
	public async getBySubdomain(subdomain: string): Promise<Archon.Servers.Internal.Lookup> {
		return this.client.request<Archon.Servers.Internal.Lookup>(
			`/servers/by-subdomain/${encodeURIComponent(subdomain)}`,
			{
				api: 'archon',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * List all locked servers.
	 * GET /_internal/admin/server-locks
	 */
	public async getLocks(): Promise<Archon.Servers.Internal.ServerLock[]> {
		return this.client.request<Archon.Servers.Internal.ServerLock[]>('/admin/server-locks', {
			api: 'archon',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Get the lock on a server.
	 * GET /_internal/admin/server/:server_id/lock
	 */
	public async getLock(serverId: string): Promise<Archon.Servers.Internal.ServerLock> {
		return this.client.request<Archon.Servers.Internal.ServerLock>(
			`/admin/server/${serverId}/lock`,
			{
				api: 'archon',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Lock a server.
	 * PUT /_internal/admin/server/:server_id/lock
	 */
	public async lock(
		serverId: string,
		request: Archon.Servers.Internal.LockServerRequest,
	): Promise<void> {
		return this.client.request<void>(`/admin/server/${serverId}/lock`, {
			api: 'archon',
			version: 'internal',
			method: 'PUT',
			body: request,
		})
	}

	/**
	 * Unlock a server.
	 * DELETE /_internal/admin/server/:server_id/lock
	 */
	public async unlock(serverId: string): Promise<void> {
		return this.client.request<void>(`/admin/server/${serverId}/lock`, {
			api: 'archon',
			version: 'internal',
			method: 'DELETE',
		})
	}

	/**
	 * Temporarily lift a suspended server's suspension, or update its current temporary lift.
	 * POST /_internal/suspensions/server/:server_id/lift
	 */
	public async liftSuspension(
		serverId: string,
		request: Archon.Servers.Internal.LiftSuspensionRequest,
	): Promise<Archon.Servers.Internal.LiftSuspensionResponse> {
		return this.client.request<Archon.Servers.Internal.LiftSuspensionResponse>(
			`/suspensions/server/${serverId}/lift`,
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
				body: request,
			},
		)
	}

	/**
	 * End the temporary lift of a server's suspension before it expires.
	 * POST /_internal/suspensions/server/:server_id/resuspend
	 */
	public async resuspend(serverId: string): Promise<void> {
		return this.client.request<void>(`/suspensions/server/${serverId}/resuspend`, {
			api: 'archon',
			version: 'internal',
			method: 'POST',
		})
	}
}
