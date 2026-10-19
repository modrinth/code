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
	issuesMessage?: string
}

export class ReviewIssuesError extends Error {
	constructor(
		readonly mode: 'message' | 'issues',
		cause: unknown,
	) {
		super(cause instanceof Error ? cause.message : String(cause), { cause })
	}
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
	// Send thread messages after the status change so they appear below its message in the thread.
	if (decision.status) {
		await client.labrinth.projects_v3.edit(decision.projectId, {
			status: decision.status,
		})
	}
	assertCurrent()
	if (decision.body.trim()) {
		await sendThreadReply(decision, client, assertCurrent)
		onMessageSent()
	}
	if (decision.issues) {
		assertCurrent()
		try {
			await threads.createIssues(decision.threadId, decision.issues)
		} catch (error) {
			throw new ReviewIssuesError('issues', error)
		}
	}
	if (decision.issuesMessage) {
		assertCurrent()
		try {
			await sendThreadReply(
				{
					threadId: decision.threadId,
					body: decision.issuesMessage,
					images: [],
					privateMessage: false,
				},
				client,
				assertCurrent,
			)
		} catch (error) {
			throw new ReviewIssuesError('message', error)
		}
	}
	for (const facet of decision.facetUpdates) {
		assertCurrent()
		await threads.editIssueFacet(facet.id, facet.data)
	}
	assertCurrent()
}
