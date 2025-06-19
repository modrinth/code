import type { Allocation } from '@modrinth/utils'
import { ServerModule } from './base.js'

export class NetworkModule extends ServerModule {
  allocations: Allocation[] = []

  async fetch(): Promise<void> {
    this.allocations = await this.server.request<Allocation[]>(
      `servers/${this.serverId}/allocations`,
      {},
      'network',
    )
  }

  async reserveAllocation(name: string): Promise<Allocation> {
    return await this.server.request<Allocation>(
      `servers/${this.serverId}/allocations?name=${name}`,
      {
        method: 'POST',
      },
    )
  }

  async updateAllocation(port: number, name: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/allocations/${port}?name=${name}`, {
      method: 'PUT',
    })
  }

  async deleteAllocation(port: number): Promise<void> {
    await this.server.request(`servers/${this.serverId}/allocations/${port}`, {
      method: 'DELETE',
    })
  }

  async checkSubdomainAvailability(subdomain: string): Promise<boolean> {
    const result = (await this.server.request(`subdomains/${subdomain}/isavailable`)) as {
      available: boolean
    }
    return result.available
  }

  async changeSubdomain(subdomain: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/subdomain`, {
      method: 'POST',
      body: { subdomain },
    })
  }
}
