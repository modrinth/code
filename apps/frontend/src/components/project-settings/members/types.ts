import type { Labrinth } from '@modrinth/api-client'

export type MemberDraft = Pick<
	Labrinth.Projects.v3.TeamMember,
	'role' | 'accepted' | 'is_owner' | 'permissions'
> & {
	user: Pick<Labrinth.Users.v3.User, 'id' | 'username' | 'avatar_url'>
	organization_permissions?: number | null
	payouts_split: number | null | undefined
	override?: boolean
	oldOverride?: boolean
	organizationDefaults?: Pick<MemberDraft, 'role' | 'permissions' | 'payouts_split'>
}
