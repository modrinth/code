import type { AbstractModrinthClient } from '@modrinth/api-client'
import { moderationSettings } from '@modrinth/moderation'

import type { ModerationQueueService } from './queue.ts'

interface QueueCandidateCheck {
	locked: boolean
	expired?: boolean
	isOwnLock?: boolean
	isProcessing: boolean
}

export interface EligibleQueueProject {
	project: string
	excluded: string[]
}

const BATCH_SIZE = 5

function isEligibleQueueCandidate(
	result: QueueCandidateCheck | undefined,
	includeNonProcessing: boolean,
): boolean {
	if (!result || (!result.isProcessing && !includeNonProcessing)) return false
	return !result.locked || !!result.expired || !!result.isOwnLock
}

async function batchCheckQueueCandidates(
	client: AbstractModrinthClient,
	moderationQueue: ModerationQueueService,
	projectIds: string[],
	strict = false,
): Promise<Map<string, QueueCandidateCheck>> {
	const results = new Map<string, QueueCandidateCheck>()

	const projects = await client.labrinth.projects_v3.getMultiple(projectIds)
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	const checks = await Promise.allSettled(
		projectIds.map(async (id) => {
			const project = projectsById.get(id) ?? null
			if (!project) {
				return { id, locked: false, isProcessing: false }
			}

			const lockResponse = await moderationQueue.checkLock(id, strict)

			return {
				id,
				locked: lockResponse.locked,
				expired: lockResponse.expired,
				isOwnLock: lockResponse.is_own_lock,
				isProcessing: project.status === 'processing',
			}
		}),
	)

	checks.forEach((result, index) => {
		if (result.status === 'fulfilled') {
			results.set(result.value.id, result.value)
		} else {
			if (strict) throw result.reason
			results.set(projectIds[index], { locked: false, isProcessing: false })
		}
	})

	return results
}

export async function findNextEligibleQueueProject(
	client: AbstractModrinthClient,
	moderationQueue: ModerationQueueService,
	candidateIds: string[],
	strict = false,
	includeNonProcessing = useModerationSettings().value.get(
		moderationSettings.General.IncludeNonProcessingProjects,
	),
): Promise<EligibleQueueProject | null> {
	const excluded: string[] = []
	let checkedCount = 0

	while (checkedCount < candidateIds.length) {
		const batch = candidateIds.slice(checkedCount, checkedCount + BATCH_SIZE)
		checkedCount += batch.length

		const results = await batchCheckQueueCandidates(client, moderationQueue, batch, strict)

		for (const id of batch) {
			const result = results.get(id)
			if (isEligibleQueueCandidate(result, includeNonProcessing)) {
				return { project: id, excluded: [...excluded] }
			}
			excluded.push(id)
		}
	}

	return null
}
