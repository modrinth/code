import { pyroFetch } from "../pyro-fetch";
import { ServerModule } from "./base";
import type { JWTAuth } from "@modrinth/utils";

export class WSModule extends ServerModule implements JWTAuth {
  url!: string;
  token!: string;

  async fetch(): Promise<void> {
    const data = await pyroFetch<JWTAuth>(`servers/${this.serverId}/ws`, {}, "ws");
    Object.assign(this, data);
  }
}
