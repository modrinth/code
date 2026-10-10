import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'

import { sendThreadReply, type ThreadReply } from '~/helpers/thread-issues'

export type FacetUpdate = {
	id: string
	data: Labrinth.Threads.v3.EditThreadIssueFacet
}

export interface ReviewDecision extends ThreadReply {
	projectId: string
	status?: Labrinth.Projects.v2.ProjectStatus
	facetUpdates: FacetUpdate[]
	issues?: Labrinth.Threads.v3.NewThreadIssues
}

/** Applies a review's findings and optional status change. */
export async function applyReviewDecision(
	decision: ReviewDecision,
	client: Pick<AbstractModrinthClient, 'labrinth'>,
	assertCurrent: () => void,
	onMessageSent: () => void,
): Promise<void> {
	const threads = client.labrinth.threads_v3
	assertCurrent()
	if (decision.body.trim()) {
		await sendThreadReply(decision, client, assertCurrent)
		onMessageSent()
	}
	if (decision.issues) {
		assertCurrent()
		await threads.createIssues(decision.threadId, decision.issues)
	}
	for (const facet of decision.facetUpdates) {
		assertCurrent()
		await threads.editIssueFacet(facet.id, facet.data)
	}
	assertCurrent()
	if (decision.status) {
		await client.labrinth.projects_v3.edit(decision.projectId, {
			status: decision.status,
		})
	}
	assertCurrent()
}
