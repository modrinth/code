import type { Labrinth } from '@modrinth/api-client'

export type AccountChoice = {
	id: string
	username: string
	avatarUrl?: string | null
	role?: Labrinth.Users.v2.Role | null
}
