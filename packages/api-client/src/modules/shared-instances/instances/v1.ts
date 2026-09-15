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

	public async getForUser(userId: string): Promise<string[]> {
		return this.client.request<string[]>('/instances', {
			api: 'sharedinstances',
			version: 1,
			method: 'GET',
			params: { user: userId },
		})
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

	public async inviteUsers(instanceId: string, userIds: string[]): Promise<{ failed: string[] }> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/users`, {
			api: 'sharedinstances', version: 1, method: 'POST', body: { user_ids: userIds }, retry: false,
		})
	}

	public async removeUsers(instanceId: string, userIds: string[]): Promise<void> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/users`, {
			api: 'sharedinstances', version: 1, method: 'DELETE', body: { user_ids: userIds },
		})
	}

	public async downloadMrpack(instanceId: string, version: number): Promise<Blob> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/versions/${version}/mrpack`, {
			api: 'sharedinstances', version: 1, method: 'GET', responseType: 'blob', timeout: 600_000,
		})
	}
}
