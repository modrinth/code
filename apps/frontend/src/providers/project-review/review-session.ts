import type { NodeState } from '@modrinth/moderation/src/types/node'
import { createContext } from '@modrinth/ui'
import { reactive } from 'vue'

type ReviewState = Record<string, NodeState>
type ProjectReviewState = Record<string, ReviewState>

export const [injectReviewSession, provideReviewSession] =
	createContext<ReturnType<typeof createReviewSession>>('ProjectReviewSession')

export function createReviewSession() {
	const projects = reactive(new Map<string, ProjectReviewState>())

	function read(projectId: string, scope: string): ReviewState {
		return projects.get(projectId)?.[scope] ?? {}
	}

	function write(projectId: string, scope: string, id: string, value: NodeState) {
		const project = projects.get(projectId) ?? {}
		const state = { ...project[scope] }
		if (value === undefined) Reflect.deleteProperty(state, id)
		else state[id] = value
		projects.set(projectId, { ...project, [scope]: state })
	}

	return { read, write }
}
