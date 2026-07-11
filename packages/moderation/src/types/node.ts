import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '../utils'

export type NodeType = 'boolean' | 'select' | 'multi-select' | 'text' | 'markdown' | 'label' | 'group' | 'prose'
export type NodeVariant = 'button' | 'toggle'
export type ModerationStatus = 'approved' | 'rejected' | 'flagged'
export type ModerationSeverity = 'low' | 'medium' | 'high' | 'critical'

export type NodeState = boolean | string | number | Set<string> | NodeStateWithChildren | null | undefined

export interface NodeStateWithChildren {
	value?: boolean | string | number | Set<string> | null
	[childId: string]: NodeState
}

export interface NodeContext {
	project: Labrinth.Projects.v3.Project
	projectV2: Labrinth.Projects.v2.Project
	state: Record<string, NodeState>
	globalState: Record<string, Record<string, NodeState>>
}

export type MessageFn = (ctx: NodeContext) => Promise<string>
export type ContentFn = (ctx: NodeContext) => string | Promise<string>
export type ShowFn = (ctx: NodeContext) => boolean
export type ChildrenFn = (ctx: NodeContext) => Node[]

const messageFiles = import.meta.glob('../data/messages/checklist-messages/**/*.md', {
	query: '?raw',
	import: 'default',
})

export function mdMsg(
	path: string,
	getVars?: (ctx: NodeContext) => Record<string, NodeState>,
): MessageFn {
	return async (ctx) => {
		const loader = messageFiles[`../data/messages/checklist-messages/${path}.md`]
		if (!loader) return ''
		const raw = (await loader()) as string
		if (!getVars) return raw
		const vars = getVars(ctx)
		return Object.entries(vars).reduce(
			(result, [key, value]) => result.replace(new RegExp(`%${key}%`, 'g'), String(value ?? '')),
			raw,
		)
	}
}

const textFiles = import.meta.glob('../data/messages/checklist-text/**/*.md', {
	query: '?raw',
	import: 'default',
})

export function mdEscape(text: string): string {
	return text.replace(/[\\*_`[~]/g, '\\$&')
}

const USER_CONTENT_KEYS = ['PROJECT_TITLE', 'PROJECT_SLUG', 'PROJECT_SUMMARY', 'PROJECT_TYPE', 'PROJECT_STATUS']

export function mdText(path: string): ContentFn {
	return async (ctx) => {
		const loader = textFiles[`../data/messages/checklist-text/${path}.md`]
		if (!loader) return ''
		const raw = (await loader()) as string
		const vars: Record<string, string> = {
			...flattenStaticVariables(),
			...flattenProjectVariables(ctx.projectV2),
			...flattenProjectV3Variables(ctx.project),
		}
		for (const key of USER_CONTENT_KEYS) {
			if (key in vars) vars[key] = mdEscape(vars[key])
		}
		return expandVariables(raw, ctx.projectV2, ctx.project, vars)
	}
}

export interface Node {
	id?: string
	type: NodeType
	label?: string
	variant?: NodeVariant
	layout?: 'flex' | 'column'
	weight?: number
	children?: Node[]
	childrenFn?: ChildrenFn
	message?: MessageFn
	content?: ContentFn
	shown?: ShowFn
	enabled?: ShowFn
	suggestedStatus?: ModerationStatus
	severity?: ModerationSeverity
	defaultChecked?: boolean
	placeholder?: string
	required?: boolean
}

export interface ChecklistStage {
	id: string
	title: string
	hint: string
	guidance_url: string
	icon?: FunctionalComponent<SVGAttributes>
	navigate?: string
	nodes: Node[]
	shown?: (
		project: Labrinth.Projects.v2.Project,
		projectV3?: Labrinth.Projects.v3.Project,
	) => boolean
}

type Resolvable = Node | NodeBuilder

function resolve(n: Resolvable): Node {
	return n instanceof NodeBuilder ? n.build() : n
}

export class NodeBuilder {
	private data: Partial<Node>

	constructor(data: Partial<Node>) {
		this.data = { ...data }
	}

	weight(w: number): this {
		this.data.weight = w
		return this
	}

	message(fn: MessageFn): this {
		this.data.message = fn
		return this
	}

	content(fn: ContentFn): this {
		this.data.content = fn
		return this
	}

	column(): this {
		this.data.layout = 'column'
		return this
	}

	children(...nodes: Resolvable[]): this {
		this.data.children = nodes.map(resolve)
		return this
	}

	childrenFn(fn: ChildrenFn): this {
		this.data.childrenFn = fn
		return this
	}

	shown(fn: ShowFn): this {
		this.data.shown = fn
		return this
	}

	enabled(fn: ShowFn): this {
		this.data.enabled = fn
		return this
	}

	suggestedStatus(s: ModerationStatus): this {
		this.data.suggestedStatus = s
		return this
	}

	severity(s: ModerationSeverity): this {
		this.data.severity = s
		return this
	}

	defaultChecked(v = true): this {
		this.data.defaultChecked = v
		return this
	}

	placeholder(p: string): this {
		this.data.placeholder = p
		return this
	}

	required(v = true): this {
		this.data.required = v
		return this
	}

	build(): Node {
		return this.data as Node
	}
}

function node(id: string, label: string, type: NodeType, variant?: NodeVariant): NodeBuilder {
	return new NodeBuilder({ id, label, type, variant })
}

export const button = (id: string, label: string) => node(id, label, 'boolean', 'button')
export const toggle = (id: string, label: string) => node(id, label, 'boolean', 'toggle')
export const select = (id: string, label: string) => node(id, label, 'select')
export const chips = (id: string, label: string) => node(id, label, 'multi-select')
export const text = (id: string, label: string) => node(id, label, 'text')
export const markdown = (id: string, label: string) => node(id, label, 'markdown')
export const label = (id: string, content: string) => node(id, content, 'label')
export const group = (id?: string) => new NodeBuilder({ id, type: 'group' })
export const prose = (content: ContentFn) => new NodeBuilder({ type: 'prose', content })

interface StageOptions {
	icon?: ChecklistStage['icon']
	navigate?: string
	shown?: ChecklistStage['shown']
}

export function stage(
	id: string,
	title: string,
	hint: string,
	guidance_url: string,
	options: StageOptions = {},
	nodes: Resolvable[],
): ChecklistStage {
	return {
		id,
		title,
		hint,
		guidance_url,
		nodes: nodes.map(resolve),
		...options,
	}
}
