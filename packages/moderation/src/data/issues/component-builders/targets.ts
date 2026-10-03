import type { Labrinth } from '@modrinth/api-client'
import { defineMessages, type MessageDescriptor } from '@modrinth/ui'

import type { IssueFacet, ReviewContext, WithContext } from './types'

export const issueTargetLabels = defineMessages({
	modify_title: {
		id: 'project-review.issue-target.modify-title',
		defaultMessage: 'Change title',
	},
	modify_slug: {
		id: 'project-review.issue-target.modify-slug',
		defaultMessage: 'Change URL slug',
	},
	modify_summary: {
		id: 'project-review.issue-target.modify-summary',
		defaultMessage: 'Change summary',
	},
	modify_description: {
		id: 'project-review.issue-target.modify-description',
		defaultMessage: 'Change description',
	},
	modify_license: {
		id: 'project-review.issue-target.modify-license',
		defaultMessage: 'Change license',
	},
	modify_icon: {
		id: 'project-review.issue-target.modify-icon',
		defaultMessage: 'Change icon',
	},
	remove_tags: {
		id: 'project-review.issue-target.remove-tags',
		defaultMessage: 'Remove tags',
	},
	modify_links: {
		id: 'project-review.issue-target.modify-links',
		defaultMessage: 'Change links',
	},
	add_gallery_images: {
		id: 'project-review.issue-target.add-gallery-images',
		defaultMessage: 'Add gallery images',
	},
	modify_gallery_image: {
		id: 'project-review.issue-target.modify-gallery-image',
		defaultMessage: 'Change gallery image',
	},
	remove_gallery_images: {
		id: 'project-review.issue-target.remove-gallery-images',
		defaultMessage: 'Remove gallery images',
	},
	remove_project_disclosures: {
		id: 'project-review.issue-target.remove-project-disclosures',
		defaultMessage: 'Remove disclosures',
	},
	modify_project_disclosure: {
		id: 'project-review.issue-target.modify-project-disclosure',
		defaultMessage: 'Change disclosure',
	},
	modify_project_disclosure_note: {
		id: 'project-review.issue-target.modify-project-disclosure-note',
		defaultMessage: 'Change disclosure note',
	},
	version: {
		id: 'project-review.issue-target.version',
		defaultMessage: 'Change version',
	},
	modify_team_member_role: {
		id: 'project-review.issue-target.modify-team-member-role',
		defaultMessage: 'Change team member role',
	},
	modify_server_languages: {
		id: 'project-review.issue-target.modify-server-languages',
		defaultMessage: 'Change server languages',
	},
	modify_server_address: {
		id: 'project-review.issue-target.modify-server-address',
		defaultMessage: 'Change server address',
	},
	acknowledge: {
		id: 'project-review.issue-target.acknowledge',
		defaultMessage: 'Acknowledge',
	},
}) satisfies Record<Labrinth.Threads.v3.ThreadIssueTarget['type'], MessageDescriptor>

type Suggestion = WithContext<string | null | undefined>

function resolveSuggestion(suggestion: Suggestion | undefined, ctx: ReviewContext) {
	return typeof suggestion === 'function' ? suggestion(ctx) : suggestion
}

function textTarget(original: string, suggestion: string | null | undefined) {
	return { original, ...(suggestion === undefined ? {} : { suggestion }) }
}

export const issueTargets = {
	modifyTitle:
		(suggestion?: Suggestion): IssueFacet =>
		(ctx) => ({
			type: 'modify_title',
			value: textTarget(ctx.projectV3.name, resolveSuggestion(suggestion, ctx)),
		}),
	modifySlug:
		(suggestion?: Suggestion): IssueFacet =>
		(ctx) => ({
			type: 'modify_slug',
			value: textTarget(ctx.projectV3.slug ?? '', resolveSuggestion(suggestion, ctx)),
		}),
	modifySummary:
		(suggestion?: Suggestion): IssueFacet =>
		(ctx) => ({
			type: 'modify_summary',
			value: textTarget(ctx.projectV3.summary, resolveSuggestion(suggestion, ctx)),
		}),
	modifyDescription:
		(suggestion?: Suggestion): IssueFacet =>
		(ctx) => ({
			type: 'modify_description',
			value: textTarget(ctx.projectV3.description, resolveSuggestion(suggestion, ctx)),
		}),
	modifyLicense:
		(licenseSuggestion?: Suggestion, urlSuggestion?: Suggestion): IssueFacet =>
		(ctx) => {
			const license = resolveSuggestion(licenseSuggestion, ctx)
			const url = resolveSuggestion(urlSuggestion, ctx)
			return {
				type: 'modify_license',
				value: {
					license: textTarget(ctx.projectV3.license.id, license),
					url: textTarget(ctx.projectV3.license.url ?? '', url),
				},
			}
		},
	modifyIcon:
		(): IssueFacet =>
		({ projectV3 }) => ({
			type: 'modify_icon',
			value: { original_url: projectV3.icon_url ?? null },
		}),
	removeTags:
		(tags: WithContext<string[]>): IssueFacet =>
		(ctx) => ({
			type: 'remove_tags',
			value: { tags: typeof tags === 'function' ? tags(ctx) : tags },
		}),
	modifySelectedLinks:
		(): IssueFacet =>
		({ projectV3, selected }) => {
			const platforms = new Set(
				selected.toggleIds
					.map((id) => id.split(':')[0])
					.filter((id) => id.endsWith('-link'))
					.map((id) => id.slice(0, -'-link'.length)),
			)
			const links = Object.fromEntries(
				[...platforms].map((platform) => [
					platform,
					textTarget(projectV3.link_urls[platform]?.url ?? '', undefined),
				]),
			)
			return { type: 'modify_links', value: { links } }
		},
	addGalleryImages:
		(): IssueFacet =>
		({ projectV3 }) => ({
			type: 'add_gallery_images',
			value: { original_count: projectV3.gallery.length },
		}),
	modifyServerLanguages:
		(suggestion?: WithContext<string[] | null | undefined>): IssueFacet =>
		(ctx) => {
			const suggested = typeof suggestion === 'function' ? suggestion(ctx) : suggestion
			return {
				type: 'modify_server_languages',
				value: {
					original: [...(ctx.projectV3.minecraft_server?.languages ?? [])],
					...(suggested === undefined ? {} : { suggestion: suggested }),
				},
			}
		},
	acknowledge:
		(mode: 'checkbox' | 'reply'): IssueFacet =>
		() => ({
			type: 'acknowledge',
			value: { mode },
		}),
}

export function resolveIssueFacets(
	facets: readonly [IssueFacet, ...IssueFacet[]],
	ctx: ReviewContext,
): Labrinth.Threads.v3.NewThreadIssue['facets'] {
	const [first, ...rest] = facets
	return [{ what: first(ctx) }, ...rest.map((facet) => ({ what: facet(ctx) }))]
}
