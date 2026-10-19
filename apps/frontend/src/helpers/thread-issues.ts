import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'

export function isThreadIssueVerified(issue: Labrinth.Threads.v3.ThreadIssue): boolean {
	return issue.facets.length > 0 && issue.facets.every((facet) => facet.moderator_verified)
}

export function threadIssueVerdict(
	issue: Labrinth.Threads.v3.ThreadIssue,
): Labrinth.Threads.v3.ThreadIssueVerdict {
	if (!issue.facets.length) return issue.verdict
	if (issue.facets.some((facet) => facet.verdict === 'open')) return 'open'
	return issue.facets.every((facet) => facet.verdict === 'resolved') ? 'resolved' : 'addressed'
}

export function isThreadIssueStatusChange(
	body: Labrinth.Threads.v3.MessageBody,
): body is Extract<Labrinth.Threads.v3.MessageBody, { type: 'status_change' | 'auto_approval' }> {
	return body.type === 'status_change' || body.type === 'auto_approval'
}

/** Verifies outstanding facets when a moderator explicitly approves the whole project. */
export async function verifyThreadIssuesForApproval(
	threadId: string,
	client: Pick<AbstractModrinthClient, 'labrinth'>,
	assertCurrent: () => void,
): Promise<void> {
	const thread = await client.labrinth.threads_v3.getThread(threadId)
	assertCurrent()
	for (const facet of (thread.issues ?? []).flatMap((issue) => issue.facets)) {
		if (facet.moderator_verified) continue
		assertCurrent()
		await client.labrinth.threads_v3.moderator_verified(facet.id)
	}
	assertCurrent()
}

export interface ThreadReply {
	threadId: string
	body: string
	images: string[]
	privateMessage: boolean
}

export async function sendThreadReply(
	reply: ThreadReply,
	client: Pick<AbstractModrinthClient, 'labrinth'>,
	assertCurrent: () => void,
): Promise<void> {
	if (!reply.body.trim()) return
	assertCurrent()
	await client.labrinth.threads_v3.sendMessage(reply.threadId, {
		body: {
			type: 'text',
			body: reply.body,
			private: reply.privateMessage,
			associated_images: reply.images,
		},
	})
	assertCurrent()
}

export function isThreadIssueFacetReadyToAddress(
	facet: Labrinth.Threads.v3.ThreadIssueFacet,
	current: Labrinth.Projects.v3.Project | undefined,
	members: Labrinth.Projects.v3.TeamMember[],
	disclosures?: (Labrinth.Projects.v3.ProjectDisclosure & { deleted_at?: string | null })[],
): boolean {
	if (facet.verdict !== 'open') return true
	const target = facet.what
	if (target.type === 'mark_addressed') return true
	if (!current) return facet.verdict !== 'open'
	switch (target.type) {
		case 'acknowledge':
			return target.value.mode === 'checkbox'
		case 'modify_title':
			return current.name !== target.value.original
		case 'modify_slug':
			return (current.slug ?? '') !== target.value.original
		case 'modify_summary':
			return current.summary !== target.value.original
		case 'modify_description':
			return current.description !== target.value.original
		case 'modify_license':
			return (
				current.license.id !== target.value.license.original ||
				(current.license.url ?? '') !== target.value.url.original
			)
		case 'remove_icon':
			return current.icon_url == null
		case 'modify_icon':
			return (current.icon_url ?? null) !== target.value.original_url
		case 'modify_links':
			return (
				Object.keys(target.value.links).length > 0 &&
				Object.entries(target.value.links).every(
					([platform, { original }]) => (current.link_urls[platform]?.url ?? '') !== original,
				)
			)
		case 'add_gallery_images':
			return current.gallery.some(
				(image) => image.id !== undefined && !target.value.original_image_ids.includes(image.id),
			)
		case 'remove_tags':
			return target.value.tags.every(
				(tag) => !current.categories.includes(tag) && !current.additional_categories.includes(tag),
			)
		case 'remove_gallery_images':
			return target.value.image_ids.every((id) => !current.gallery.some((image) => image.id === id))
		case 'add_project_disclosures':
			return (
				disclosures !== undefined &&
				target.value.disclosure_types.length > 0 &&
				target.value.disclosure_types.every((type) =>
					disclosures.some((disclosure) => disclosure.type === type && !disclosure.deleted_at),
				)
			)
		case 'remove_project_disclosures':
			return (
				disclosures !== undefined &&
				target.value.disclosure_types.length > 0 &&
				target.value.disclosure_types.every(
					(type) =>
						!disclosures.some((disclosure) => disclosure.type === type && !disclosure.deleted_at),
				)
			)
		case 'modify_project_disclosure': {
			const disclosure = disclosures?.find(
				(disclosure) => disclosure.type === target.value.disclosure_type && !disclosure.deleted_at,
			)
			if (!disclosure) return false
			const metadata = Object.fromEntries(
				Object.entries(disclosure).filter(
					([key]) =>
						![
							'type',
							'set_by_moderator',
							'lock_status',
							'updated_at',
							'updated_by',
							'deleted_at',
						].includes(key),
				),
			)
			return (
				JSON.stringify(normalizeDisclosureMetadata(metadata)) !==
				JSON.stringify(normalizeDisclosureMetadata(target.value.metadata.original))
			)
		}
		case 'modify_project_disclosure_note': {
			const disclosure = disclosures?.find(
				(disclosure) => disclosure.type === target.value.disclosure_type && !disclosure.deleted_at,
			)
			return (
				disclosure !== undefined &&
				('note' in disclosure ? (disclosure.note ?? '') : '') !== (target.value.note.original ?? '')
			)
		}
		case 'version':
			return !['remove', 'remove_additional_files'].includes(target.value.target.type)
		case 'modify_gallery_image': {
			const image = current.gallery.find(({ id }) => id === target.value.image_id)
			if (!image) return true
			return (
				(target.value.name !== undefined &&
					(image.name ?? '') !== (target.value.name?.original ?? '')) ||
				(target.value.description !== undefined &&
					(image.description ?? '') !== (target.value.description?.original ?? ''))
			)
		}
		case 'modify_team_member_role': {
			const member = members.find(
				({ user, team_id }) => user.id === target.value.user_id && team_id === target.value.team_id,
			)
			return member ? member.role !== target.value.role.original : true
		}
		case 'modify_server_languages':
			return (
				JSON.stringify([...(current.minecraft_server?.languages ?? [])].sort()) !==
				JSON.stringify([...target.value.original].sort())
			)
		case 'modify_server_address':
			return (
				((target.value.platform === 'minecraft_java'
					? current.minecraft_java_server?.address
					: current.minecraft_bedrock_server?.address) ?? '') !== target.value.address.original
			)
		default:
			return true
	}
}

function normalizeDisclosureMetadata(value: unknown): unknown {
	if (Array.isArray(value)) return value.map(normalizeDisclosureMetadata)
	if (value !== null && typeof value === 'object') {
		return Object.fromEntries(
			Object.entries(value)
				.filter(([key, entry]) => entry != null && !(key === 'note' && entry === ''))
				.sort(([left], [right]) => left.localeCompare(right))
				.map(([key, entry]) => [key, normalizeDisclosureMetadata(entry)]),
		)
	}
	return value
}
