import type { JWTAuth } from "@modrinth/utils";
import { usePyroFetch } from "../pyro-fetch.ts";
import { ServerModule } from "./base.ts";

export class WSModule extends ServerModule implements JWTAuth {
  url!: string;
  token!: string;

  async fetch(): Promise<void> {
    const data = await usePyroFetch<JWTAuth>(`servers/${this.serverId}/ws`, {}, "ws");
    Object.assign(this, data);
  }
}
