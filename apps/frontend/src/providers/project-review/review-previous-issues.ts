import type { Labrinth } from '@modrinth/api-client'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import { createContext, useVIntl } from '@modrinth/ui'
import { computed, type Ref, watch } from 'vue'

import type { ReviewTarget } from './review'
import type { createReviewMessages } from './review-messages'
import type {
	ResolvedIssueControl,
	ReviewIssue,
	ReviewPanelBinding,
	createReviewPanels,
} from './review-panels'
import type { createReviewSession } from './review-session'

type ThreadIssue = Labrinth.Threads.v3.ThreadIssue

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
		const why = issue.why
		const id = why && typeof why === 'object' && 'issue_id' in why ? why.issue_id : undefined
		return panels.availableIssues.value.find((entry) => entry.id === id)
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
		const id = reviewIssue(issue)?.id
		return id &&
			isApplicable(issue) &&
			panels.activeIssues.value.some((active) => active.id === id) &&
			!panels.isRestoredIssue(id)
			? messages.issueMessage(id)
			: issueMessage(issue)
	}

	function isApplicable(issue: ThreadIssue) {
		const projectId = project.value?.id
		if (!projectId) return false
		return session.read(projectId, 'previous-issue-applicability')[issue.id] === true
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
						: formatMessage(issueTargetLabels[issue.facets[0]?.what.type ?? 'acknowledge'])),
			category: '',
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
			} else if (what.type === 'remove_project_disclosures') {
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
			const binding = panels.resolve(target)
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
		const definition = reviewIssue(issue)
		if (!definition) return
		if (panels.activeIssues.value.some(({ id }) => id === definition.id)) {
			session.write(
				projectId,
				'previous-issue-selection',
				issue.id,
				JSON.stringify({ id: definition.id, ...panels.issueSelection(definition.id) }),
			)
			return
		}
		panels.addIssue(definition.id)
		const stored = issueDetails(issue).selection
		const selection =
			stored && typeof stored === 'object' && !Array.isArray(stored)
				? (stored as Record<string, unknown>)
				: undefined
		const toggleIds = Array.isArray(selection?.toggle_ids)
			? new Set(selection.toggle_ids.filter((value): value is string => typeof value === 'string'))
			: undefined
		const bindings = toggleIds
			? [...new Map(definition.controls.map(({ binding }) => [binding.key, binding])).values()]
			: issueBindings(issue)
		for (const binding of bindings) {
			const toggles = binding.panel.sections
				.flatMap((section) => section.controls)
				.filter(
					(control) =>
						control.issueId === definition.id && control.type === 'toggle' && !control.disabled,
				)
			for (const control of toggles) {
				if (control.type !== 'toggle') continue
				if (
					toggleIds ? control.id !== undefined && toggleIds.has(control.id) : toggles.length === 1
				)
					panels.write(binding, control, true)
			}
		}
		for (const { binding, control } of reviewIssue(issue)?.controls ?? []) {
			if (control.type === 'toggle' || control.disabled) continue
			const values = control.type === 'select' ? selection?.select_values : selection?.text_values
			if (!values || typeof values !== 'object' || Array.isArray(values)) continue
			const value = (values as Record<string, unknown>)[control.key]
			if (
				control.type === 'select' &&
				Array.isArray(value) &&
				value.every((entry) => typeof entry === 'string')
			)
				panels.write(binding, control, value)
			else if (control.type !== 'select' && typeof value === 'string')
				panels.write(binding, control, value)
		}
		session.write(
			projectId,
			'previous-issue-selection',
			issue.id,
			JSON.stringify({ id: definition.id, ...panels.issueSelection(definition.id) }),
		)
	}

	function isResolved(issue: ThreadIssue) {
		return issue.verdict === 'resolved'
	}

	function wasControlSelected(issue: ThreadIssue, control: ResolvedIssueControl) {
		if (control.type !== 'toggle' || issue.why?.issue_id !== control.issueId) return false
		const selection = issue.why?.selection
		return control.id === undefined
			? selection?.active === true
			: Array.isArray(selection?.toggle_ids) && selection.toggle_ids.includes(control.id)
	}

	const reReviewIssues = computed(() =>
		issues.value.filter((issue) => !isApplicable(issue) && !isResolved(issue)),
	)
	const resolvedIssues = computed(() =>
		issues.value.filter((issue) => !isApplicable(issue) && isResolved(issue)),
	)

	const appliedIssues = computed(() => issues.value.filter(isApplicable))
	const associatedIssueIds = computed(
		() =>
			new Set(
				issues.value
					.filter((issue) => !isResolved(issue) || isApplicable(issue))
					.map((issue) => cardIssue(issue).id),
			),
	)
	const recreatedIssues = computed(() =>
		appliedIssues.value
			.filter(isResolved)
			.flatMap((issue): Labrinth.Threads.v3.NewThreadIssue[] => {
				const id = reviewIssue(issue)?.id
				const active = panels.activeIssues.value.find((active) => active.id === id)
				const facets = active?.facets ?? issue.facets.map(({ what }) => ({ what }))
				const [first, ...rest] = facets
				if (!first) return []
				return [
					{
						why: {
							...issueDetails(issue),
							message: reviewMessage(issue),
							...(active ? { selection: panels.issueSelection(active.id) } : {}),
						},
						facets: [first, ...rest],
					},
				]
			}),
	)
	const issueUpdates = computed(() =>
		issues.value
			.filter((issue) => !isResolved(issue))
			.map((issue) => {
				const applicable = isApplicable(issue)
				const id = reviewIssue(issue)?.id
				const active = panels.activeIssues.value.find((active) => active.id === id)
				const changed = applicable && !!active && !panels.isRestoredIssue(active.id)
				const data: Labrinth.Threads.v3.EditThreadIssue = applicable
					? { user_addressed: false }
					: { moderator_verified: true }
				if (changed || messages.hasIssueOverride(messageKey(issue))) {
					data.why = {
						...issueDetails(issue),
						message: reviewMessage(issue),
						...(changed ? { selection: panels.issueSelection(active.id) } : {}),
					}
				}
				if (changed && active.facets) data.facets = active.facets
				return { id: issue.id, data }
			}),
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
		issueUpdates,
		messageKey,
		reviewMessage,
		cardIssue,
		issueMessage,
		issueBindings,
		markNoLongerApplicable,
		restoreIssue,
	}
}
