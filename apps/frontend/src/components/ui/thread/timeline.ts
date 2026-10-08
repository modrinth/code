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
