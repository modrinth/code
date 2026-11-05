import type { ProjectFlag } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import { computed, type Ref } from 'vue'

/** Temporary project flags until the project-flags endpoint is available. */
const mockProjectFlags: ProjectFlag[] = [
	{
		key: 'owner-account-age-under-three-days',
		name: 'Owner account is less than 3 days old',
		reason: 'The project owner created their account 2 days ago, below the 3-day threshold.',
	},
	{
		key: 'owner-has-rejected-projects',
		name: 'Owner has previously rejected projects',
		reason: 'The project owner has 2 projects with a rejected status.',
	},
	{
		key: 'project-previous-malware-detection',
		name: 'Project has a previous malware detection',
		reason: 'A file in a previous version was identified as malware and removed.',
		retain_if_removed: true,
	},
	{
		key: 'owner-has-open-reports',
		name: 'Owner has unresolved reports',
		reason: 'There are 3 unresolved reports associated with the project owner.',
	},
	{
		key: 'summary-contains-external-url',
		name: 'Summary contains an external URL',
		reason: 'The summary contains a link to an external download site.',
		location: { review_panel_key: 'summary' },
	},
	{
		key: 'summary-matches-rejected-project',
		name: 'Summary matches a rejected project',
		reason: 'The normalized summary is identical to the summary of a previously rejected project.',
		location: { review_panel_key: 'summary' },
	},
	{
		key: 'description-contains-shortened-url',
		name: 'Description contains a shortened URL',
		reason: 'The description contains a bit.ly URL that obscures the destination.',
		location: { review_panel_key: 'description' },
	},
	{
		key: 'license-custom-without-url',
		name: 'Custom license has no license URL',
		reason: 'The project uses LicenseRef-Custom but does not provide a URL to the license terms.',
		location: { review_panel_key: 'license' },
	},
	{
		key: 'slug-contains-official',
		name: 'Project slug contains "official"',
		reason: 'The slug contains the reserved branding term "official".',
		location: { review_panel_key: 'slug' },
	},
	{
		key: 'title-contains-truman',
		name: 'Project name cannot contain "Truman"',
		reason:
			'The project name matches the prohibited term "Truman" using a case-insensitive comparison.',
		location: { review_panel_key: 'title' },
	},
	{
		key: 'disclosures-missing-external-service',
		name: 'External service is not disclosed',
		reason:
			'An uploaded file references an external analytics endpoint, but no external service is disclosed.',
		location: { review_panel_key: 'disclosures' },
	},
	{
		key: 'version-file-matches-rejected-file',
		name: 'Version file matches a rejected file',
		reason:
			'An uploaded file has the same SHA-512 hash as a file from a previously rejected project.',
		location: { review_panel_key: 'versions' },
	},
]

export const [injectReviewFlags, provideReviewFlags] =
	createContext<ReturnType<typeof createReviewFlags>>('ProjectReviewFlags')

export function createReviewFlags(projectId: Ref<string | undefined>) {
	const flags = computed(() =>
		(projectId.value ? [...mockProjectFlags] : []).sort(
			(a, b) =>
				Number(!!a.location) - Number(!!b.location) ||
				(a.name || a.key).localeCompare(b.name || b.key, 'en'),
		),
	)
	return {
		flags,
		forPanel: (key: string) =>
			flags.value.filter((flag) => flag.location?.review_panel_key === key),
	}
}
