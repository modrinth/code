import { $fetch } from "ofetch";
import type { ServerGeneral, Project, PowerAction, JWTAuth } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class GeneralModule extends ServerModule implements ServerGeneral {
  server_id!: string;
  name!: string;
  net!: { ip: string; port: number; domain: string };
  game!: string;
  backup_quota!: number;
  used_backup_quota!: number;
  status!: string;
  suspension_reason!: string;
  loader!: string;
  loader_version!: string;
  mc_version!: string;
  upstream!: {
    kind: "modpack" | "mod" | "resourcepack";
    version_id: string;
    project_id: string;
  } | null;

  motd?: string;
  image?: string;
  project?: Project;
  sftp_username!: string;
  sftp_password!: string;
  sftp_host!: string;
  datacenter?: string;
  notices?: any[];
  node!: { token: string; instance: string };
  flows?: { intro?: boolean };

  async fetch(): Promise<void> {
    const data = await useServersFetch<ServerGeneral>(`servers/${this.serverId}`, {}, "general");

    if (data.upstream?.project_id) {
      const project = await $fetch(
        `https://api.modrinth.com/v2/project/${data.upstream.project_id}`,
      );
      data.project = project as Project;
    }

    if (import.meta.client) {
      data.image = (await this.server.processImage(data.project?.icon_url)) ?? undefined;
    }

    const motd = await this.getMotd();
    if (motd === "A Minecraft Server") {
      await this.setMotd(
        `§b${data.project?.title || data.loader + " " + data.mc_version} §f♦ §aModrinth Servers`,
      );
    }
    data.motd = motd;

    // Copy data to this module
    Object.assign(this, data);
  }

  async updateName(newName: string): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/name`, {
      method: "POST",
      body: { name: newName },
    });
  }

  async power(action: PowerAction): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/power`, {
      method: "POST",
      body: { action },
    });
    await new Promise((resolve) => setTimeout(resolve, 1000));
    await this.fetch(); // Refresh this module
  }

  async reinstall(
    loader: boolean,
    projectId: string,
    versionId?: string,
    loaderVersionId?: string,
    hardReset: boolean = false,
  ): Promise<void> {
    const hardResetParam = hardReset ? "true" : "false";
    if (loader) {
      if (projectId.toLowerCase() === "neoforge") {
        projectId = "NeoForge";
      }
      await useServersFetch(`servers/${this.serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { loader: projectId, loader_version: loaderVersionId, game_version: versionId },
      });
    } else {
      await useServersFetch(`servers/${this.serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { project_id: projectId, version_id: versionId },
      });
    }
  }

  reinstallFromMrpack(
    mrpack: File,
    hardReset: boolean = false,
  ): {
    promise: Promise<void>;
    onProgress: (cb: (p: { loaded: number; total: number; progress: number }) => void) => void;
  } {
    const hardResetParam = hardReset ? "true" : "false";

    const progressSubject = new EventTarget();

    const uploadPromise = (async () => {
      try {
        const auth = await useServersFetch<JWTAuth>(`servers/${this.serverId}/reinstallFromMrpack`);

        await new Promise<void>((resolve, reject) => {
          const xhr = new XMLHttpRequest();

          xhr.upload.addEventListener("progress", (e) => {
            if (e.lengthComputable) {
              progressSubject.dispatchEvent(
                new CustomEvent("progress", {
                  detail: {
                    loaded: e.loaded,
                    total: e.total,
                    progress: (e.loaded / e.total) * 100,
                  },
                }),
              );
            }
          });

          xhr.onload = () =>
            xhr.status >= 200 && xhr.status < 300
              ? resolve()
              : reject(new Error(`[pyroservers] XHR error status: ${xhr.status}`));

          xhr.onerror = () => reject(new Error("[pyroservers] .mrpack upload failed"));
          xhr.onabort = () => reject(new Error("[pyroservers] .mrpack upload cancelled"));
          xhr.ontimeout = () => reject(new Error("[pyroservers] .mrpack upload timed out"));
          xhr.timeout = 30 * 60 * 1000;

          xhr.open("POST", `https://${auth.url}/reinstallMrpackMultiparted?hard=${hardResetParam}`);
          xhr.setRequestHeader("Authorization", `Bearer ${auth.token}`);

          const formData = new FormData();
          formData.append("file", mrpack);
          xhr.send(formData);
        });
      } catch (err) {
        console.error("Error reinstalling from mrpack:", err);
        throw err;
      }
    })();

    return {
      promise: uploadPromise,
      onProgress: (cb: (p: { loaded: number; total: number; progress: number }) => void) =>
        progressSubject.addEventListener("progress", ((e: CustomEvent) =>
          cb(e.detail)) as EventListener),
    };
  }

  async suspend(status: boolean): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/suspend`, {
      method: "POST",
      body: { suspended: status },
    });
  }

  async endIntro(): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/flows/intro`, {
      method: "DELETE",
      version: 1,
    });
    await this.fetch(); // Refresh this module
  }

  async getMotd(): Promise<string | undefined> {
    try {
      const props = await this.server.fs.downloadFile("/server.properties");
      if (props) {
        const lines = props.split("\n");
        for (const line of lines) {
          if (line.startsWith("motd=")) {
            return line.slice(5);
          }
        }
      }
    } catch {
      return undefined;
    }
    return undefined;
  }

  async setMotd(motd: string): Promise<void> {
    try {
      const props = (await this.server.fetchConfigFile("ServerProperties")) as any;
      if (props) {
        props.motd = motd;
        const newProps = this.server.constructServerProperties(props);
        const octetStream = new Blob([newProps], { type: "application/octet-stream" });
        const auth = await useServersFetch<JWTAuth>(`servers/${this.serverId}/fs`);

        await useServersFetch(`/update?path=/server.properties`, {
          method: "PUT",
          contentType: "application/octet-stream",
          body: octetStream,
          override: auth,
        });
      }
    } catch {
      console.error(
        "[Modrinth Servers] [General] Failed to set MOTD due to lack of server properties file.",
      );
    }
  }
}
