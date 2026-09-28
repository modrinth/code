import type { NodeState } from '@modrinth/moderation/src/types/node'
import { createContext } from '@modrinth/ui'
import { reactive, watch } from 'vue'

import { useAuthState } from '~/composables/auth'

type ReviewState = Record<string, NodeState>
type ProjectReviewState = Record<string, ReviewState>

const DRAFT_EXPIRY_MS = 30 * 24 * 60 * 60 * 1000

function isRecord(value: unknown): value is Record<string, unknown> {
	return !!value && typeof value === 'object' && !(value instanceof Set) && !Array.isArray(value)
}

function isNodeState(value: unknown): value is NodeState {
	return (
		value === null ||
		typeof value === 'boolean' ||
		typeof value === 'string' ||
		typeof value === 'number' ||
		(value instanceof Set && [...value].every((entry) => typeof entry === 'string')) ||
		(isRecord(value) && Object.values(value).every(isNodeState))
	)
}

function isProjectReviewState(value: unknown): value is ProjectReviewState {
	return (
		isRecord(value) &&
		Object.values(value).every(
			(scope) => isRecord(scope) && Object.values(scope).every(isNodeState),
		)
	)
}

export const [injectReviewSession, provideReviewSession] =
	createContext<ReturnType<typeof createReviewSession>>('ProjectReviewSession')

export function createReviewSession() {
	const auth = useAuthState()
	const projects = reactive(new Map<string, ProjectReviewState>())

	watch(
		() => auth.value.user?.id,
		() => projects.clear(),
		{ flush: 'sync' },
	)

	function storageKey(projectId: string) {
		const userId = auth.value.user?.id
		return import.meta.client && userId
			? `project-review-draft:v1:${userId}:${projectId}`
			: undefined
	}

	function removeDraft(projectId: string) {
		const key = storageKey(projectId)
		if (!key) return
		try {
			localStorage.removeItem(key)
		} catch (error) {
			console.warn('Failed to clear project review draft:', error)
		}
	}

	function readProject(projectId: string): ProjectReviewState {
		const cached = projects.get(projectId)
		if (cached) return cached
		const key = storageKey(projectId)
		let state: ProjectReviewState = {}
		if (key) {
			try {
				const saved = localStorage.getItem(key)
				if (saved) {
					const draft: unknown = JSON.parse(saved, (_, value) =>
						Array.isArray(value) ? new Set(value) : value,
					)
					if (
						isRecord(draft) &&
						typeof draft.updatedAt === 'number' &&
						Date.now() - draft.updatedAt < DRAFT_EXPIRY_MS &&
						isProjectReviewState(draft.state)
					) {
						state = draft.state
					} else {
						removeDraft(projectId)
					}
				}
			} catch (error) {
				console.warn('Failed to restore project review draft:', error)
				removeDraft(projectId)
			}
		}
		projects.set(projectId, state)
		return projects.get(projectId)!
	}

	function read(projectId: string, scope: string): ReviewState {
		return readProject(projectId)[scope] ?? {}
	}

	function write(projectId: string, scope: string, id: string, value: NodeState) {
		const project = readProject(projectId)
		const state = { ...project[scope] }
		if (value === undefined) Reflect.deleteProperty(state, id)
		else state[id] = value
		const next = { ...project, [scope]: state }
		projects.set(projectId, next)
		const key = storageKey(projectId)
		if (!key) return
		if (Object.values(next).every((scope) => Object.keys(scope).length === 0)) {
			removeDraft(projectId)
			return
		}
		try {
			localStorage.setItem(
				key,
				JSON.stringify({ updatedAt: Date.now(), state: next }, (_, value) =>
					value instanceof Set ? [...value] : value,
				),
			)
		} catch (error) {
			console.warn('Failed to save project review draft:', error)
		}
	}

	function clearProject(projectId: string) {
		projects.set(projectId, {})
		removeDraft(projectId)
	}

	return { read, write, readProject, clearProject }
}
