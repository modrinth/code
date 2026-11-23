export interface GameAwardsNominee {
	projectName: string
	year: number
	votingDeadline: Date
	link: string
}

const CREATE_AERONAUTICS_PROJECT_ID = 'oWaK0Q19'

export function getGameAwardsNominee(
	project: { id?: string | null; title?: string | null } | null | undefined,
): GameAwardsNominee | null {
	if (project?.id === CREATE_AERONAUTICS_PROJECT_ID && project.title) {
		return {
			projectName: project.title,
			year: 2026,
			votingDeadline: new Date(2026, 11, 10),
			link: 'https://modrinth.com/mod/create-aeronautics/vote',
		}
	}

	return null
}
