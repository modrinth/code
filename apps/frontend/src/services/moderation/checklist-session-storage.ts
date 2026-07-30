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
	} catch {}
}
