import type { Labrinth } from '@modrinth/api-client'
import { defineMessages } from '@modrinth/ui'

import { issueTargetLabels } from './targets'

type Location = Labrinth.Threads.v3.ThreadIssueLocation

const labels = defineMessages({
	gallery: {
		id: 'project-review.issue-location.gallery',
		defaultMessage: 'Edit gallery images',
	},
	versions: { id: 'project-review.issue-location.versions', defaultMessage: 'Manage versions' },
	disclosures: {
		id: 'project-review.issue-location.disclosures',
		defaultMessage: 'Edit disclosures',
	},
	permissions: {
		id: 'project-review.issue-location.permissions',
		defaultMessage: 'Review permissions',
	},
})

export const issueLocationRegistry = {
	title: { area: '', inline: true, label: issueTargetLabels.modify_title },
	slug: { area: '', inline: true, label: issueTargetLabels.modify_slug },
	summary: { area: '', inline: true, label: issueTargetLabels.modify_summary },
	icon: { area: '', inline: true, label: issueTargetLabels.modify_icon },
	description: { area: 'description', inline: true, label: issueTargetLabels.modify_description },
	license: { area: 'license', inline: true, label: issueTargetLabels.modify_license },
	tags: { area: 'tags', inline: true, label: issueTargetLabels.remove_tags },
	links: { area: 'links', inline: false, label: issueTargetLabels.modify_links },
	gallery: { area: 'gallery', inline: true, label: labels.gallery },
	disclosures: { area: 'disclosures', inline: false, label: labels.disclosures },
	versions: { area: 'versions', inline: false, label: labels.versions },
	members: { area: 'members', inline: false, label: issueTargetLabels.modify_team_member_role },
	server: { area: 'server', inline: false, label: issueTargetLabels.modify_server_address },
	permissions: { area: 'permissions', inline: false, label: labels.permissions },
} satisfies Record<
	Location['field'],
	{ area: string; inline: boolean; label: NonNullable<Location['label']> }
>

export function issueLocation(field: Location['field'], label?: Location['label']): Location {
	return { field, ...(label ? { label } : {}) }
}

/** Ignore unknown or malformed locations on older or externally created issues. */
export function readIssueLocations(value: unknown): Location[] {
	if (!Array.isArray(value)) return []
	const locations = new Map<Location['field'], Location>()
	for (const entry of value) {
		if (!entry || typeof entry !== 'object' || typeof entry.field !== 'string') continue
		if (!Object.hasOwn(issueLocationRegistry, entry.field)) continue
		const field = entry.field as Location['field']
		const label = entry.label
		locations.set(
			field,
			issueLocation(
				field,
				label &&
					typeof label === 'object' &&
					typeof label.id === 'string' &&
					(label.defaultMessage === undefined || typeof label.defaultMessage === 'string')
					? {
							id: label.id,
							...(label.defaultMessage === undefined
								? {}
								: { defaultMessage: label.defaultMessage }),
						}
					: undefined,
			),
		)
	}
	return [...locations.values()]
}
