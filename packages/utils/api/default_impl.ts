import type { Project, ProjectV3Partial } from '../types'
import type { ModrinthApi } from './index'
import type { ModrinthApiProjects, ProjectEditBody, ProjectV3EditBodyPartial } from './projects'

export class RestModrinthApi implements ModrinthApi {
	projects: ModrinthApiProjects

	constructor(requestApi: (url: string, options?: object) => Promise<Response>) {
		this.projects = new RestModrinthApiProjects(requestApi)
	}
}

class RestModrinthApiProjects implements ModrinthApiProjects {
	constructor(private request: (url: string, options?: object) => Promise<Response>) {}

	async get(id: string): Promise<Project> {
		const res = await this.request(`/v2/project/${id}`)
		return res.json()
	}

	async getV3(id: string): Promise<ProjectV3Partial> {
		const res = await this.request(`/v3/project/${id}`)
		return res.json()
	}

	async edit(id: string, data: ProjectEditBody): Promise<void> {
		await this.request(`/v2/project/${id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(data),
		})
	}

	async editV3(id: string, data: ProjectV3EditBodyPartial): Promise<void> {
		await this.request(`/v3/project/${id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(data),
		})
	}
}
