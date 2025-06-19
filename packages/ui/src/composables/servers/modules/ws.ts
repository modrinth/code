import type { JWTAuth } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.js";
import { ServerModule } from "./base.js";

export class WSModule extends ServerModule implements JWTAuth {
  url!: string;
  token!: string;

  async fetch(): Promise<void> {
    const data = await useServersFetch<JWTAuth>(`servers/${this.serverId}/ws`, {}, "ws");
    Object.assign(this, data);
  }
}
