import type { AbstractModrinthClient } from '@modrinth/api-client'
import { moderationSettings } from '@modrinth/moderation'

import type { LocksStatusResponse, ModerationQueueService } from './queue.ts'

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
	locks: LocksStatusResponse,
	projectIds: string[],
): Promise<Map<string, QueueCandidateCheck>> {
	const results = new Map<string, QueueCandidateCheck>()

	const projects = await client.labrinth.projects_v3.getMultiple(projectIds)
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	for (const id of projectIds) {
		const project = projectsById.get(id)
		if (!project) {
			results.set(id, { locked: false, isProcessing: false })
			continue
		}

		const lockResponse = locks[project.id]

		results.set(id, {
			locked: lockResponse?.locked ?? false,
			expired: lockResponse?.expired,
			isOwnLock: lockResponse?.is_own_lock,
			isProcessing: project.status === 'processing',
		})
	}

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
	if (candidateIds.length === 0) return null
	const locks = await moderationQueue.checkLocks(strict)

	while (checkedCount < candidateIds.length) {
		const batch = candidateIds.slice(checkedCount, checkedCount + BATCH_SIZE)
		checkedCount += batch.length

		const results = await batchCheckQueueCandidates(client, locks, batch)

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
