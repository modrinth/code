import type { Labrinth } from '@modrinth/api-client'

export type MemberDraft = Pick<
	Labrinth.Projects.v3.TeamMember,
	'user' | 'role' | 'accepted' | 'is_owner' | 'permissions'
> & {
	payouts_split: number | null | undefined
	override?: boolean
	oldOverride?: boolean
}

export type MemberChanges = Partial<
	Pick<MemberDraft, 'role' | 'payouts_split' | 'permissions' | 'override'>
>
