import { AbstractModule } from '../../../core/abstract-module'

export class LabrinthProjectsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_projects_v2'
	}

	public getProjects(): any {
		this.client.request(...)
	}
}
