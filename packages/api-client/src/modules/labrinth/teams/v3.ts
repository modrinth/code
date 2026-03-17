import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTeamsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_teams_v3'
	}

	/**
	 * Get multiple teams by their IDs
	 *
	 * @param ids - Array of team IDs
	 * @returns Promise resolving to an array of team member arrays (one per team)
	 *
	 * @example
	 * ```typescript
	 * const teams = await client.labrinth.teams_v3.getMultiple(['team1', 'team2'])
	 * // teams[0] = members of team1, teams[1] = members of team2
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Projects.v3.TeamMember[][]> {
		return this.client.request<Labrinth.Projects.v3.TeamMember[][]>(
			`/teams?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}
}
