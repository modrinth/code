import type { Labrinth } from '@modrinth/api-client'

import { createContext } from '#ui/providers/create-context'

export interface UserProfileContext {
	getUser: (userId: string) => Promise<Labrinth.Users.v3.User>
	getProjects: (userId: string) => Promise<Labrinth.Projects.v2.Project[]>
	getOrganizations: (userId: string) => Promise<Labrinth.Organizations.v3.Organization[]>
	getCollections: (userId: string) => Promise<Labrinth.Collections.Collection[]>
	patchUser: (
		userId: string,
		patch: Partial<Pick<Labrinth.Users.v3.User, 'badges' | 'role'>>,
	) => Promise<void>
}

export const [injectUserProfile, provideUserProfile] = createContext<UserProfileContext>(
	'UserProfilePageLayout',
	'userProfileContext',
)
