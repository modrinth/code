import type { AbstractModrinthClient } from '@modrinth/api-client'

import type { ModerationQueueService } from './queue.ts'

export interface QueueCandidateCheck {
	locked: boolean
	expired?: boolean
	isOwnLock?: boolean
	slug?: string
	projectType?: string
	status?: string
	isProcessing: boolean
}

export interface EligibleQueueProject {
	project: string
	result: QueueCandidateCheck
	excluded: string[]
}

const BATCH_SIZE = 5

export function isEligibleQueueCandidate(result: QueueCandidateCheck | undefined): boolean {
	if (!result?.isProcessing) return false
	return !result.locked || !!result.expired || !!result.isOwnLock
}

export async function batchCheckQueueCandidates(
	client: AbstractModrinthClient,
	moderationQueue: ModerationQueueService,
	projectIds: string[],
): Promise<Map<string, QueueCandidateCheck>> {
	const results = new Map<string, QueueCandidateCheck>()

	const projects = await client.labrinth.projects_v3.getMultiple(projectIds).catch(() => [])
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	const checks = await Promise.allSettled(
		projectIds.map(async (id) => {
			const lockResponse = await moderationQueue.checkLock(id)
			const project = projectsById.get(id) ?? null

			return {
				id,
				locked: lockResponse.locked,
				expired: lockResponse.expired,
				isOwnLock: lockResponse.is_own_lock,
				slug: project?.slug,
				projectType: project?.project_types[0],
				status: project?.status,
				isProcessing: project === null ? true : project.status === 'processing',
			}
		}),
	)

	checks.forEach((result, index) => {
		if (result.status === 'fulfilled') {
			results.set(result.value.id, result.value)
		} else {
			results.set(projectIds[index], { locked: false, isProcessing: true })
		}
	})

	return results
}

export async function findNextEligibleQueueProject(
	client: AbstractModrinthClient,
	moderationQueue: ModerationQueueService,
	candidateIds: string[],
): Promise<EligibleQueueProject | null> {
	const excluded: string[] = []
	let checkedCount = 0

	while (checkedCount < candidateIds.length) {
		const batch = candidateIds.slice(checkedCount, checkedCount + BATCH_SIZE)
		checkedCount += batch.length

		const results = await batchCheckQueueCandidates(client, moderationQueue, batch)

		for (const id of batch) {
			const result = results.get(id)
			if (isEligibleQueueCandidate(result)) {
				return { project: id, result: result!, excluded: [...excluded] }
			}
			excluded.push(id)
		}
	}

	return null
}
