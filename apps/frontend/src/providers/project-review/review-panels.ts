import type { Labrinth } from '@modrinth/api-client'
import { categoriesReviewPanel } from '@modrinth/moderation/src/data/issues/categories'
import { aggregateCorrections } from '@modrinth/moderation/src/data/issues/component-builders/corrections'
import type {
	Issue,
	Panel,
	PanelNode,
	ReviewContext,
	WithContext,
} from '@modrinth/moderation/src/data/issues/component-builders/types'
import { descriptionReviewPanel } from '@modrinth/moderation/src/data/issues/description'
import {
	adsDisclosureReviewPanel,
	aiDisclosureReviewPanel,
	aiFunctionalityDisclosureReviewPanel,
	archiveDisclosureReviewPanel,
	derivativeContentDisclosureReviewPanel,
	disclosuresReviewPanel,
	paidFeaturesDisclosureReviewPanel,
	photosensitivityDisclosureReviewPanel,
	systemInteractionsDisclosureReviewPanel,
	telemetryDisclosureReviewPanel,
} from '@modrinth/moderation/src/data/issues/disclosures'
import { galleryReviewPanel } from '@modrinth/moderation/src/data/issues/gallery'
import { iconReviewPanel } from '@modrinth/moderation/src/data/issues/icon'
import { licenseReviewPanel } from '@modrinth/moderation/src/data/issues/license'
import {
	bmacReviewPanel,
	discordReviewPanel,
	githubReviewPanel,
	issuesReviewPanel,
	koFiReviewPanel,
	otherReviewPanel,
	patreonReviewPanel,
	paypalReviewPanel,
	siteReviewPanel,
	sourceReviewPanel,
	storeReviewPanel,
	wikiReviewPanel,
} from '@modrinth/moderation/src/data/issues/links'
import { metadataReviewPanel } from '@modrinth/moderation/src/data/issues/metadata'
import { permissionsReviewPanel } from '@modrinth/moderation/src/data/issues/permissions'
import { postApprovalReviewPanel } from '@modrinth/moderation/src/data/issues/post-approval'
import { reReviewReviewPanel } from '@modrinth/moderation/src/data/issues/re-review'
import { reuploadReviewPanel } from '@modrinth/moderation/src/data/issues/reupload'
import { rulesReviewPanel } from '@modrinth/moderation/src/data/issues/rules'
import { statusAlertsReviewPanel } from '@modrinth/moderation/src/data/issues/status-alerts'
import { summaryReviewPanel } from '@modrinth/moderation/src/data/issues/summary'
import { titleReviewPanel } from '@modrinth/moderation/src/data/issues/title'
import { undefinedProjectReviewPanel } from '@modrinth/moderation/src/data/issues/undefined-project'
import { versionsReviewPanel } from '@modrinth/moderation/src/data/issues/versions'
import { createContext } from '@modrinth/ui'
import { computed, type Ref } from 'vue'

import type { ReviewTarget } from './review'
import type { createReviewSession } from './review-session'

const reviewPanels = {
	title: titleReviewPanel,
	summary: summaryReviewPanel,
	description: descriptionReviewPanel,
	'issues-link': issuesReviewPanel,
	'source-link': sourceReviewPanel,
	'wiki-link': wikiReviewPanel,
	'discord-link': discordReviewPanel,
	'site-link': siteReviewPanel,
	'store-link': storeReviewPanel,
	'patreon-link': patreonReviewPanel,
	'bmac-link': bmacReviewPanel,
	'paypal-link': paypalReviewPanel,
	'github-link': githubReviewPanel,
	'ko-fi-link': koFiReviewPanel,
	'other-link': otherReviewPanel,
	categories: categoriesReviewPanel,
	disclosures: disclosuresReviewPanel,
	'ai-disclosure': aiDisclosureReviewPanel,
	'ai-functionality-disclosure': aiFunctionalityDisclosureReviewPanel,
	'ads-disclosure': adsDisclosureReviewPanel,
	'paid-features-disclosure': paidFeaturesDisclosureReviewPanel,
	'telemetry-disclosure': telemetryDisclosureReviewPanel,
	'derivative-content-disclosure': derivativeContentDisclosureReviewPanel,
	'photosensitivity-disclosure': photosensitivityDisclosureReviewPanel,
	'system-interactions-disclosure': systemInteractionsDisclosureReviewPanel,
	'archive-disclosure': archiveDisclosureReviewPanel,
	gallery: galleryReviewPanel,
	icon: iconReviewPanel,
	license: licenseReviewPanel,
	metadata: metadataReviewPanel,
	permissions: permissionsReviewPanel,
	'post-approval': postApprovalReviewPanel,
	're-review': reReviewReviewPanel,
	reupload: reuploadReviewPanel,
	rules: rulesReviewPanel,
	'status-alerts': statusAlertsReviewPanel,
	'undefined-project': undefinedProjectReviewPanel,
	versions: versionsReviewPanel,
} satisfies Record<string, Panel>

interface ResolvedIssueControlOptions {
	disabled: boolean
	required: boolean
	issue: Issue
	issueId: string
	label: string
	tooltip?: string
}

type ResolvedIssueControl = ResolvedIssueControlOptions &
	(
		| { type: 'toggle'; id?: string }
		| {
				type: 'markdown' | 'text'
				key: string
				initial: string
				placeholder?: string
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
	projectId: string
	panel: {
		icon: Panel['icon']
		field?: string
		title: string
		hint: string
		sections: ResolvedPanelSection[]
	}
}

function resolveWithContext<T>(value: WithContext<T>, context: ReviewContext): T {
	return typeof value === 'function' ? (value as (context: ReviewContext) => T)(context) : value
}

export const [injectReviewPanels, provideReviewPanels] =
	createContext<ReturnType<typeof createReviewPanels>>('ProjectReviewPanels')

export function createReviewPanels(
	project: Ref<Labrinth.Projects.v3.Project | undefined>,
	session: ReturnType<typeof createReviewSession>,
	reviewData: Ref<Pick<ReviewContext, 'wasReviewed' | 'permissions'>>,
	definitions: Record<string, Panel> = reviewPanels,
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
		return Object.entries(session.read(projectId, 'issue-active'))
			.filter(([, selected]) => selected === true)
			.map(([id]) => id)
	})

	const panels = computed(() => {
		const bindings = new Map<string, ReviewPanelBinding>()
		const ProjectV3 = project.value
		if (!ProjectV3) return bindings
		const context: ReviewContext = {
			ProjectV3,
			...reviewData.value,
			getTextValue: (id, issueId) => (issueId ? (textValues(ProjectV3.id, issueId)[id] ?? '') : ''),
			getSelectValue: (id, issueId) =>
				issueId ? (readSelectValues(ProjectV3.id, issueId, id)[0] ?? '') : '',
			getSelectValues: (id, issueId) =>
				issueId ? readSelectValues(ProjectV3.id, issueId, id) : [],
			getMarkdownValue: (id, issueId) =>
				issueId ? (textValues(ProjectV3.id, issueId)[id] ?? '') : '',
			selected: {
				issueIds: selectedIssueIds.value,
				toggleIds: [
					...new Set(
						Object.values(session.read(ProjectV3.id, 'issues')).flatMap((keys) =>
							keys instanceof Set ? [...keys] : [],
						),
					),
				],
			},
		}
		for (const [key, panel] of Object.entries(definitions)) {
			if (resolveWithContext(panel.shown, context) === false) continue
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
					const issueId = node.issue.id
					const issueContext: ReviewContext = {
						...context,
						getMarkdownValue: (id, scope = issueId) => textValues(ProjectV3.id, scope)[id] ?? '',
						getTextValue: (id, scope = issueId) => textValues(ProjectV3.id, scope)[id] ?? '',
						getSelectValue: (id, scope = issueId) =>
							readSelectValues(ProjectV3.id, scope, id)[0] ?? '',
						getSelectValues: (id, scope = issueId) => readSelectValues(ProjectV3.id, scope, id),
						selected: {
							issueIds: selectedIssueIds.value,
							toggleIds: [...selectedToggleIds(ProjectV3.id, issueId)],
						},
					}
					if (resolveWithContext(node.shown, issueContext) === false) continue
					const options: ResolvedIssueControlOptions = {
						disabled: resolveWithContext(node.disabled, issueContext) === true,
						issue: node.issue,
						issueId,
						required:
							node.type !== 'toggle' && resolveWithContext(node.required, issueContext) === true,
						label: resolveWithContext(node.label, issueContext),
						tooltip: resolveWithContext(node.tooltip, issueContext),
					}
					let control: ResolvedIssueControl
					if (node.type === 'toggle') {
						control = { ...options, type: 'toggle', id: node.id }
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
								node.type === 'text' ? (resolveWithContext(node.initial, issueContext) ?? '') : '',
							placeholder:
								node.type === 'text'
									? resolveWithContext(node.placeholder, issueContext)
									: undefined,
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
				projectId: ProjectV3.id,
				panel: {
					icon: panel.icon,
					field: panel.field,
					title: resolveWithContext(panel.title, context),
					hint: resolveWithContext(panel.hint, context),
					sections,
				},
			})
		}
		return bindings
	})

	function resolve(target: ReviewTarget) {
		const aliases: Partial<Record<ReviewTarget['kind'], string>> = {
			slug: 'title',
			tags: 'categories',
			compatibility: 'metadata',
			'license-url': 'license',
			'gallery-image': 'gallery',
			version: 'versions',
		}
		if (target.kind === 'disclosure') return panels.value.get(`${target.key}-disclosure`)
		return panels.value.get(
			target.kind === 'link' ? `${target.key}-link` : (aliases[target.kind] ?? target.kind),
		)
	}

	function selected(binding: ReviewPanelBinding, control: ResolvedIssueControl): boolean {
		return isSelected(binding.projectId, control)
	}

	function textValue(binding: ReviewPanelBinding, control: ResolvedIssueControl): string {
		if (control.type !== 'markdown' && control.type !== 'text') return ''
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
		if (current.id === undefined) {
			session.write(binding.projectId, 'issue-active', current.issueId, value || undefined)
			return
		}
		const keys = new Set(selectedToggleIds(binding.projectId, current.issueId))
		if (value) keys.add(current.id)
		else keys.delete(current.id)
		session.write(binding.projectId, 'issues', current.issueId, keys.size ? keys : undefined)
	}

	const activeIssues = computed(() => {
		const ProjectV3 = project.value
		if (!ProjectV3) return []
		const issues = new Map<string, IssueSelection>()
		for (const binding of panels.value.values()) {
			for (const section of binding.panel.sections) {
				for (const control of section.controls) {
					if (control.disabled || (control.type === 'toggle' && !selected(binding, control)))
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
		return [...issues]
			.filter(([, { active }]) => active)
			.map(([id, { issue, keys, missing }]) => {
				const context: ReviewContext = {
					ProjectV3,
					...reviewData.value,
					selected: {
						issueIds: selectedIssueIds.value,
						toggleIds: [...keys],
					},
					getMarkdownValue: (key, scope = id) => issues.get(scope)?.textValues[key] ?? '',
					getTextValue: (key, scope = id) => issues.get(scope)?.textValues[key] ?? '',
					getSelectValue: (key, scope = id) => issues.get(scope)?.selectValues[key]?.[0] ?? '',
					getSelectValues: (key, scope = id) => issues.get(scope)?.selectValues[key] ?? [],
				}
				return {
					id,
					missing: [...new Set(missing)],
					hasCorrections: issue.corrections !== undefined,
					corrections: missing.length ? undefined : resolveWithContext(issue.corrections, context),
					applyCorrections: issue.applyCorrections === true,
					issue: {
						message: resolveWithContext(issue.message, context),
						suggestedStatus: resolveWithContext(issue.suggestedStatus, context),
					},
				}
			})
	})

	const corrections = computed(() => aggregateCorrections(activeIssues.value))
	const correctionsRequested = computed(() =>
		activeIssues.value.some((issue) => issue.applyCorrections),
	)
	const correctionPanels = computed(() => {
		const issueIds = new Set(
			activeIssues.value.filter((issue) => issue.hasCorrections).map((issue) => issue.id),
		)
		return [...panels.value.values()].flatMap((binding) => {
			const sections = binding.panel.sections
				.map((section) => ({
					...section,
					controls: section.controls.filter((control) => issueIds.has(control.issueId)),
				}))
				.filter((section) => section.controls.length > 0)
			return sections.length ? [{ ...binding, panel: { ...binding.panel, sections } }] : []
		})
	})
	const validationErrors = computed(() =>
		activeIssues.value.flatMap(({ id, missing }) => missing.map((key) => ({ issueId: id, key }))),
	)

	return {
		resolve,
		selected,
		textValue,
		selectValues,
		missing,
		write,
		activeIssues,
		corrections,
		correctionsRequested,
		correctionPanels,
		validationErrors,
	}
}
