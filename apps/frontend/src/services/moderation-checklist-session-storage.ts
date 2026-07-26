// Session-only checklist data — cleared automatically when the tab closes, unlike
// `moderation-checklist-storage.ts` (IndexedDB, survives across visits). Named-fields shape
// mirrors that store's `PersistedChecklistState` so future session-only fields can be added here
// without new plumbing.
export interface SessionChecklistState {
	visitedStages?: string[]
}

function sessionStorageKey(projectId: string): string {
	return `moderation-checklist-session:${projectId}`
}

export function getSessionChecklistState(projectId: string): SessionChecklistState {
	try {
		const raw = sessionStorage.getItem(sessionStorageKey(projectId))
		return raw ? JSON.parse(raw) : {}
	} catch {
		return {}
	}
}

export function patchSessionChecklistState(
	projectId: string,
	patch: Partial<SessionChecklistState>,
): void {
	try {
		sessionStorage.setItem(
			sessionStorageKey(projectId),
			JSON.stringify({ ...getSessionChecklistState(projectId), ...patch }),
		)
	} catch {
		// sessionStorage unavailable (private browsing, quota, etc.) — visited-state just won't persist
	}
}
