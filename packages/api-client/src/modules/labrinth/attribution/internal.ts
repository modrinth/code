import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

const BASE62_CHARS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'

/**
 * Decode a base62-encoded ID string into a number.
 * The backend serializes attribution group IDs as base62 strings in responses,
 * but the assign/update endpoints expect raw integer IDs in their request payloads.
 */
function decodeBase62Id(id: string): number {
	let value = 0
	for (const char of id) {
		const digit = BASE62_CHARS.indexOf(char)
		if (digit < 0) {
			throw new Error(`Invalid base62 character "${char}" in id "${id}"`)
		}
		value = value * 62 + digit
		if (!Number.isSafeInteger(value)) {
			throw new Error(`Base62 id "${id}" exceeds safe integer range`)
		}
	}
	return value
}

export class LabrinthAttributionInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_attribution_internal'
	}

	/**
	 * List attribution groups for a project
	 * GET /_internal/attribution/{project_id}
	 */
	public async listProjectAttribution(
		projectId: string,
	): Promise<Labrinth.Attribution.Internal.AttributionGroup[]> {
		return this.client.request<Labrinth.Attribution.Internal.AttributionGroup[]>(
			`/attribution/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Update an attribution group's attribution payload.
	 * PATCH /_internal/attribution/group/{group_id}
	 *
	 * @param groupId - The base62 attribution group id (as returned from listProjectAttribution).
	 */
	public async updateGroup(
		groupId: string,
		body: Labrinth.Attribution.Internal.UpdateGroupRequest,
	): Promise<void> {
		const numericId = decodeBase62Id(groupId)
		return this.client.request<void>(`/attribution/group/${numericId}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body,
		})
	}

	/**
	 * Reassign a file (by sha1) to another attribution group within the same project.
	 * POST /_internal/attribution/assign
	 *
	 * @param body.target_group_id - The base62 id of the attribution group to assign the file to.
	 */
	public async assignFileToGroup(body: {
		sha1: string
		target_group_id: string
		project_id: string
	}): Promise<void> {
		const wireBody: Labrinth.Attribution.Internal.AssignRequest = {
			sha1: body.sha1,
			target_group_id: decodeBase62Id(body.target_group_id),
			project_id: body.project_id,
		}
		return this.client.request<void>('/attribution/assign', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: wireBody,
		})
	}

	/**
	 * Split a file (by sha1) out of its current attribution group into a new group.
	 * POST /_internal/attribution/split
	 */
	public async splitFile(body: Labrinth.Attribution.Internal.SplitRequest): Promise<void> {
		return this.client.request<void>('/attribution/split', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body,
		})
	}
}
