import type { Allocation } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class NetworkModule extends ServerModule {
  allocations: Allocation[] = [];

  async fetch(): Promise<void> {
    this.allocations = await useServersFetch<Allocation[]>(
      `servers/${this.serverId}/allocations`,
      {},
      "network",
    );
  }

  async reserveAllocation(name: string): Promise<Allocation> {
    return await useServersFetch<Allocation>(`servers/${this.serverId}/allocations?name=${name}`, {
      method: "POST",
    });
  }

  async updateAllocation(port: number, name: string): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/allocations/${port}?name=${name}`, {
      method: "PUT",
    });
  }

  async deleteAllocation(port: number): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/allocations/${port}`, {
      method: "DELETE",
    });
  }

  async checkSubdomainAvailability(subdomain: string): Promise<boolean> {
    const result = (await useServersFetch(`subdomains/${subdomain}/isavailable`)) as {
      available: boolean;
    };
    return result.available;
  }

  async changeSubdomain(subdomain: string): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/subdomain`, {
      method: "POST",
      body: { subdomain },
    });
  }
}
