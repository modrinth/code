import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'

export const PROJECT_REVIEW_VALIDATION_ERROR =
	'project must have no required validation nags before or during review or approval'

export function canSubmitProjectForReview(
	validation: Pick<Labrinth.Projects.v3.ProjectValidationResponse, 'nags'> | null | undefined,
	loading: boolean,
): boolean {
	return !loading && !!validation && !validation.nags.some((nag) => nag.severity === 'required')
}

export function canResubmitProjectForReview(
	thread: Pick<Labrinth.Threads.v3.Thread, 'issues'> | null | undefined,
): boolean {
	return !!thread && thread.issues.every((issue) => issue.verdict !== 'open')
}

export async function submitProjectForReview(
	projectId: string,
	client: Pick<AbstractModrinthClient, 'labrinth'>,
): Promise<Labrinth.Projects.v3.Project> {
	await client.labrinth.projects_v3.edit(projectId, { status: 'processing' })
	return await client.labrinth.projects_v3.get(projectId)
}
