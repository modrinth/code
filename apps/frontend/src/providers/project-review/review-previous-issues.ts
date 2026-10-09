import type { Labrinth } from '@modrinth/api-client'
import { IssuePriority } from '@modrinth/moderation/src/data/issues'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import { createContext, useVIntl } from '@modrinth/ui'
import { computed, type Ref, watch } from 'vue'

import type { ReviewTarget } from './review'
import type { createReviewMessages } from './review-messages'
import type {
	createReviewPanels,
	ResolvedIssueControl,
	ReviewIssue,
	ReviewPanelBinding,
} from './review-panels'
import type { createReviewSession } from './review-session'

type ThreadIssue = Labrinth.Threads.v3.ThreadIssue
type ThreadIssueFacet = Labrinth.Threads.v3.ThreadIssueFacet

export const [injectReviewPreviousIssues, provideReviewPreviousIssues] =
	createContext<ReturnType<typeof createReviewPreviousIssues>>('ReviewPreviousIssues')

export function createReviewPreviousIssues(
	project: Ref<Labrinth.Projects.v3.Project | undefined>,
	thread: Ref<Labrinth.Threads.v3.Thread | undefined>,
	wasReviewed: Ref<boolean>,
	session: ReturnType<typeof createReviewSession>,
	panels: ReturnType<typeof createReviewPanels>,
	messages: ReturnType<typeof createReviewMessages>,
) {
	const { formatMessage } = useVIntl()
	const issues = computed(() =>
		wasReviewed.value && project.value?.thread_id === thread.value?.id
			? (thread.value?.issues ?? [])
			: [],
	)

	function reviewIssue(issue: ThreadIssue) {
		const restored = panels.customIssues.value.find(
			(entry) => entry.custom?.previous?.id === issue.id,
		)
		if (restored) return restored
		const why = issue.why
		if (why?.custom) return undefined
		const id = why && typeof why === 'object' && 'issue_id' in why ? why.issue_id : undefined
		return panels.availableIssues.value.find((entry) => !entry.custom && entry.id === id)
	}

	function issueDetails(issue: ThreadIssue) {
		const why = issue.why
		return why && typeof why === 'object' && !Array.isArray(why)
			? (why as Record<string, unknown>)
			: {}
	}

	function issueMessage(issue: ThreadIssue) {
		const message = issueDetails(issue).message
		return typeof message === 'string' ? message : ''
	}

	function messageKey(issue: ThreadIssue) {
		return `previous:${issue.id}`
	}

	function reviewMessage(issue: ThreadIssue) {
		const key = messageKey(issue)
		if (messages.hasIssueOverride(key)) return messages.issueMessage(key)
		const definition = reviewIssue(issue)
		if (!definition || !isApplicable(issue)) return issueMessage(issue)
		if (!panels.activeIssues.value.some((active) => active.id === definition.id))
			return issueMessage(issue)
		return messages.issueMessage(definition.id)
	}

	function isApplicable(issue: ThreadIssue) {
		const projectId = project.value?.id
		if (!projectId) return false
		return session.read(projectId, 'previous-issue-applicability')[issue.id] === true
	}

	function isFacetApplicable(issue: ThreadIssue, facet: ThreadIssueFacet): boolean {
		const projectId = project.value?.id
		if (!projectId) return false
		const stored = session.read(projectId, 'previous-facet-applicability')[facet.id]
		return typeof stored === 'boolean'
			? stored
			: isApplicable(issue) && (isResolved(issue) || facet.verdict !== 'resolved')
	}

	function setFacetApplicable(issue: ThreadIssue, facet: ThreadIssueFacet, applicable: boolean) {
		const projectId = project.value?.id
		if (!projectId || !issue.facets.some(({ id }) => id === facet.id)) return
		const selections = issue.facets.map((entry) => ({
			id: entry.id,
			applicable: entry.id === facet.id ? applicable : isFacetApplicable(issue, entry),
		}))
		if (applicable && !isApplicable(issue)) restoreIssue(issue)
		for (const entry of selections)
			session.write(projectId, 'previous-facet-applicability', entry.id, entry.applicable)
		if (!selections.some((entry) => entry.applicable)) markNoLongerApplicable(issue)
	}

	function cardIssue(issue: ThreadIssue): ReviewIssue {
		const definition = reviewIssue(issue)
		if (definition) return definition
		const details = issueDetails(issue)
		const heading = issueMessage(issue)
			.match(/^##[ \t]+([^\r\n]*)/m)?.[1]
			.trim()
		return {
			id: typeof details.issue_id === 'string' ? details.issue_id : `previous:${issue.id}`,
			title:
				heading ||
				(typeof details.title === 'string'
					? details.title
					: typeof details.issue_id === 'string'
						? details.issue_id.replaceAll('-', ' ')
						: formatMessage(issueTargetLabels[issue.facets[0]?.what.type ?? 'mark_addressed'])),
			category: '',
			priority: issue.why.custom
				? (IssuePriority[issue.why.custom.priority as keyof typeof IssuePriority] ??
					IssuePriority.Bottom)
				: undefined,
			controls: [],
		}
	}

	function issueBindings(issue: ThreadIssue): ReviewPanelBinding[] {
		const definitionBindings = new Map(
			cardIssue(issue).controls.map(({ binding }) => [binding.key, binding]),
		)
		const targets: ReviewTarget[] = []
		const simpleTargets: Partial<
			Record<Labrinth.Threads.v3.ThreadIssueTarget['type'], ReviewTarget>
		> = {
			modify_title: { kind: 'title' },
			modify_slug: { kind: 'slug' },
			modify_summary: { kind: 'summary' },
			modify_description: { kind: 'description' },
			modify_license: { kind: 'license' },
			modify_icon: { kind: 'icon' },
			remove_icon: { kind: 'icon' },
			remove_tags: { kind: 'tags' },
			add_gallery_images: { kind: 'gallery' },
			modify_gallery_image: { kind: 'gallery' },
			remove_gallery_images: { kind: 'gallery' },
			version: { kind: 'versions' },
			modify_server_languages: { kind: 'server' },
			modify_server_address: { kind: 'server' },
		}
		const disclosureKeys: Record<string, string> = {
			ai_content: 'ai',
			ai_functionality: 'ai-functionality',
			advertisements: 'ads',
			epilepsy_triggers: 'photosensitivity',
			system_interactions: 'system-interactions',
			telemetry: 'telemetry',
			derivative_work: 'derivative-content',
			paid_features: 'paid-features',
			archived: 'archive',
		}
		for (const { what } of issue.facets) {
			if (what.type === 'modify_links') {
				for (const key of Object.keys(what.value.links)) targets.push({ kind: 'link', key })
			} else if (what.type === 'modify_gallery_image') {
				targets.push({ kind: 'gallery-image', key: String(what.value.image_id) })
			} else if (what.type === 'remove_gallery_images') {
				for (const id of what.value.image_ids)
					targets.push({ kind: 'gallery-image', key: String(id) })
			} else if (
				what.type === 'add_project_disclosures' ||
				what.type === 'remove_project_disclosures'
			) {
				for (const type of what.value.disclosure_types) {
					const key = disclosureKeys[type]
					if (key) targets.push({ kind: 'disclosure', key })
				}
			} else if (
				what.type === 'modify_project_disclosure' ||
				what.type === 'modify_project_disclosure_note'
			) {
				const key = disclosureKeys[what.value.disclosure_type]
				if (key) targets.push({ kind: 'disclosure', key })
			} else {
				const target = simpleTargets[what.type]
				if (target) targets.push(target)
			}
		}
		if (!targets.length && definitionBindings.size === 1) return [...definitionBindings.values()]
		const bindings = new Map<string, ReviewPanelBinding>()
		for (const target of targets) {
			const binding =
				panels.resolve(target) ??
				(target.kind === 'gallery-image' ? panels.resolve({ kind: 'gallery' }) : undefined)
			if (binding) bindings.set(binding.key, binding)
		}
		return [...bindings.values()]
	}

	const initializedIssues = new Set<string>()
	watch(
		[issues, panels.availableIssues],
		() => {
			const projectId = project.value?.id
			if (!projectId) return
			for (const issue of issues.value) {
				messages.setIssueDefault(messageKey(issue), issueMessage(issue))
				const definition = reviewIssue(issue)
				if (!definition) continue
				messages.setIssueDefault(definition.id, issueMessage(issue))
				const key = `${projectId}:${issue.id}`
				if (initializedIssues.has(key)) continue
				initializedIssues.add(key)
				if (session.read(projectId, 'previous-issue-applicability')[issue.id] !== undefined)
					continue
				if (!panels.activeIssues.value.some(({ id }) => id === definition.id)) continue
				const autoSelected = panels.isRestoredIssue(definition.id)
				const explicitlyAdded = issues.value.some(
					(previous) => reviewIssue(previous)?.id === definition.id && isApplicable(previous),
				)
				if (autoSelected && !explicitlyAdded) {
					panels.removeIssue(definition.id)
					session.write(projectId, 'previous-issue-selection', issue.id, undefined)
				} else if (!autoSelected && issue.verdict !== 'resolved') {
					session.write(projectId, 'previous-issue-applicability', issue.id, true)
				}
			}
		},
		{ immediate: true },
	)

	watch(panels.activeIssues, (active, previous) => {
		const projectId = project.value?.id
		if (!projectId) return
		const activeIds = new Set(active.map(({ id }) => id))
		const previousIds = new Set(previous.map(({ id }) => id))
		for (const issue of issues.value) {
			const id = reviewIssue(issue)?.id
			if (!id) continue
			if (!isResolved(issue) && activeIds.has(id) && !previousIds.has(id)) {
				const alreadyAdded = issues.value.some(
					(previousIssue) =>
						reviewIssue(previousIssue)?.id === id &&
						session.read(projectId, 'previous-issue-applicability')[previousIssue.id] === true,
				)
				if (!alreadyAdded) {
					session.write(projectId, 'previous-issue-applicability', issue.id, true)
					session.write(projectId, 'previous-issue-selection', issue.id, undefined)
				}
			} else if (!activeIds.has(id) && previousIds.has(id)) {
				session.write(projectId, 'previous-issue-applicability', issue.id, false)
			}
		}
	})

	function markNoLongerApplicable(issue: ThreadIssue) {
		const id = project.value?.id
		if (!id || !issues.value.some(({ id }) => id === issue.id)) return
		session.write(id, 'previous-issue-applicability', issue.id, false)
		session.write(id, 'previous-issue-copy', issue.id, undefined)
		for (const facet of issue.facets)
			session.write(id, 'previous-facet-applicability', facet.id, false)
		const definition = reviewIssue(issue)
		if (
			definition &&
			!issues.value.some(
				(previous) => isApplicable(previous) && reviewIssue(previous)?.id === definition.id,
			)
		) {
			panels.removeIssue(definition.id)
		}
	}

	function restoreIssue(issue: ThreadIssue) {
		const projectId = project.value?.id
		if (!projectId || !issues.value.some(({ id }) => id === issue.id)) return
		session.write(projectId, 'previous-issue-applicability', issue.id, true)
		session.write(projectId, 'previous-issue-copy', issue.id, true)
		messages.resetIssueMessage(messageKey(issue))
		for (const facet of issue.facets)
			session.write(projectId, 'previous-facet-applicability', facet.id, true)
		const definition = reviewIssue(issue) ?? panels.restoreCustomIssue(issue)
		if (!definition) return
		if (!definition.custom) {
			messages.resetIssueMessage(definition.id)
			panels.removeIssue(definition.id)
		}
		if (definition.custom) {
			panels.addIssue(definition.id)
			session.write(
				projectId,
				'previous-issue-selection',
				issue.id,
				panels.issueSnapshot(definition.id),
			)
			return
		}
		panels.addIssue(definition.id)
		const selection = issue.why.selection
		if (selection) {
			session.write(projectId, 'issues', definition.id, new Set(selection.toggle_ids ?? []))
			session.write(projectId, 'issue-text', definition.id, { ...selection.text_values })
			session.write(
				projectId,
				'issue-select',
				definition.id,
				Object.fromEntries(
					Object.entries(selection.select_values ?? {}).map(([key, values]) => [
						key,
						new Set(values),
					]),
				),
			)
		} else {
			for (const binding of issueBindings(issue)) {
				const toggles = binding.panel.sections
					.flatMap((section) => section.controls)
					.filter(
						(control) =>
							control.issueId === definition.id && control.type === 'toggle' && !control.disabled,
					)
				if (toggles.length === 1) panels.write(binding, toggles[0], true)
			}
		}
		session.write(
			projectId,
			'previous-issue-selection',
			issue.id,
			panels.issueSnapshot(definition.id),
		)
	}

	function isResolved(issue: ThreadIssue) {
		return issue.verdict === 'resolved'
	}

	function wasControlSelected(issue: ThreadIssue, control: ResolvedIssueControl) {
		if (control.type !== 'toggle' || issue.why?.issue_id !== control.issueId) return false
		const selection = issue.why?.selection
		return (
			control.id === undefined ||
			(Array.isArray(selection?.toggle_ids) && selection.toggle_ids.includes(control.id))
		)
	}

	const reReviewIssues = computed(() => issues.value.filter((issue) => !isResolved(issue)))
	const resolvedIssues = computed(() => issues.value.filter(isResolved))

	const appliedIssues = computed(() => issues.value.filter(isApplicable))
	const associatedIssueIds = computed(
		() =>
			new Set(
				issues.value
					.filter((issue) => !isResolved(issue) || isApplicable(issue))
					.map((issue) => cardIssue(issue).id),
			),
	)
	function needsReplacement(issue: ThreadIssue): boolean {
		if (!isApplicable(issue)) return false
		const id = reviewIssue(issue)?.id
		return (
			session.read(project.value!.id, 'previous-issue-copy')[issue.id] === true ||
			isResolved(issue) ||
			messages.hasIssueOverride(messageKey(issue)) ||
			reviewMessage(issue) !== issueMessage(issue) ||
			(panels.activeIssues.value.some((active) => active.id === id) &&
				!panels.isRestoredIssue(id ?? '')) ||
			issue.facets.some((facet) => isFacetApplicable(issue, facet) && facet.verdict === 'resolved')
		)
	}

	function targetKey(what: Labrinth.Threads.v3.ThreadIssueTarget): string {
		switch (what.type) {
			case 'version':
				return `${what.type}:${what.value.version_id}:${what.value.target.type}:${
					what.value.target.type === 'modify_additional_file_type'
						? what.value.target.value.file_id
						: what.value.target.type === 'remove_additional_files'
							? [...what.value.target.value.file_ids].sort().join(',')
							: ''
				}`
			case 'modify_gallery_image':
				return `${what.type}:${what.value.image_id}`
			case 'modify_team_member_role':
				return `${what.type}:${what.value.team_id}:${what.value.user_id}`
			case 'modify_server_address':
				return `${what.type}:${what.value.platform}`
			case 'modify_project_disclosure':
			case 'modify_project_disclosure_note':
				return `${what.type}:${what.value.disclosure_type}`
			case 'modify_links':
				return `${what.type}:${Object.keys(what.value.links).sort().join(',')}`
			case 'remove_tags':
				return `${what.type}:${[...what.value.tags].sort().join(',')}`
			case 'remove_gallery_images':
				return `${what.type}:${[...what.value.image_ids].sort((a, b) => a - b).join(',')}`
			case 'add_project_disclosures':
			case 'remove_project_disclosures':
				return `${what.type}:${[...what.value.disclosure_types].sort().join(',')}`
			case 'acknowledge':
				return `${what.type}:${what.value.mode}`
			default:
				return what.type
		}
	}

	const recreatedIssues = computed(() =>
		appliedIssues.value
			.filter(needsReplacement)
			.flatMap((issue): Labrinth.Threads.v3.NewThreadIssue[] => {
				const id = reviewIssue(issue)?.id
				const active = panels.activeIssues.value.find((active) => active.id === id)
				const changed = !!active && !panels.isRestoredIssue(active.id)
				const custom = panels.customIssues.value.find((entry) => entry.id === id)?.custom
				const previousFacets = new Map(issue.facets.map((facet) => [targetKey(facet.what), facet]))
				const facets = active
					? active.facets.filter(({ what }) => {
							const previous = previousFacets.get(targetKey(what))
							return changed || !previous || isFacetApplicable(issue, previous)
						})
					: issue.facets
							.filter((facet) => isFacetApplicable(issue, facet))
							.map(({ what }) => ({ what }))
				const [first, ...rest] = facets
				if (!first) return []
				return [
					{
						why: {
							...issueDetails(issue),
							message: reviewMessage(issue),
							...(custom
								? {
										issue_id: custom.id.trim(),
										custom: { priority: custom.priority },
									}
								: {}),
							...(active
								? {
										selection: panels.issueSelection(active.id),
										locations: active.locations,
									}
								: {}),
						},
						facets: [first, ...rest],
					},
				]
			}),
	)
	const facetUpdates = computed(() =>
		issues.value.flatMap((issue) =>
			issue.facets.flatMap(
				(facet): { id: string; data: Labrinth.Threads.v3.EditThreadIssueFacet }[] => {
					if (facet.moderator_verified) return []
					if (!needsReplacement(issue) && isFacetApplicable(issue, facet))
						return facet.user_addressed ? [{ id: facet.id, data: { user_addressed: false } }] : []
					return [{ id: facet.id, data: { moderator_verified: true } }]
				},
			),
		),
	)

	return {
		issues,
		reReviewIssues,
		appliedIssues,
		resolvedIssues,
		associatedIssueIds,
		recreatedIssues,
		isReReviewControl: (control: ResolvedIssueControl) =>
			issues.value.some((issue) => !isResolved(issue) && wasControlSelected(issue, control)),
		reReviewFindingCount: (binding: ReviewPanelBinding) =>
			binding.panel.sections.reduce(
				(count, section) =>
					count +
					section.controls.filter((control) =>
						reReviewIssues.value.some((issue) => wasControlSelected(issue, control)),
					).length,
				0,
			),
		facetUpdates,
		hasUnresolvedFacets: computed(() =>
			issues.value.some((issue) => issue.facets.some((facet) => isFacetApplicable(issue, facet))),
		),
		isFacetApplicable,
		setFacetApplicable,
		messageKey,
		reviewMessage,
		cardIssue,
		issueMessage,
		issueBindings,
		markNoLongerApplicable,
		restoreIssue,
	}
}
