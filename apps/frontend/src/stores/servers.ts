import { defineStore } from "pinia";
import type { Allocation, Project, Server, ServerBackup } from "~/types/servers";
const config = true;

interface ServerState {
  serverData: Record<string, Server>;
  fileAPIAuth: Record<string, any>;
  error: Error | null;
}

const constructServerProperties = (properties: any): string => {
  let fileContent = `#Minecraft server properties\n#${new Date().toUTCString()}\n`;

  for (const [key, value] of Object.entries(properties)) {
    if (typeof value === "object") {
      fileContent += `${key}=${JSON.stringify(value)}\n`;
    } else if (typeof value === "boolean") {
      fileContent += `${key}=${value ? "true" : "false"}\n`;
    } else {
      fileContent += `${key}=${value}\n`;
    }
  }

  return fileContent;
};

export const useServerStore = defineStore("servers", {
  state: (): ServerState => ({
    serverData: {},
    fileAPIAuth: {},
    error: null,
  }),

  actions: {
    async retryWithAuth(serverId: string, requestFn: () => Promise<any>) {
      try {
        console.log("retrying with auth");
        return await requestFn();
      } catch {
        await this.refreshFileApiInfo(serverId);
        return await requestFn();
      }
    },

    async processImage(data: Server, serverId: string) {
      const image = ref<string | null>(null);
      try {
        const fileData = await this.retryWithAuth(serverId, async () => {
          return await usePyroFetch(`/download?path=/server-icon-original.png`, {
            override: this.fileAPIAuth[serverId],
          });
        });

        if (fileData instanceof Blob) {
          const canvas = document.createElement("canvas");
          const ctx = canvas.getContext("2d");
          const img = new Image();
          img.src = URL.createObjectURL(fileData);
          await new Promise<void>((resolve) => {
            img.onload = () => {
              canvas.width = 512;
              canvas.height = 512;
              ctx?.drawImage(img, 0, 0, 512, 512);
              const dataURL = canvas.toDataURL("image/png");
              data.image = dataURL;
              image.value = dataURL;
              resolve();
            };
          });
        }
      } catch (error) {
        console.error("Error processing server image:", error);
      }

      if (image.value === null && data.project?.icon_url) {
        try {
          const response = await fetch(data.project.icon_url);
          const file = await response.blob();
          const originalfile = new File([file], "server-icon-original.png", {
            type: "image/png",
          });
          const scaledFile = await new Promise<File>((resolve, reject) => {
            const canvas = document.createElement("canvas");
            const ctx = canvas.getContext("2d");
            const img = new Image();
            img.src = URL.createObjectURL(file);
            img.onload = () => {
              canvas.width = 64;
              canvas.height = 64;
              ctx?.drawImage(img, 0, 0, 64, 64);
              canvas.toBlob((blob) => {
                if (blob) {
                  const data = new File([blob], "server-icon.png", { type: "image/png" });
                  resolve(data);
                } else {
                  reject(new Error("Canvas toBlob failed"));
                }
              }, "image/png");
            };
            img.onerror = reject;
          });
          if (scaledFile) {
            await this.uploadFile(serverId, "/server-icon.png", scaledFile);
            await this.uploadFile(serverId, "/server-icon-original.png", originalfile);
          }
        } catch (error) {
          console.error("Error processing server image:", error);
        }
      }
      return image.value;
    },

    async fetchServerData(serverId: string) {
      try {
        await this.confirmFileApiInfo(serverId);
        const data = await usePyroFetch<Server>(`servers/${serverId}`);

        if (data.upstream?.project_id) {
          // @ts-ignore
          const project = await this.fetchProject(data.upstream.project_id);
          data.project = project as Project | null;
        }

        const backups = await this.fetchServerBackups(serverId);
        data.backups = backups;

        const motd = await this.getMotd(serverId);
        if (motd === "A Minecraft Server") {
          await this.setMotd(serverId, `§b${data.project?.title} §f♦ §aModrinth Servers`);
        }
        data.motd = motd;

        data.image = await this.processImage(data, serverId);

        this.serverData[serverId] = data;
        this.error = null;
      } catch (error) {
        console.error("Error fetching server data:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async listServers() {
      try {
        return await usePyroFetch<{ servers: Server[] }>("servers");
      } catch (error) {
        console.error("Error listing servers:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async fetchModpackVersion(modpackId: string): Promise<Project> {
      try {
        const result = await toRaw(useBaseFetch(`version/${modpackId}`, {}, false, config));
        return result as Project;
      } catch (error) {
        console.error("Error fetching modpack version:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async fetchProject(projectId: string) {
      try {
        return await toRaw(useBaseFetch(`project/${projectId}`, {}, false, config));
      } catch (error) {
        console.error("Error fetching project:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async fetchServerBackups(serverId: string) {
      try {
        const result = await usePyroFetch<ServerBackup[]>(`servers/${serverId}/backups`);
        return result.sort((a, b) => (a.created_at > b.created_at ? -1 : 1));
      } catch (error) {
        console.error("Error fetching server backups:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    updateServerData(serverId: string, data: Partial<Server>) {
      if (!this.serverData[serverId]) {
        console.warn(`Attempting to update non-existent server data for server ID: ${serverId}`);
        return;
      }
      this.serverData[serverId] = {
        ...this.serverData[serverId],
        ...data,
      };
    },

    async requestWebsocket(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/ws`);
      } catch (error) {
        console.error("Error requesting websocket:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async sendPowerAction(serverId: string, action: string) {
      try {
        await usePyroFetch(`servers/${serverId}/power`, {
          method: "POST",
          body: { action },
        });
      } catch (error) {
        console.error("Error changing power state:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async updateServerName(serverId: string, newName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/name`, {
          method: "POST",
          body: { name: newName },
        });

        if (this.serverData[serverId]) {
          this.serverData[serverId] = {
            ...this.serverData[serverId],
            name: newName,
          };
        } else {
          console.warn(
            `Attempting to update name for non-existent server data. Server ID: ${serverId}`,
          );
        }
      } catch (error) {
        console.error("Error updating server name:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async createBackup(serverId: string, backupName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups`, {
          method: "POST",
          body: { name: backupName },
        });
      } catch (error) {
        console.error("Error creating backup:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async renameBackup(serverId: string, backupId: string, newName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}/rename`, {
          method: "POST",
          body: { name: newName },
        });
      } catch (error) {
        console.error("Error renaming backup:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async deleteBackup(serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          method: "DELETE",
        });
      } catch (error) {
        console.error("Error deleting backup:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async restoreBackup(serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}/restore`, {
          method: "POST",
        });
      } catch (error) {
        console.error("Error restoring backup:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async downloadBackup(serverId: string, backupId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/backups/${backupId}`);
      } catch (error) {
        console.error("Error downloading backup:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async initiateWorldDownload(serverId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/world`);
      } catch (error) {
        console.error("Error initiating world download:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async getWorldDownloadURL(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/download`);
      } catch (error) {
        console.error("Error getting world download URL:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async fetchConfigFile(serverId: string, fileName: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/config/${fileName}`);
      } catch (error) {
        console.error("Error fetching config file:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async saveConfigFile(serverId: string, fileName: string, data: any) {
      try {
        await usePyroFetch(`servers/${serverId}/config/${fileName}`, {
          method: "PUT",
          body: data,
        });
      } catch (error) {
        console.error("Error saving config file:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async checkSubdomainAvailability(subdomain: string) {
      try {
        return await usePyroFetch(`subdomains/${subdomain}/isavailable`);
      } catch (error) {
        console.error("Error checking subdomain availability:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async changeSubdomain(serverId: string, subdomain: string) {
      try {
        await usePyroFetch(`servers/${serverId}/subdomain`, {
          method: "POST",
          body: { subdomain },
        });
      } catch (error) {
        console.error("Error changing subdomain:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async getStartupSettings(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/startup`, {
          method: "GET",
        });
      } catch (error) {
        console.error("Error updating startup settings:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async updateStartupSettings(
      serverId: string,
      invocation: string,
      jdkVersion: "lts8" | "lts11" | "lts17" | "lts21",
      jdkBuild: "corretto" | "temurin" | "graal",
    ) {
      try {
        await usePyroFetch(`servers/${serverId}/startup`, {
          method: "POST",
          body: {
            invocation: invocation || null,
            jdk_version: jdkVersion || null,
            jdk_build: jdkBuild || null,
          },
        });
      } catch (error) {
        console.error("Error updating startup settings:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async getAllocations(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/allocations`, {
          method: "GET",
        });
      } catch (error) {
        console.error("Error getting allocations:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async reserveAllocation(serverId: string, name: string): Promise<Allocation> {
      try {
        return await usePyroFetch<Allocation>(`servers/${serverId}/allocations?name=${name}`, {
          method: "POST",
        });
      } catch (error) {
        console.error("Error reserving new allocation:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async updateAllocation(serverId: string, port: number, name: string) {
      try {
        await usePyroFetch(`servers/${serverId}/allocations/${port}?name=${name}`, {
          method: "PUT",
        });
      } catch (error) {
        console.error("Error updating allocations:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async deleteAllocation(serverId: string, port: number) {
      try {
        await usePyroFetch(`servers/${serverId}/allocations/${port}`, {
          method: "DELETE",
        });
      } catch (error) {
        console.error("Error deleting allocation:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async getMods(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/mods`);
      } catch (error) {
        console.error("Error getting mods:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async installMod(serverId: string, projectId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods`, {
          method: "POST",
          body: { rinth_ids: { project_id: projectId, version_id: versionId } },
        });
      } catch (error) {
        console.error("Error installing mod:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async removeMod(serverId: string, modId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/deleteMod`, {
          method: "POST",
          body: {
            path: modId,
          },
        });
      } catch (error) {
        console.error("Error removing mod:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async reinstallMod(serverId: string, modId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods/${modId}`, {
          method: "PUT",
          body: { version_id: versionId },
        });
      } catch (error) {
        console.error("Error reinstalling mod:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async reinstallServer(
      serverId: string,
      loader: boolean,
      projectId: string,
      versionId?: string,
    ) {
      try {
        await usePyroFetch(`servers/${serverId}/reinstall`, {
          method: "POST",
          body: loader
            ? { loader: projectId, version: projectId }
            : { project_id: projectId, version_id: versionId },
        });
      } catch (error) {
        console.error("Error reinstalling server:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async suspendServer(serverId: string, status: boolean) {
      try {
        await usePyroFetch(`servers/${serverId}/suspend`, {
          method: "POST",
          body: { suspended: status },
        });
      } catch (error) {
        console.error("Error suspending server:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    // --- FILE API ---

    async refreshFileApiInfo(serverId: string) {
      try {
        this.fileAPIAuth[serverId] = await usePyroFetch(`servers/${serverId}/fs`);
      } catch (error) {
        console.error("Error getting file api info:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async confirmFileApiInfo(serverId: string) {
      if (this.fileAPIAuth[serverId] === null) {
        await this.refreshFileApiInfo(serverId);
      }
    },

    async listDirContents(serverId: string, path: string, page: number, pageSize: number) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/list?path=${path}&page=${page}&page_size=${pageSize}`, {
          override: this.fileAPIAuth[serverId],
        });
      });
    },

    async createFileOrFolder(
      serverId: string,
      path: string,
      // name: string,
      type: "file" | "directory",
    ) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/create?path=${path}&type=${type}`, {
          method: "POST",
          contentType: "application/octet-stream",
          override: this.fileAPIAuth[serverId],
        });
      });
    },

    async uploadFile(serverId: string, path: string, file: File) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/create?path=${path}&type=file`, {
          method: "POST",
          contentType: "application/octet-stream",
          body: file,
          override: this.fileAPIAuth[serverId],
        });
      });
    },

    async renameFileOrFolder(serverId: string, path: string, name: string) {
      await this.confirmFileApiInfo(serverId);
      const pathName = path.split("/").slice(0, -1).join("/") + "/" + name;
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/move`, {
          method: "POST",
          override: this.fileAPIAuth[serverId],
          body: {
            source: path,
            destination: pathName,
          },
        });
      });
    },

    async updateFile(serverId: string, path: string, content: string) {
      await this.confirmFileApiInfo(serverId);
      const octetStream = new Blob([content], { type: "application/octet-stream" });
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/update?path=${path}`, {
          method: "PUT",
          contentType: "application/octet-stream",
          body: octetStream,
          override: this.fileAPIAuth[serverId],
        });
      });
    },

    async moveFileOrFolder(serverId: string, path: string, newPath: string) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/move`, {
          method: "POST",
          override: this.fileAPIAuth[serverId],
          body: {
            source: path,
            destination: newPath,
          },
        });
      });
    },

    async deleteFileOrFolder(serverId: string, path: string, recursive: boolean) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        return await usePyroFetch(`/delete?path=${path}&recursive=${recursive}`, {
          method: "DELETE",
          override: this.fileAPIAuth[serverId],
        });
      });
    },

    async downloadFile(serverId: string, path: string) {
      await this.confirmFileApiInfo(serverId);
      return this.retryWithAuth(serverId, async () => {
        const fileData = await usePyroFetch(`/download?path=${path}`, {
          override: this.fileAPIAuth[serverId],
        });

        if (fileData instanceof Blob) {
          return fileData.text();
        }
      });
    },

    async getMotd(serverId: string) {
      try {
        const props = await this.downloadFile(serverId, "/server.properties");
        if (props) {
          const lines = props.split("\n");
          for (const line of lines) {
            if (line.startsWith("motd=")) {
              return line.slice(5);
            }
          }
        }
      } catch {
        return null;
      }
    },

    async setMotd(serverId: string, motd: string) {
      try {
        const props = (await this.fetchConfigFile(serverId, "ServerProperties")) as any;
        if (props) {
          props.motd = motd;
          const newProps = constructServerProperties(props);
          await this.updateFile(serverId, "server.properties", newProps);
        }
      } catch (error) {
        console.error("Error setting motd:", error);
      }
    },

    clearError() {
      this.error = null;
    },
  },

  getters: {
    getServerData:
      (state) =>
      (serverId: string): Server | undefined =>
        state.serverData[serverId],
    hasError: (state): boolean => state.error !== null,
  },
});
