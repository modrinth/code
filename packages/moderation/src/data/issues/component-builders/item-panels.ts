import type {
	EachPanel,
	Issue,
	LabeledReviewPanel,
	Panel,
	PanelNode,
	PanelRegistration,
	ReviewContext,
	ReviewPanelItem,
	WithContext,
} from './types'

function isPanelGroup(definition: PanelRegistration): definition is readonly LabeledReviewPanel[] {
	return Array.isArray(definition)
}

function resolve<T>(value: WithContext<T>, context: ReviewContext): T {
	return typeof value === 'function' ? (value as (ctx: ReviewContext) => T)(context) : value
}

function itemContext(
	ctx: ReviewContext,
	item: ReviewPanelItem,
	toggleIds: ReadonlyMap<string, readonly string[]>,
): ReviewContext {
	const prefix = `${item.key}:`
	return {
		...ctx,
		selected: {
			...ctx.selected,
			issueIds: ctx.selected.issueIds.filter((id) =>
				toggleIds
					.get(id)
					?.some((toggleId) => ctx.selected.toggleIds.includes(`${prefix}${toggleId}`)),
			),
			toggleIds: ctx.selected.toggleIds
				.filter((id) => id.startsWith(prefix))
				.map((id) => id.slice(prefix.length)),
		},
		getMarkdownValue: (id, issue) => ctx.getMarkdownValue(`${prefix}${id}`, issue),
		getTextValue: (id, issue) => ctx.getTextValue(`${prefix}${id}`, issue),
		getSelectValue: (id, issue) => ctx.getSelectValue(`${prefix}${id}`, issue),
		getSelectValues: (id, issue) => ctx.getSelectValues(`${prefix}${id}`, issue),
	}
}

/** Expands keyed item panels while preserving independent inputs and shared issue aggregation. */
export function expandItemReviewPanels(
	definitions: Record<string, PanelRegistration>,
	context: ReviewContext,
) {
	const panels: Record<string, Panel> = {}
	const parents = new Map<string, string>()
	const items = new Map<string, ReviewPanelItem>()
	const itemKinds = new Map<ReviewPanelItem, string>()
	const issues = new Map<string, Issue>()
	const itemIssues = new Map<ReviewPanelItem, Map<string, Issue>>()
	const itemToggleIds = new Map<ReviewPanelItem, Map<string, string[]>>()
	const registrations = new Map<string, Panel | EachPanel>()
	for (const [key, definition] of Object.entries(definitions)) {
		const entries = isPanelGroup(definition)
			? definition.map(({ key, panel }) => [key, panel] as const)
			: [[key, definition] as const]
		for (const [panelKey, panel] of entries) {
			if (registrations.has(panelKey)) throw new Error(`Duplicate review panel "${panelKey}"`)
			registrations.set(panelKey, panel)
		}
	}
	for (const [key, definition] of registrations) {
		if (!('type' in definition)) {
			panels[key] = definition
			continue
		}
		for (const item of definition.items(context)) {
			if (resolve(item.panel.shown, context) === false) continue
			const itemKey = `${key}:${item.key}`
			if (items.has(itemKey)) throw new Error(`Duplicate review panel "${itemKey}"`)
			items.set(itemKey, item)
			itemKinds.set(item, key)
		}
	}
	function collectItemControls(nodes: readonly PanelNode[], item: ReviewPanelItem) {
		const definitions = itemIssues.get(item) ?? new Map<string, Issue>()
		const toggles = itemToggleIds.get(item) ?? new Map<string, string[]>()
		itemIssues.set(item, definitions)
		itemToggleIds.set(item, toggles)
		for (const node of nodes) {
			if (node.type === 'section') {
				collectItemControls(node.children, item)
				continue
			}
			if (node.issue) definitions.set(node.issue.id, node.issue)
			if (node.type === 'toggle') {
				const ids = toggles.get(node.issue.id) ?? []
				ids.push(node.id ?? node.issue.id)
				toggles.set(node.issue.id, ids)
			}
		}
	}
	for (const item of items.values()) collectItemControls(item.panel.children, item)
	function scopedContext(ctx: ReviewContext, item: ReviewPanelItem) {
		return itemContext(ctx, item, itemToggleIds.get(item)!)
	}
	function sharedIssue(original: Issue): Issue {
		const existing = issues.get(original.id)
		if (existing) return existing
		const selectedItems = (ctx: ReviewContext) =>
			[...items.values()].filter((item) =>
				itemToggleIds
					.get(item)
					?.get(original.id)
					?.some((id) => ctx.selected.toggleIds.includes(`${item.key}:${id}`)),
			)
		function selectedContext(ctx: ReviewContext): ReviewContext {
			const selected: Record<string, { key: string; context: ReviewContext }[]> = {}
			for (const item of selectedItems(ctx)) {
				const kind = itemKinds.get(item)!
				const entries = (selected[kind] ??= [])
				entries.push({ key: item.key, context: scopedContext(ctx, item) })
			}
			return { ...ctx, selected: { ...ctx.selected, items: selected } }
		}
		const issue: Issue = {
			...original,
			message: (ctx) => {
				ctx = selectedContext(ctx)
				const selected = selectedItems(ctx)
				if (!selected.length) return resolve(original.message, ctx)
				const messages = new Set(
					selected.map((item) =>
						resolve(itemIssues.get(item)!.get(original.id)!.message, {
							...scopedContext(ctx, item),
							selected: ctx.selected,
						}),
					),
				)
				return [...messages].join('\n\n')
			},
			locations: (ctx) => resolve(original.locations, selectedContext(ctx)) ?? [],
			actions: (ctx) => {
				const context = selectedContext(ctx)
				return (resolve(original.actions, context) ?? []).map((action) => () => action(context))
			},
		}
		issues.set(original.id, issue)
		return issue
	}
	for (const definitions of itemIssues.values()) {
		for (const issue of definitions.values()) sharedIssue(issue)
	}
	function scopeNodes(nodes: readonly PanelNode[], item?: ReviewPanelItem): PanelNode[] {
		return nodes.flatMap((node): PanelNode[] => {
			if (node.type === 'section') {
				if (!item) {
					const children = scopeNodes(node.children)
					return children.length ? [{ ...node, children }] : []
				}
				return [
					{
						...node,
						shown: (ctx) => resolve(node.shown, scopedContext(ctx, item)) !== false,
						label: (ctx) => resolve(node.label, scopedContext(ctx, item)) ?? '',
						children: scopeNodes(node.children, item),
					},
				]
			}
			const issue = node.issue ? (issues.get(node.issue.id) ?? node.issue) : undefined
			if (!item) return [node]
			const scoped = {
				...node,
				...(issue ? { issue } : {}),
				id: `${item.key}:${node.type === 'toggle' ? (node.id ?? node.issue.id) : node.id}`,
				label: (ctx: ReviewContext) => resolve(node.label, scopedContext(ctx, item)),
				shown: (ctx: ReviewContext) => resolve(node.shown, scopedContext(ctx, item)) !== false,
				disabled: (ctx: ReviewContext) => resolve(node.disabled, scopedContext(ctx, item)) === true,
				tooltip:
					node.tooltip === undefined
						? undefined
						: (ctx: ReviewContext) => resolve(node.tooltip, scopedContext(ctx, item)) ?? '',
			}
			if (scoped.type === 'select' && node.type === 'select') {
				scoped.initial = (ctx) => resolve(node.initial, scopedContext(ctx, item)) ?? []
				scoped.placeholder =
					node.placeholder === undefined
						? undefined
						: (ctx) => resolve(node.placeholder, scopedContext(ctx, item)) ?? ''
				scoped.options = (ctx) =>
					resolve(node.options, scopedContext(ctx, item)).map((option) => ({
						...option,
						label: (ctx: ReviewContext) => resolve(option.label, scopedContext(ctx, item)),
						shown: (ctx: ReviewContext) =>
							resolve(option.shown, scopedContext(ctx, item)) !== false,
						disabled: (ctx: ReviewContext) =>
							resolve(option.disabled, scopedContext(ctx, item)) === true,
					}))
			}
			if (scoped.type !== 'toggle' && node.type !== 'toggle') {
				scoped.required = (ctx) => resolve(node.required, scopedContext(ctx, item)) === true
			}
			if (
				(scoped.type === 'text' || scoped.type === 'textarea') &&
				(node.type === 'text' || node.type === 'textarea')
			) {
				scoped.initial = (ctx) => resolve(node.initial, scopedContext(ctx, item)) ?? ''
				scoped.placeholder =
					node.placeholder === undefined
						? undefined
						: (ctx) => resolve(node.placeholder, scopedContext(ctx, item)) ?? ''
			}
			if (node.type !== 'toggle') return [scoped]
			if (scoped.type === 'toggle') {
				scoped.issueListLabel = (ctx) =>
					resolve(node.issueListLabel, scopedContext(ctx, item)) ??
					resolve(item.panel.title, scopedContext(ctx, item)) ??
					item.key
				scoped.issueListGroup = (ctx) =>
					resolve(node.issueListGroup, scopedContext(ctx, item)) ??
					resolve(node.label, scopedContext(ctx, item))
			}
			return [scoped]
		})
	}
	for (const [key, panel] of Object.entries(panels)) {
		panels[key] = { ...panel, children: scopeNodes(panel.children) }
	}
	for (const [key, item] of items) {
		if (item.panel.parent) {
			if (!panels[item.panel.parent])
				throw new Error(`Review panel "${key}" has an unknown parent "${item.panel.parent}"`)
			parents.set(key, item.panel.parent)
		}
		panels[key] = {
			...item.panel,
			title: (ctx) => resolve(item.panel.title, scopedContext(ctx, item)) ?? item.key,
			hint: (ctx) => resolve(item.panel.hint, scopedContext(ctx, item)),
			children: scopeNodes(item.panel.children, item),
		}
	}
	return { panels, parents, issues }
}
