import type { Labrinth } from '@modrinth/api-client'

type ThreadIssue = Labrinth.Threads.v3.ThreadIssue

export interface ThreadIssueHistoryEntry {
	type: 'issues'
	id: string
	created: string
	authorId: string | null
	issues: ThreadIssue[]
}

interface TimelineMessage {
	id?: string | null
	created: string
	body?: { type: string; old_status?: string; new_status?: string }
}

/** Counts each issue against its closest review within ten minutes, preferring earlier reviews on ties. */
export function countThreadReviewIssues<T extends TimelineMessage>(
	messages: readonly T[],
	issues: readonly ThreadIssue[],
): Map<T, number> {
	const reviews = messages
		.filter(
			(message) =>
				message.body?.type === 'status_change' &&
				message.body.old_status === 'processing' &&
				message.body.new_status !== 'processing',
		)
		.map((message) => ({ message, time: Date.parse(message.created) }))
		.sort((a, b) => a.time - b.time || (a.message.id ?? '').localeCompare(b.message.id ?? ''))
	const counts = new Map<T, number>()
	const reviewWindow = 10 * 60 * 1000
	for (const issue of issues) {
		const issueTime = Date.parse(issue.created_at)
		let closest: T | undefined
		let closestDistance = Infinity
		for (const { message, time } of reviews) {
			const distance = Math.abs(time - issueTime)
			if (distance <= reviewWindow && distance < closestDistance) {
				closest = message
				closestDistance = distance
			}
		}
		if (closest) counts.set(closest, (counts.get(closest) ?? 0) + 1)
	}
	return counts
}

/** Adds issue history using its own timestamps, without inferring links to nearby messages. */
export function buildThreadTimeline<T extends TimelineMessage>(
	messages: readonly T[],
	issues: readonly ThreadIssue[],
	includeIssueHistory: boolean,
) {
	const entries: (
		| { type: 'message'; id: string; created: string; message: T }
		| ThreadIssueHistoryEntry
	)[] = messages.map((message, index) => ({
		type: 'message',
		id: `message:${message.id ?? `${message.created}:${index}`}`,
		created: message.created,
		message,
	}))
	if (includeIssueHistory) {
		const groups = new Map<string, ThreadIssueHistoryEntry>()
		for (const issue of issues) {
			const id = `issues:${JSON.stringify([issue.created_at, issue.created_by])}`
			const group: ThreadIssueHistoryEntry = groups.get(id) ?? {
				type: 'issues',
				id,
				created: issue.created_at,
				authorId: issue.created_by,
				issues: [],
			}
			group.issues.push(issue)
			groups.set(id, group)
		}
		entries.push(...groups.values())
	}
	return entries.sort((a, b) => Date.parse(a.created) - Date.parse(b.created))
}
