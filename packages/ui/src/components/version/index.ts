import type { Labrinth } from '@modrinth/api-client'

export { default as VersionChannelIndicator } from './VersionChannelIndicator.vue'
export { default as VersionDependencyItem } from './VersionDependencyItem.vue'
export { default as VersionFilterControl } from './VersionFilterControl.vue'
export { default as VersionPage } from './VersionPage.vue'
export { default as VersionSummary } from './VersionSummary.vue'

export type DependencyContext = {
	dependency: Labrinth.Versions.v3.Dependency
	project?: Labrinth.Projects.v2.Project
	version?: Labrinth.Versions.v2.Version
}
