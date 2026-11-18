import type { Labrinth } from '@modrinth/api-client'
import { getProjectNagCategory, normalizeProjectNagKind, toProjectNag } from '@modrinth/moderation'
import {
	issueLocationRegistry,
	readIssueLocations,
} from '@modrinth/moderation/src/data/issues/component-builders/locations'

import { threadIssueField } from '~/components/ui/project-issue-card/issue-targets'
import { isThreadIssueVerified, threadIssueVerdict } from '~/helpers/thread-issues'

export function projectIssueSettingsPath(field: keyof typeof issueLocationRegistry) {
	const area = issueLocationRegistry[field].area
	return settingsIssuePath(area ? `settings/${area}` : 'settings')
}

function settingsIssuePath(path: string) {
	return path === 'settings/license' ? 'settings' : path
}

export function getSettingsIssueCounts(
	projectType: string,
	nags: readonly Labrinth.Projects.v3.ProjectNag[],
	issues: readonly Labrinth.Threads.v3.ThreadIssue[],
) {
	const counts = new Map<string, { count: number; hasValidationErrors: boolean }>()
	const increment = (path: string, hasValidationErrors = false) => {
		const settingsPath = settingsIssuePath(path)
		const previous = counts.get(settingsPath)
		counts.set(settingsPath, {
			count: (previous?.count ?? 0) + 1,
			hasValidationErrors: hasValidationErrors || (previous?.hasValidationErrors ?? false),
		})
	}

	for (const nag of nags) {
		if (nag.severity === 'suggestion' || !normalizeProjectNagKind(nag.kind)) continue
		const path = toProjectNag(nag, projectType).link?.path
		if (path?.startsWith('settings')) {
			increment(
				path,
				nag.severity === 'required' && getProjectNagCategory(nag.kind) === 'validation',
			)
		}
	}

	for (const issue of issues) {
		if (isThreadIssueVerified(issue) || threadIssueVerdict(issue) !== 'open') continue
		const fields = [
			...readIssueLocations(issue.why?.locations).map((location) => location.field),
			...issue.facets
				.filter((facet) => facet.verdict === 'open' && !facet.moderator_verified)
				.map((facet) => threadIssueField(facet.what)),
		]
		const paths = new Set(
			fields.flatMap((field) => {
				if (!field) return []
				return [projectIssueSettingsPath(field)]
			}),
		)
		for (const path of paths) increment(path)
	}

	return counts
}
