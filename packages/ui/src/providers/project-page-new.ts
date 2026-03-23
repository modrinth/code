import type { Labrinth } from '@modrinth/api-client/src/modules/types'
import type { UseQueryReturnType } from '@tanstack/vue-query'
import type { ComputedRef, Reactive, Ref } from 'vue'

import { createContext } from '.'

/**
 * Computed version with additional display properties
 */
export interface ComputedVersion extends Labrinth.Versions.v3.Version {
	displayUrlEnding: string
	primaryFile: Labrinth.Versions.v3.Version['files'][number] | null
	author: Labrinth.Projects.v3.TeamMember | null
}

/**
 * Authenticated user info for permission checks
 */
export interface AuthUser {
	id: string
	username: string
	role: string
	avatar_url?: string
}

/**
 * Context for the new project page system
 */
export interface NewProjectPageContext {
	// Core project data
	project: Ref<Labrinth.Projects.v3.Project>

	// Async loaded data with loading states
	organization: Reactive<UseQueryReturnType<Labrinth.Organizations.v3.Organization | null, Error>>
	members: Reactive<UseQueryReturnType<Labrinth.Projects.v3.TeamMember[], Error>>
	versions: Reactive<UseQueryReturnType<ComputedVersion[], Error>>
	dependencies: Reactive<UseQueryReturnType<Labrinth.Projects.v3.ProjectDependencies, Error>>
	thread: Reactive<UseQueryReturnType<Labrinth.Threads.v3.Thread | null, Error>>

	// Current authenticated user's membership (null if not a member)
	currentMember: ComputedRef<Labrinth.Projects.v3.TeamMember | null>

	// Auth user for permission checks
	authUser: Ref<AuthUser | null>

	// Link generation functions for cross-platform compatibility
	createUserLink: (user: Labrinth.Users.v3.User) => string
	createOrgLink: (org: Labrinth.Organizations.v3.Organization) => string
	createVersionLink: (version: ComputedVersion) => string

	// Refresh functions
	refreshProject: () => Promise<void>
	refreshVersions: () => Promise<void>
	refreshMembers: () => Promise<void>
	refreshThread: () => Promise<void>

	// Project base path for navigation
	basePath: Ref<string>
}

export const [injectNewProjectPageContext, provideNewProjectPageContext] =
	createContext<NewProjectPageContext>('root', 'newProjectPageContext')
