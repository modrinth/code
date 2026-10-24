import type { Labrinth } from '@modrinth/api-client'
import { defineMessages, type MessageDescriptor } from '@modrinth/ui'

import type { IssueAction, ReviewContext, WithContext } from './types'

export const issueTargetLabels = defineMessages({
	mark_addressed: {
		id: 'project-review.issue-target.mark-addressed',
		defaultMessage: 'Mark addressed',
	},
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
	remove_icon: {
		id: 'project-review.issue-target.remove-icon',
		defaultMessage: 'Remove icon',
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
	add_project_disclosures: {
		id: 'project-review.issue-target.add-project-disclosures',
		defaultMessage: 'Add disclosures',
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

function optionalTextTarget(
	original: string | null | undefined,
	suggestion: string | null | undefined,
) {
	return { original: original ?? null, ...(suggestion === undefined ? {} : { suggestion }) }
}

export const issueTargets = {
	markAddressed: (): IssueAction => () => ({ type: 'mark_addressed' }),
	modifyTitle:
		(suggestion?: Suggestion): IssueAction =>
		(ctx) => ({
			type: 'modify_title',
			value: textTarget(ctx.projectV3.name, resolveSuggestion(suggestion, ctx)),
		}),
	modifySlug:
		(suggestion?: Suggestion): IssueAction =>
		(ctx) => ({
			type: 'modify_slug',
			value: textTarget(ctx.projectV3.slug ?? '', resolveSuggestion(suggestion, ctx)),
		}),
	modifySummary:
		(suggestion?: Suggestion): IssueAction =>
		(ctx) => ({
			type: 'modify_summary',
			value: textTarget(ctx.projectV3.summary, resolveSuggestion(suggestion, ctx)),
		}),
	modifyDescription:
		(suggestion?: Suggestion): IssueAction =>
		(ctx) => ({
			type: 'modify_description',
			value: textTarget(ctx.projectV3.description, resolveSuggestion(suggestion, ctx)),
		}),
	modifyLicense:
		(licenseSuggestion?: Suggestion, urlSuggestion?: Suggestion): IssueAction =>
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
		(): IssueAction =>
		({ projectV3 }) => ({
			type: 'modify_icon',
			value: { original_url: projectV3.icon_url ?? null },
		}),
	removeIcon: (): IssueAction => () => ({ type: 'remove_icon' }),
	removeTags:
		(tags: WithContext<string[]>): IssueAction =>
		(ctx) => ({
			type: 'remove_tags',
			value: { tags: typeof tags === 'function' ? tags(ctx) : tags },
		}),
	modifySelectedLinks:
		(
			suggestion?: (platform: string, ctx: ReviewContext) => string | null | undefined,
		): IssueAction =>
		(ctx) => {
			const { projectV3, selected } = ctx
			const platforms = new Set(
				selected.toggleIds
					.map((id) => id.split(':')[0])
					.filter((id) => id.endsWith('-link'))
					.map((id) => id.slice(0, -'-link'.length)),
			)
			const links = Object.fromEntries(
				[...platforms].map((platform) => [
					platform,
					textTarget(projectV3.link_urls[platform]?.url ?? '', suggestion?.(platform, ctx)),
				]),
			)
			return { type: 'modify_links', value: { links } }
		},
	addGalleryImages:
		({ autoResolve = false }: { autoResolve?: boolean } = {}): IssueAction =>
		({ projectV3 }) => ({
			type: 'add_gallery_images',
			value: {
				original_image_ids: projectV3.gallery.flatMap((image) =>
					image.id !== undefined ? [image.id] : [],
				),
				auto_resolve: autoResolve,
			},
		}),
	modifyGalleryImage:
		(
			imageId: WithContext<number>,
			fields: { name?: Suggestion; description?: Suggestion } = {},
		): IssueAction =>
		(ctx) => {
			const id = typeof imageId === 'function' ? imageId(ctx) : imageId
			const image = ctx.projectV3.gallery.find((image) => image.id === id)
			if (!image) throw new Error(`Gallery image ${id} was not found`)
			return {
				type: 'modify_gallery_image',
				value: {
					image_id: id,
					original_url: image.url,
					...('name' in fields
						? { name: optionalTextTarget(image.name, resolveSuggestion(fields.name, ctx)) }
						: {}),
					...('description' in fields
						? {
								description: optionalTextTarget(
									image.description,
									resolveSuggestion(fields.description, ctx),
								),
							}
						: {}),
				},
			}
		},
	removeGalleryImages:
		(imageIds: WithContext<number[]>): IssueAction =>
		(ctx) => {
			const ids = typeof imageIds === 'function' ? imageIds(ctx) : imageIds
			return {
				type: 'remove_gallery_images',
				value: {
					image_ids: ids,
					original_images: ctx.projectV3.gallery.flatMap((image) =>
						image.id !== undefined && ids.includes(image.id)
							? [
									{
										id: image.id,
										url: image.url,
										...(image.name === undefined ? {} : { name: image.name }),
									},
								]
							: [],
					),
				},
			}
		},
	addDisclosures:
		(disclosureTypes: Labrinth.Projects.v3.ProjectDisclosureType[]): IssueAction =>
		() => ({
			type: 'add_project_disclosures',
			value: { disclosure_types: disclosureTypes },
		}),
	removeDisclosures:
		(disclosureTypes: WithContext<Labrinth.Projects.v3.ProjectDisclosureType[]>): IssueAction =>
		(ctx) => ({
			type: 'remove_project_disclosures',
			value: {
				disclosure_types: typeof disclosureTypes === 'function' ? disclosureTypes(ctx) : disclosureTypes,
			},
		}),
	modifyDisclosure:
		(disclosureType: Labrinth.Projects.v3.ProjectDisclosureType): IssueAction =>
		({ disclosures }) => {
			const disclosure = disclosures.find(
				(disclosure) => disclosure.type === disclosureType && !disclosure.deleted_at,
			)
			let original: Labrinth.Threads.v3.ModifyDisclosureTarget['value']['metadata']['original'] =
				null
			if (disclosure) {
				const {
					type,
					set_by_moderator,
					lock_status,
					updated_at,
					updated_by,
					deleted_at,
					...metadata
				} = disclosure
				original = metadata
			}
			return {
				type: 'modify_project_disclosure',
				value: { disclosure_type: disclosureType, metadata: { original } },
			} as Labrinth.Threads.v3.ModifyDisclosureTarget
		},
	modifyDisclosureNote:
		(
			disclosureType: Labrinth.Projects.v3.ProjectDisclosureType,
			suggestion?: Suggestion,
		): IssueAction =>
		(ctx) => {
			const disclosure = ctx.disclosures.find(
				(disclosure) => disclosure.type === disclosureType && !disclosure.deleted_at,
			)
			return {
				type: 'modify_project_disclosure_note',
				value: {
					disclosure_type: disclosureType,
					note: optionalTextTarget(
						disclosure && 'note' in disclosure ? disclosure.note : null,
						resolveSuggestion(suggestion, ctx),
					),
				},
			}
		},
	modifyTeamMemberRole:
		(member: WithContext<Labrinth.Projects.v3.TeamMember>, suggestion?: Suggestion): IssueAction =>
		(ctx) => {
			const resolved = typeof member === 'function' ? member(ctx) : member
			return {
				type: 'modify_team_member_role',
				value: {
					team_id: resolved.team_id,
					user_id: resolved.user.id,
					role: textTarget(resolved.role, resolveSuggestion(suggestion, ctx)),
				},
			}
		},
	modifyServerLanguages:
		(suggestion?: WithContext<string[] | null | undefined>): IssueAction =>
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
	modifyServerAddress:
		(platform: 'minecraft_java' | 'minecraft_bedrock', suggestion?: Suggestion): IssueAction =>
		(ctx) => ({
			type: 'modify_server_address',
			value: {
				platform,
				address: textTarget(
					ctx.projectV3[`${platform}_server`]?.address ?? '',
					resolveSuggestion(suggestion, ctx),
				),
			},
		}),
	acknowledge:
		(mode: 'checkbox' | 'reply'): IssueAction =>
		() => ({
			type: 'acknowledge',
			value: { mode },
		}),
}

export function resolveIssueActions(
	actions: WithContext<readonly IssueAction[]> | undefined,
	ctx: ReviewContext,
): Labrinth.Threads.v3.NewThreadIssue['facets'] {
	const [first, ...rest] = (typeof actions === 'function' ? actions(ctx) : actions) ?? []
	if (!first) return [{ what: issueTargets.markAddressed()(ctx) }]
	return [{ what: first(ctx) }, ...rest.map((action) => ({ what: action(ctx) }))]
}
