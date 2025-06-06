import type { Mod, ContentType } from "@modrinth/utils";
import { usePyroFetch } from "../pyro-fetch.ts";
import { ServerModule } from "./base.ts";

export class ContentModule extends ServerModule {
  data: Mod[] = [];

  async fetch(): Promise<void> {
    const mods = await usePyroFetch<Mod[]>(`servers/${this.serverId}/mods`, {}, "content");
    this.data = mods.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? ""));
  }

  async install(contentType: ContentType, projectId: string, versionId: string): Promise<void> {
    await usePyroFetch(`servers/${this.serverId}/mods`, {
      method: "POST",
      body: {
        rinth_ids: { project_id: projectId, version_id: versionId },
        install_as: contentType,
      },
    });
  }

  async remove(path: string): Promise<void> {
    await usePyroFetch(`servers/${this.serverId}/deleteMod`, {
      method: "POST",
      body: { path },
    });
  }

  async reinstall(replace: string, projectId: string, versionId: string): Promise<void> {
    await usePyroFetch(`servers/${this.serverId}/mods/update`, {
      method: "POST",
      body: { replace, project_id: projectId, version_id: versionId },
    });
  }
}
