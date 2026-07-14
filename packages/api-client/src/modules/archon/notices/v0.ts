import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonNoticesV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_notices_v0'
	}

	/**
	 * Get all server notices.
	 * GET /modrinth/v0/notices
	 */
	public async list(): Promise<Archon.Notices.v0.ListedNotice[]> {
		return this.client.request<Archon.Notices.v0.ListedNotice[]>('/notices', {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/**
	 * Create a server notice.
	 * POST /modrinth/v0/notices
	 */
	public async create(
		request: Archon.Notices.v0.Announce,
	): Promise<Archon.Notices.v0.PostNoticeResponseBody> {
		return this.client.request<Archon.Notices.v0.PostNoticeResponseBody>('/notices', {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: request,
		})
	}

	/**
	 * Update a server notice.
	 * PATCH /modrinth/v0/notices/:id
	 */
	public async update(id: number, request: Archon.Notices.v0.AnnouncePatch): Promise<void> {
		await this.client.request(`/notices/${id}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'PATCH',
			body: request,
		})
	}

	/**
	 * Delete a server notice.
	 * DELETE /modrinth/v0/notices/:id
	 */
	public async delete(id: number): Promise<void> {
		await this.client.request(`/notices/${id}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'DELETE',
		})
	}

	/**
	 * Assign a notice to a server or node.
	 * PUT /modrinth/v0/notices/:id/assign?server=:serverId
	 * PUT /modrinth/v0/notices/:id/assign?node=:nodeId
	 */
	public async assign(id: number, target: Archon.Notices.v0.AssignmentTarget): Promise<void> {
		await this.client.request(`/notices/${id}/assign`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'PUT',
			params: this.assignmentTargetToParams(target),
		})
	}

	/**
	 * Unassign a notice from a server or node.
	 * PUT /modrinth/v0/notices/:id/unassign?server=:serverId
	 * PUT /modrinth/v0/notices/:id/unassign?node=:nodeId
	 */
	public async unassign(id: number, target: Archon.Notices.v0.AssignmentTarget): Promise<void> {
		await this.client.request(`/notices/${id}/unassign`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'PUT',
			params: this.assignmentTargetToParams(target),
		})
	}

	private assignmentTargetToParams(
		target: Archon.Notices.v0.AssignmentTarget,
	): Record<string, string> {
		if ('server' in target) {
			return { server: target.server }
		}

		return { node: target.node }
	}
}
