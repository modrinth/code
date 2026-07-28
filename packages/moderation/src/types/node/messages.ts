import type { Labrinth } from '@modrinth/api-client'
import type { Ref } from 'vue'

import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '../../utils'
import type { GetVarsFn, MessageSegment, NodeState } from './state'

const messageFiles = import.meta.glob('../../data/messages/**/*.md', {
	query: '?raw',
	import: 'default',
})

export type MessageFn = ((state: Record<string, NodeState>) => Promise<string>) & {
	concat(...others: MessageFn[]): MessageFn
}

function makeMessageFn(fn: (state: Record<string, NodeState>) => Promise<string>): MessageFn {
	const rich = fn as MessageFn
	rich.concat = (...others) =>
		makeMessageFn(async (state) =>
			(await Promise.all([rich, ...others].map((f) => f(state)))).join(''),
		)
	return rich
}

let _project: Ref<Labrinth.Projects.v3.Project> | null = null
let _projectV2: Ref<Labrinth.Projects.v2.Project> | null = null
let _onMissingMd: ((path: string) => void) | null = null

export function setMissingMdHandler(handler: (path: string) => void) {
	_onMissingMd = handler
}

export function setMessageProject(
	project: Ref<Labrinth.Projects.v3.Project>,
	projectV2: Ref<Labrinth.Projects.v2.Project>,
) {
	_project = project
	_projectV2 = projectV2
}

export function mdEscape(text: string): string {
	return text.replace(/[\\*_`[~]/g, '\\$&')
}

const USER_CONTENT_KEYS = ['PROJECT_TITLE', 'PROJECT_SLUG', 'PROJECT_SUMMARY', 'PROJECT_TYPE', 'PROJECT_STATUS']

export async function loadMd(
	path: string,
	state: Record<string, NodeState>,
	project: Labrinth.Projects.v3.Project,
	projectV2: Labrinth.Projects.v2.Project,
	getVars?: GetVarsFn,
): Promise<string> {
	const extraVars = getVars ? getVars(state) : null
	const loader = messageFiles[`../../data/messages/${path}.md`]
	if (!loader) {
		_onMissingMd?.(path)
		return ''
	}
	const raw = (await loader()) as string
	const vars: Record<string, string> = {
		...flattenStaticVariables(),
		...flattenProjectVariables(projectV2),
		...flattenProjectV3Variables(project),
	}
	for (const key of USER_CONTENT_KEYS) {
		if (key in vars) vars[key] = mdEscape(vars[key])
	}
	if (extraVars) {
		for (const [k, v] of Object.entries(extraVars)) {
			vars[k] = String(v ?? '')
		}
	}
	const expanded = expandVariables(raw, projectV2, project, vars)
	return expanded.replace(/`[^`\n]*`/g, (match) => match.replace(/\\([\\*_`[~])/g, '$1'))
}

export function mdOptional(path: string, getVars?: GetVarsFn): MessageFn {
	return makeMessageFn(async (state) => {
		const loader = messageFiles[`../../data/messages/${path}.md`]
		if (!loader) return ''
		return loadMd(path, state, _project!.value, _projectV2!.value, getVars)
	})
}

export function md(path: string | ((state: Record<string, NodeState>) => string), getVars?: GetVarsFn): MessageFn {
	return makeMessageFn(async (state) => {
		const resolvedPath = typeof path === 'function' ? path(state) : path
		return loadMd(resolvedPath, state, _project!.value, _projectV2!.value, getVars)
	})
}

export function resolveRelativeMessagePath(messagePath: string | (() => string), statePath: string[]): string {
	const name = typeof messagePath === 'function' ? messagePath() : messagePath
	if (name.startsWith('/')) return `checklist/messages${name}`
	const parts = [...statePath.slice(0, -1), ...name.split('/')]
	const normalized = parts.reduce<string[]>((acc, p) => {
		if (p === '..') acc.pop()
		else if (p) acc.push(p)
		return acc
	}, [])
	return `checklist/messages/${normalized.join('/')}`
}

export async function evalSegment(
	seg: MessageSegment,
	state: Record<string, NodeState>,
	statePath: string[],
): Promise<string> {
	if (seg.type === 'collect') return ''
	if (seg.type === 'fn') return String((await seg.fn(state)) ?? '')
	if (seg.type === 'auto') {
		return loadMd(`checklist/messages/${statePath.join('/')}`, state, _project!.value, _projectV2!.value, seg.getVars)
	}
	return loadMd(resolveRelativeMessagePath(seg.path, statePath), state, _project!.value, _projectV2!.value, seg.getVars)
}
