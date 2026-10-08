import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'

export function isThreadIssueVerified(issue: Labrinth.Threads.v3.ThreadIssue): boolean {
	return issue.facets.length > 0 && issue.facets.every((facet) => facet.moderator_verified)
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
	for (const facet of thread.issues.flatMap((issue) => issue.facets)) {
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
		case 'modify_icon':
			return (current.icon_url ?? null) !== target.value.original_url
		case 'modify_links':
			return (
				Object.keys(target.value.links).length > 0 &&
				Object.entries(target.value.links).some(
					([platform, { original }]) => (current.link_urls[platform]?.url ?? '') !== original,
				)
			)
		case 'add_gallery_images':
			return current.gallery.length > target.value.original_count
		case 'remove_tags':
			return target.value.tags.every(
				(tag) => !current.categories.includes(tag) && !current.additional_categories.includes(tag),
			)
		case 'remove_gallery_images':
			return target.value.image_ids.some((id) => !current.gallery.some((image) => image.id === id))
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
