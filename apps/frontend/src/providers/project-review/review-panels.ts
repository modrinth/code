import type { Labrinth } from '@modrinth/api-client'
import { IssuePriority, reviewPanels } from '@modrinth/moderation/src/data/issues'
import { expandItemReviewPanels } from '@modrinth/moderation/src/data/issues/component-builders/item-panels'
import { resolveIssueFacets } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import type {
	Issue,
	Panel,
	PanelNode,
	PanelRegistration,
	ReviewContext,
	WithContext,
} from '@modrinth/moderation/src/data/issues/component-builders/types'
import { createContext } from '@modrinth/ui'
import { computed, type Ref } from 'vue'

import type { ReviewTarget } from './review'
import type { createReviewSession } from './review-session'

interface ResolvedIssueControlOptions {
	disabled: boolean
	required: boolean
	issue: Issue
	issueId: string
	childToggleIds?: string[]
	label: string
	tooltip?: string
}

export type ResolvedIssueControl = ResolvedIssueControlOptions &
	(
		| { type: 'toggle'; id?: string; issueListLabel?: string; issueListGroup?: string }
		| {
				type: 'markdown' | 'text' | 'textarea'
				key: string
				initial: string
				placeholder?: string
				rows?: number
				maxlength?: number
		  }
		| {
				type: 'select'
				key: string
				multiple: boolean
				initial: string[]
				placeholder?: string
				options: { value: string; label: string; disabled: boolean }[]
		  }
	)

interface IssueSelection {
	issue: Issue
	active: boolean
	keys: Set<string>
	textValues: Record<string, string>
	selectValues: Record<string, string[]>
	missing: string[]
}

interface ResolvedPanelSection {
	label?: string
	controls: ResolvedIssueControl[]
}

export interface ReviewPanelBinding {
	key: string
	parentKey?: string
	projectId: string
	panel: {
		icon: Panel['icon']
		title: string | undefined
		hint: string
		guidanceUrl: string
		sections: ResolvedPanelSection[]
	}
}

export interface ReviewIssueSelection {
	active: boolean
	toggle_ids: string[]
	text_values: Record<string, string>
	select_values: Record<string, string[]>
}

export interface ReviewIssueControl {
	binding: ReviewPanelBinding
	control: ResolvedIssueControl
}

export interface ReviewIssue {
	id: string
	title: string
	category: string
	priority?: Issue['priority']
	controls: ReviewIssueControl[]
}

function resolveWithContext<T>(value: WithContext<T>, context: ReviewContext): T {
	return typeof value === 'function' ? (value as (context: ReviewContext) => T)(context) : value
}

export const [injectReviewPanels, provideReviewPanels] =
	createContext<ReturnType<typeof createReviewPanels>>('ProjectReviewPanels')

export function createReviewPanels(
	project: Ref<Labrinth.Projects.v3.Project | undefined>,
	session: ReturnType<typeof createReviewSession>,
	reviewData: Ref<
		Pick<
			ReviewContext,
			| 'projectV2'
			| 'disclosures'
			| 'members'
			| 'organization'
			| 'organizationMembers'
			| 'wasReviewed'
			| 'permissions'
		> & { previousLinks: ReadonlyMap<string, string> }
	>,
	definitions: Record<string, PanelRegistration> = reviewPanels,
) {
	function selectedToggleIds(projectId: string, issueId: string): Set<string> {
		const keys = session.read(projectId, 'issues')[issueId]
		return keys instanceof Set ? keys : new Set()
	}

	function isSelected(projectId: string, control: ResolvedIssueControl): boolean {
		if (control.type !== 'toggle') return false
		return control.id === undefined
			? session.read(projectId, 'issue-active')[control.issueId] === true
			: selectedToggleIds(projectId, control.issueId).has(control.id)
	}

	function textValues(projectId: string, issueId: string): Record<string, string> {
		const values = session.read(projectId, 'issue-text')[issueId]
		if (!values || typeof values !== 'object' || values instanceof Set) return {}
		const result: Record<string, string> = {}
		for (const [key, value] of Object.entries(values)) {
			if (typeof value === 'string') result[key] = value
		}
		return result
	}

	const selectedIssueIds = computed(() => {
		const projectId = project.value?.id
		if (!projectId) return []
		return [
			...new Set([
				...Object.entries(session.read(projectId, 'issue-active'))
					.filter(([, selected]) => selected === true)
					.map(([id]) => id),
				...Object.entries(session.read(projectId, 'issues'))
					.filter(([, keys]) => keys instanceof Set && keys.size > 0)
					.map(([id]) => id),
			]),
		]
	})

	const panels = computed(() => {
		const bindings = new Map<string, ReviewPanelBinding>()
		const projectV3 = project.value
		if (!projectV3) return bindings
		const context: ReviewContext = {
			projectV3,
			...reviewData.value,
			getTextValue: (id, issueId) => (issueId ? (textValues(projectV3.id, issueId)[id] ?? '') : ''),
			getSelectValue: (id, issueId) =>
				issueId ? (readSelectValues(projectV3.id, issueId, id)[0] ?? '') : '',
			getSelectValues: (id, issueId) =>
				issueId ? readSelectValues(projectV3.id, issueId, id) : [],
			getMarkdownValue: (id, issueId) =>
				issueId ? (textValues(projectV3.id, issueId)[id] ?? '') : '',
			selected: {
				items: {},
				issueIds: selectedIssueIds.value,
				toggleIds: [
					...new Set(
						Object.values(session.read(projectV3.id, 'issues')).flatMap((keys) =>
							keys instanceof Set ? [...keys] : [],
						),
					),
				],
			},
		}
		const {
			panels: itemDefinitions,
			parents: itemParents,
			issues: itemIssues,
		} = expandItemReviewPanels(definitions, context)
		for (const [key, panel] of Object.entries(itemDefinitions)) {
			const previousLink =
				key.endsWith('-link') && reviewData.value.previousLinks.has(key.slice(0, -'-link'.length))
			if (resolveWithContext(panel.shown, context) === false && !previousLink) continue
			const resolveNodes = (
				nodes: readonly PanelNode[],
				label?: string,
			): ResolvedPanelSection[] => {
				const sections: ResolvedPanelSection[] = []
				let controls: ResolvedIssueControl[] = []

				function flush() {
					if (controls.length) sections.push({ label, controls })
					controls = []
					label = undefined
				}

				for (const node of nodes) {
					if (node.type === 'section') {
						if (resolveWithContext(node.shown, context) === false) continue
						flush()
						sections.push(...resolveNodes(node.children, resolveWithContext(node.label, context)))
						continue
					}
					const issue = itemIssues.get(node.issue.id) ?? node.issue
					const issueId = issue.id
					const issueContext: ReviewContext = {
						...context,
						getMarkdownValue: (id, scope = issueId) => textValues(projectV3.id, scope)[id] ?? '',
						getTextValue: (id, scope = issueId) => textValues(projectV3.id, scope)[id] ?? '',
						getSelectValue: (id, scope = issueId) =>
							readSelectValues(projectV3.id, scope, id)[0] ?? '',
						getSelectValues: (id, scope = issueId) => readSelectValues(projectV3.id, scope, id),
						selected: {
							items: {},
							issueIds: selectedIssueIds.value,
							toggleIds: [...selectedToggleIds(projectV3.id, issueId)],
						},
					}
					if (resolveWithContext(node.shown, issueContext) === false) continue
					const options: ResolvedIssueControlOptions = {
						disabled: resolveWithContext(node.disabled, issueContext) === true,
						issue,
						issueId,
						required:
							node.type !== 'toggle' && resolveWithContext(node.required, issueContext) === true,
						label: resolveWithContext(node.label, issueContext),
						tooltip: resolveWithContext(node.tooltip, issueContext),
					}
					let control: ResolvedIssueControl
					if (node.type === 'toggle') {
						control = {
							...options,
							type: 'toggle',
							id: node.id,
							issueListLabel: resolveWithContext(node.issueListLabel, issueContext),
							issueListGroup: resolveWithContext(node.issueListGroup, issueContext),
						}
					} else if (node.type === 'select') {
						const initial = resolveWithContext(node.initial, issueContext)
						control = {
							...options,
							type: 'select',
							key: node.id,
							multiple: node.multiple === true,
							initial: typeof initial === 'string' ? [initial] : [...(initial ?? [])],
							placeholder: resolveWithContext(node.placeholder, issueContext),
							options: resolveWithContext(node.options, issueContext)
								.filter((option) => resolveWithContext(option.shown, issueContext) !== false)
								.map((option) => ({
									value: option.value,
									label: resolveWithContext(option.label, issueContext),
									disabled: resolveWithContext(option.disabled, issueContext) === true,
								})),
						}
					} else {
						control = {
							...options,
							type: node.type,
							key: node.id,
							initial:
								node.type === 'text' || node.type === 'textarea'
									? (resolveWithContext(node.initial, issueContext) ?? '')
									: '',
							placeholder:
								node.type === 'text' || node.type === 'textarea'
									? resolveWithContext(node.placeholder, issueContext)
									: undefined,
							rows: node.type === 'textarea' ? node.rows : undefined,
							maxlength: node.type === 'textarea' ? node.maxlength : undefined,
						}
					}
					controls.push(control)
				}
				flush()
				return sections
			}

			const sections = resolveNodes(panel.children)
			bindings.set(key, {
				key,
				parentKey: itemParents.get(key),
				projectId: projectV3.id,
				panel: {
					icon: panel.icon,
					title: resolveWithContext(panel.title, context),
					hint: resolveWithContext(panel.hint, context),
					guidanceUrl: panel.guidanceUrl,
					sections,
				},
			})
		}
		for (const binding of bindings.values()) {
			for (const control of binding.panel.sections.flatMap((section) => section.controls)) {
				if (control.type !== 'toggle') continue
				const children = [...bindings.values()]
					.filter((item) => item.parentKey === binding.key)
					.flatMap((item) => item.panel.sections.flatMap((section) => section.controls))
					.filter(
						(item) =>
							item.type === 'toggle' &&
							item.issueId === control.issueId &&
							item.id?.endsWith(`:${control.id ?? control.issueId}`),
					)
				if (!children.length) continue
				control.childToggleIds = children.flatMap((item) =>
					item.type === 'toggle' && !item.disabled && item.id ? [item.id] : [],
				)
				control.disabled ||= control.childToggleIds.length === 0
			}
		}
		return bindings
	})

	function resolve(target: ReviewTarget) {
		const aliases: Partial<Record<ReviewTarget['kind'], string>> = {
			tags: 'categories',
			compatibility: 'metadata',
			'license-url': 'license',
			version: 'versions',
		}
		if (target.kind === 'gallery-image') return panels.value.get(`${target.kind}:${target.key}`)
		if (target.kind === 'disclosure') return panels.value.get(`${target.key}-disclosure`)
		return panels.value.get(
			target.kind === 'link' ? `${target.key}-link` : (aliases[target.kind] ?? target.kind),
		)
	}

	function selected(binding: ReviewPanelBinding, control: ResolvedIssueControl): boolean {
		if (control.childToggleIds)
			return (
				control.childToggleIds.length > 0 &&
				control.childToggleIds.every((id) =>
					selectedToggleIds(binding.projectId, control.issueId).has(id),
				)
			)
		return isSelected(binding.projectId, control)
	}

	/** Counts selected toggle controls in this panel, rather than distinct issue IDs. */
	function selectedFindingCount(binding: ReviewPanelBinding): number {
		return binding.panel.sections.reduce(
			(count, section) =>
				count + section.controls.filter((control) => selected(binding, control)).length,
			0,
		)
	}

	function textValue(binding: ReviewPanelBinding, control: ResolvedIssueControl): string {
		if (control.type !== 'markdown' && control.type !== 'text' && control.type !== 'textarea')
			return ''
		return textValues(binding.projectId, control.issueId)[control.key] ?? control.initial
	}

	function readSelectValues(projectId: string, issueId: string, key: string): string[] {
		const values = session.read(projectId, 'issue-select')[issueId]
		if (!values || typeof values !== 'object' || values instanceof Set) return []
		const value = values[key]
		return value instanceof Set ? [...value] : []
	}

	function selectValues(binding: ReviewPanelBinding, control: ResolvedIssueControl): string[] {
		if (control.type !== 'select') return []
		const stored = session.read(binding.projectId, 'issue-select')[control.issueId]
		const hasValue =
			stored && typeof stored === 'object' && !(stored instanceof Set) && control.key in stored
		const values = hasValue
			? readSelectValues(binding.projectId, control.issueId, control.key)
			: control.initial
		const allowed = new Set(
			control.options.filter((option) => !option.disabled).map((option) => option.value),
		)
		const selected = [...new Set(values)].filter((value) => allowed.has(value))
		return control.multiple ? selected : selected.slice(0, 1)
	}

	function missing(binding: ReviewPanelBinding, control: ResolvedIssueControl): boolean {
		if (!control.required || control.disabled) return false
		return control.type === 'select'
			? selectValues(binding, control).length === 0
			: control.type !== 'toggle' && !textValue(binding, control).trim()
	}

	function write(
		binding: ReviewPanelBinding,
		control: ResolvedIssueControl,
		value: boolean | string | string[],
	) {
		if (binding.projectId !== project.value?.id) return
		const current = panels.value
			.get(binding.key)
			?.panel.sections.flatMap((section) => section.controls)
			.find(
				(entry) =>
					entry.issueId === control.issueId &&
					entry.type === control.type &&
					(entry.type === 'toggle' && control.type === 'toggle'
						? entry.id === control.id
						: entry.type !== 'toggle' && control.type !== 'toggle' && entry.key === control.key),
			)
		if (!current || current.disabled) return
		if (current.type === 'select') {
			if (typeof value === 'boolean') return
			const values = typeof value === 'string' ? (value ? [value] : []) : value
			if (!current.multiple && values.length > 1) return
			if (
				values.some(
					(value) => !current.options.some((option) => option.value === value && !option.disabled),
				)
			)
				return
			const stored = session.read(binding.projectId, 'issue-select')[current.issueId]
			const previous =
				stored && typeof stored === 'object' && !(stored instanceof Set) ? stored : {}
			session.write(binding.projectId, 'issue-select', current.issueId, {
				...previous,
				[current.key]: new Set(values),
			})
			return
		}
		if (current.type !== 'toggle') {
			if (typeof value !== 'string') return
			const values = textValues(binding.projectId, current.issueId)
			values[current.key] = value
			session.write(binding.projectId, 'issue-text', current.issueId, values)
			return
		}
		if (typeof value !== 'boolean') return
		if (current.childToggleIds) {
			const keys = new Set(selectedToggleIds(binding.projectId, current.issueId))
			for (const id of current.childToggleIds) {
				if (value) keys.add(id)
				else keys.delete(id)
			}
			session.write(binding.projectId, 'issues', current.issueId, keys.size ? keys : undefined)
			updateIssueOrder(binding.projectId, current.issueId)
			return
		}
		if (current.id === undefined) {
			session.write(binding.projectId, 'issue-active', current.issueId, value || undefined)
			updateIssueOrder(binding.projectId, current.issueId)
			return
		}
		const keys = new Set(selectedToggleIds(binding.projectId, current.issueId))
		if (value) keys.add(current.id)
		else keys.delete(current.id)
		session.write(binding.projectId, 'issues', current.issueId, keys.size ? keys : undefined)
		updateIssueOrder(binding.projectId, current.issueId)
	}

	function updateIssueOrder(projectId: string, issueId: string) {
		const active =
			session.read(projectId, 'issue-active')[issueId] === true ||
			selectedToggleIds(projectId, issueId).size > 0
		const order = session.read(projectId, 'issue-order')
		if (active && order[issueId] !== true) session.write(projectId, 'issue-order', issueId, true)
		else if (!active && order[issueId] === true)
			session.write(projectId, 'issue-order', issueId, undefined)
	}

	const availableIssues = computed(() => {
		const issues = new Map<string, ReviewIssue>()
		for (const binding of panels.value.values()) {
			for (const section of binding.panel.sections) {
				for (const control of section.controls) {
					if (control.childToggleIds) continue
					const issue: ReviewIssue = issues.get(control.issueId) ?? {
						id: control.issueId,
						title: control.issue.title,
						category: control.issue.category,
						priority: control.issue.priority,
						controls: [],
					}
					if (
						!issue.controls.some(
							({ control: existing }) =>
								existing.type === control.type &&
								(existing.type === 'toggle' && control.type === 'toggle'
									? existing.id === control.id
									: existing.type !== 'toggle' &&
										control.type !== 'toggle' &&
										existing.key === control.key),
						)
					)
						issue.controls.push({ binding, control })
					issues.set(issue.id, issue)
				}
			}
		}
		return [...issues.values()].filter((issue) =>
			issue.controls.some(({ control }) => control.type === 'toggle' && !control.disabled),
		)
	})

	function addIssue(id: string) {
		const current = project.value
		const issue = availableIssues.value.find((issue) => issue.id === id)
		if (!current || !issue) return
		session.write(current.id, 'issue-active', id, true)
		updateIssueOrder(current.id, id)
	}

	function issueSelection(id: string): ReviewIssueSelection {
		const projectId = project.value?.id
		if (!projectId) return { active: false, toggle_ids: [], text_values: {}, select_values: {} }
		const selects = session.read(projectId, 'issue-select')[id]
		return {
			active: session.read(projectId, 'issue-active')[id] === true,
			toggle_ids: [...selectedToggleIds(projectId, id)].sort(),
			text_values: Object.fromEntries(
				Object.entries(textValues(projectId, id)).sort(([a], [b]) => a.localeCompare(b)),
			),
			select_values:
				selects && typeof selects === 'object' && !(selects instanceof Set)
					? Object.fromEntries(
							Object.entries(selects)
								.sort(([a], [b]) => a.localeCompare(b))
								.flatMap(([key, values]) =>
									values instanceof Set ? [[key, [...values].sort()]] : [],
								),
						)
					: {},
		}
	}

	function isRestoredIssue(id: string) {
		const projectId = project.value?.id
		if (!projectId) return false
		const selection = JSON.stringify({ id, ...issueSelection(id) })
		return Object.values(session.read(projectId, 'previous-issue-selection')).some(
			(saved) => saved === selection,
		)
	}

	function removeIssue(id: string) {
		if (!project.value) return
		for (const scope of ['issue-active', 'issues', 'issue-text', 'issue-select', 'issue-order']) {
			session.write(project.value.id, scope, id, undefined)
		}
	}

	const activeIssues = computed(() => {
		const projectV3 = project.value
		if (!projectV3) return []
		const issues = new Map<string, IssueSelection>()
		for (const entry of availableIssues.value) {
			if (session.read(projectV3.id, 'issue-active')[entry.id] !== true) continue
			issues.set(entry.id, {
				issue: entry.controls[0].control.issue,
				active: true,
				keys: new Set(),
				textValues: {},
				selectValues: {},
				missing: [],
			})
		}
		for (const binding of panels.value.values()) {
			for (const section of binding.panel.sections) {
				for (const control of section.controls) {
					if (
						control.childToggleIds ||
						control.disabled ||
						(control.type === 'toggle' && !selected(binding, control))
					)
						continue
					const selection: IssueSelection = issues.get(control.issueId) ?? {
						issue: control.issue,
						active: false,
						keys: new Set<string>(),
						textValues: {},
						selectValues: {},
						missing: [],
					}
					if (missing(binding, control) && control.type !== 'toggle')
						selection.missing.push(control.key)
					if (control.type === 'select')
						selection.selectValues[control.key] = selectValues(binding, control)
					else if (control.type !== 'toggle')
						selection.textValues[control.key] = textValue(binding, control)
					else {
						selection.active = true
						if (control.id !== undefined) selection.keys.add(control.id)
					}
					issues.set(control.issueId, selection)
				}
			}
		}
		const order = new Map(
			Object.keys(session.read(projectV3.id, 'issue-order')).map((id, index) => [id, index]),
		)
		return [...issues]
			.filter(([, { active }]) => active)
			.sort(
				([a, { issue: issueA }], [b, { issue: issueB }]) =>
					(issueA.priority ?? IssuePriority.Default) - (issueB.priority ?? IssuePriority.Default) ||
					(order.get(a) ?? -1) - (order.get(b) ?? -1),
			)
			.map(([id, { issue, keys, missing }]) => {
				const context: ReviewContext = {
					projectV3,
					...reviewData.value,
					selected: {
						items: {},
						issueIds: selectedIssueIds.value,
						toggleIds: [...keys],
					},
					getMarkdownValue: (key, scope = id) => issues.get(scope)?.textValues[key] ?? '',
					getTextValue: (key, scope = id) => issues.get(scope)?.textValues[key] ?? '',
					getSelectValue: (key, scope = id) => issues.get(scope)?.selectValues[key]?.[0] ?? '',
					getSelectValues: (key, scope = id) => issues.get(scope)?.selectValues[key] ?? [],
				}
				const controls = availableIssues.value.find((entry) => entry.id === id)?.controls ?? []
				if (
					!keys.size &&
					!controls.some(({ control }) => control.type === 'toggle' && control.id === undefined)
				) {
					missing.push('toggle')
				}
				return {
					id,
					missing: [...new Set(missing)],
					locations: [...(resolveWithContext(issue.locations, context) ?? [])],
					facets: resolveIssueFacets(issue.facets, context),
					issue: {
						message: resolveWithContext(issue.message, context),
						suggestedStatus: resolveWithContext(issue.suggestedStatus, context),
					},
				}
			})
	})

	const validationErrors = computed(() =>
		activeIssues.value
			.filter(({ id }) => !isRestoredIssue(id))
			.flatMap(({ id, missing }) => missing.map((key) => ({ issueId: id, key }))),
	)

	return {
		previousLinks: computed(() => reviewData.value.previousLinks),
		issueBindings: (issueId: string) =>
			[...panels.value.values()].filter((binding) =>
				binding.panel.sections.some((section) =>
					section.controls.some(
						(control) =>
							control.issueId === issueId &&
							!control.childToggleIds &&
							!control.disabled &&
							selected(binding, control),
					),
				),
			),
		availableIssues,
		addIssue,
		issueSelection,
		isRestoredIssue,
		removeIssue,
		resolve,
		mixed: (binding: ReviewPanelBinding, control: ResolvedIssueControl) =>
			!!control.childToggleIds?.some((id) =>
				selectedToggleIds(binding.projectId, control.issueId).has(id),
			) && !selected(binding, control),
		selected,
		selectedFindingCount,
		textValue,
		selectValues,
		missing,
		write,
		activeIssues,
		validationErrors,
	}
}
