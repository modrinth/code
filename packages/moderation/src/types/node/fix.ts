import type { Labrinth } from '@modrinth/api-client'

import type { NodeState } from './state'

export class FixBuilder {
	_projectFn?: (
		patch: Labrinth.Projects.v3.EditProjectRequest,
		state: Record<string, NodeState>,
	) => void
	_versionFn?: (
		patch: Labrinth.Versions.v3.ModifyVersionRequest,
		state: Record<string, NodeState>,
	) => void

	project(
		fn: (patch: Labrinth.Projects.v3.EditProjectRequest, state: Record<string, NodeState>) => void,
	): this {
		this._projectFn = fn
		return this
	}

	version(
		fn: (
			patch: Labrinth.Versions.v3.ModifyVersionRequest,
			state: Record<string, NodeState>,
		) => void,
	): this {
		this._versionFn = fn
		return this
	}
}

export function fix(): FixBuilder {
	return new FixBuilder()
}

export function createTrackedPatch<T extends object>(
	source: T,
): { proxy: T; changes: () => Partial<T> } {
	const written = new Set<string | symbol>()
	const data = { ...source }
	const proxy = new Proxy(data, {
		set(target, key, value) {
			if (value !== (source as Record<string | symbol, unknown>)[key]) {
				written.add(key)
			} else {
				written.delete(key)
			}
			;(target as Record<string | symbol, unknown>)[key] = value
			return true
		},
	})
	return {
		proxy,
		changes: () =>
			Object.fromEntries([...written].map((k) => [k, data[k as keyof T]])) as Partial<T>,
	}
}
