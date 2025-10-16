import type { ModrinthApiProjects } from './projects'

export interface ModrinthApi {
	projects: ModrinthApiProjects
}

export { RestModrinthApi } from './default_impl'
