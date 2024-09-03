import { defineStore } from "pinia";
import type { Project, Server, ServerBackup } from "~/types/servers";

interface ServerState {
  serverData: Record<string, Server>;
  error: Error | null;
}

export const useServerStore = defineStore("servers", {
  state: (): ServerState => ({
    serverData: {},
    error: null,
  }),

  actions: {
    async fetchServerData(serverId: string) {
      try {
        const data = await usePyroFetch<Server>(`servers/${serverId}`);

        if (data.modpack) {
          const pid: Project = await this.fetchModpackVersion(data.modpack);
          // @ts-ignore
          const project = await this.fetchProject(pid.project_id);

          data.modpack_id = pid.id;
          data.project = project as Project | null;
        }

        const backups = await this.fetchServerBackups(serverId);
        data.backups = backups;

        this.serverData[serverId] = data;
        this.error = null;
      } catch (error) {
        console.error("Error fetching server data:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");
        throw this.error;
      }
    },

    async fetchModpackVersion(modpackId: string): Promise<Project> {
      try {
        return await toRaw(usePyroFetch<Project>(`version/${modpackId}`));
      } catch (error) {
        console.error("Error fetching modpack version:", error);
        throw error;
      }
    },

    async fetchProject(projectId: string) {
      try {
        return await toRaw(usePyroFetch(`project/${projectId}`));
      } catch (error) {
        console.error("Error fetching project:", error);
        throw error;
      }
    },

    async fetchServerBackups(serverId: string) {
      try {
        const result = await usePyroFetch<ServerBackup[]>(`servers/${serverId}/backups`);
        return result.sort((a, b) => (a.created_at > b.created_at ? -1 : 1));
      } catch (error) {
        console.error("Error fetching server backups:", error);
        throw error;
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
        throw error;
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
        throw error;
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
        throw error;
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
        throw error;
      }
    },

    async renameBackup(serverId: string, backupId: string, newName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          method: "POST",
          body: { name: newName },
        });
      } catch (error) {
        console.error("Error renaming backup:", error);
        throw error;
      }
    },

    async deleteBackup(serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          method: "DELETE",
        });
      } catch (error) {
        console.error("Error deleting backup:", error);
        throw error;
      }
    },

    async restoreBackup(serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          method: "POST",
        });
      } catch (error) {
        console.error("Error restoring backup:", error);
        throw error;
      }
    },

    async downloadBackup(serverId: string, backupId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/backups/${backupId}`);
      } catch (error) {
        console.error("Error downloading backup:", error);
        throw error;
      }
    },

    async initiateWorldDownload(serverId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/world`);
      } catch (error) {
        console.error("Error initiating world download:", error);
        throw error;
      }
    },

    async getWorldDownloadURL(serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/download`);
      } catch (error) {
        console.error("Error getting world download URL:", error);
        throw error;
      }
    },

    async fetchConfigFile(serverId: string, fileName: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/config/${fileName}`);
      } catch (error) {
        console.error("Error fetching config file:", error);
        throw error;
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
        throw error;
      }
    },

    async checkSubdomainAvailability(subdomain: string) {
      try {
        return await usePyroFetch(`subdomains/${subdomain}/isavailable`);
      } catch (error) {
        console.error("Error checking subdomain availability:", error);
        throw error;
      }
    },

    async changeSubdomain(serverId: string, subdomain: string) {
      try {
        await usePyroFetch(`servers/${serverId}/subdomains`, {
          method: "POST",
          body: { subdomain },
        });
      } catch (error) {
        console.error("Error changing subdomain:", error);
        throw error;
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
        throw error;
      }
    },

    async removeMod(serverId: string, modId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods/${modId}`, {
          method: "DELETE",
        });
      } catch (error) {
        console.error("Error removing mod:", error);
        throw error;
      }
    },

    async reinstallMod(serverId: string, modId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods/${modId}`, {
          method: "POST",
          body: { version_id: versionId },
        });
      } catch (error) {
        console.error("Error reinstalling mod:", error);
        throw error;
      }
    },

    async reinstallServer(serverId: string, projectId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/reinstall`, {
          method: "POST",
          body: { project_id: projectId, version_id: versionId },
        });
      } catch (error) {
        console.error("Error reinstalling server:", error);
        throw error;
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
