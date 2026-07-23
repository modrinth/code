import { AbstractModule } from '../../../core/abstract-module'
import type { SharedInstances } from '../types'

export class SharedInstancesInstancesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'sharedinstances_instances_v1'
	}

	public async get(instanceId: string): Promise<SharedInstances.Instances.v1.Instance> {
		return this.client.request<SharedInstances.Instances.v1.Instance>(
			`/instances/${encodeURIComponent(instanceId)}`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'GET',
			},
		)
	}

	public async getUsers(instanceId: string): Promise<SharedInstances.Instances.v1.InstanceUsers> {
		return this.client.request<SharedInstances.Instances.v1.InstanceUsers>(
			`/instances/${encodeURIComponent(instanceId)}/users`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'GET',
			},
		)
	}

	public async getLatestVersion(
		instanceId: string,
	): Promise<SharedInstances.Instances.v1.InstanceVersion> {
		return this.client.request<SharedInstances.Instances.v1.InstanceVersion>(
			`/instances/${encodeURIComponent(instanceId)}/versions`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'GET',
			},
		)
	}

	public async getVersion(
		instanceId: string,
		version: number,
	): Promise<SharedInstances.Instances.v1.InstanceVersion> {
		return this.client.request<SharedInstances.Instances.v1.InstanceVersion>(
			`/instances/${encodeURIComponent(instanceId)}/versions/${version}`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'GET',
			},
		)
	}
}
