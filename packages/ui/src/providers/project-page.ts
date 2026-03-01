import type { Labrinth } from '@modrinth/api-client/src/modules/types'
import type { Ref } from 'vue'

import { createContext } from '.'

export interface ProjectPageContext {
	// Data refs
	projectV2: Ref<Labrinth.Projects.v2.Project>
	projectV3: Ref<Labrinth.Projects.v3.Project>
	currentMember: Ref<Labrinth.Projects.v3.TeamMember | null>
	allMembers: Ref<Labrinth.Projects.v3.TeamMember[]>
	organization: Ref<Labrinth.Projects.v3.Organization | null>
	// Lazy version loading (client-side only)
	versions: Ref<Labrinth.Versions.v2.Version[] | null>
	versionsLoading: Ref<boolean>
	// Lazy dependencies loading (client-side only)
	dependencies: Ref<Labrinth.Projects.v2.DependencyInfo | null>
	dependenciesLoading: Ref<boolean>

	// Invalidate all project queries (auto-refetches active ones)
	invalidate: () => Promise<void>

	// Lazy loading
	loadVersions: () => void
	loadDependencies: () => void

	// Mutation functions
	patchProject: (data: Record<string, unknown>, quiet?: boolean) => Promise<boolean>
	patchProjectV3: (data: Record<string, unknown>, quiet?: boolean) => Promise<boolean>
	patchIcon: (icon: File) => Promise<boolean>
	setProcessing: () => Promise<void>
	createGalleryItem: (
		file: File,
		title?: string,
		description?: string,
		featured?: boolean,
		ordering?: number,
	) => Promise<boolean>
	editGalleryItem: (
		imageUrl: string,
		title?: string,
		description?: string,
		featured?: boolean,
		ordering?: number,
	) => Promise<boolean>
	deleteGalleryItem: (imageUrl: string) => Promise<boolean>
}

export const [injectProjectPageContext, provideProjectPageContext] =
	createContext<ProjectPageContext>('root', 'projectPageContext')
