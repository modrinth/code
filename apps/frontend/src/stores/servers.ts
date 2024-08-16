import { defineStore } from "pinia";
import type { Server } from "~/types/servers";

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
        const auth = await useAuth();
        const data = await usePyroFetch<Server>(auth.value.token, `servers/${serverId}`);

        if (!data) {
          throw new Error("Failed to fetch server data");
        }

        if (data.modpack) {
          const pid = await this.fetchModpackVersion(data.modpack);
          // @ts-ignore
          const project = await this.fetchProject(pid.project_id);

          // @ts-ignore
          data.modpack = pid.id;

          // @ts-ignore
          data.modpack_id = pid.id;

          // @ts-ignore
          data.project = project;
        }

        this.serverData[serverId] = data;
        this.error = null;
      } catch (error) {
        console.error("Error fetching server data:", error);
        this.error = error instanceof Error ? error : new Error("An unknown error occurred");

        throw this.error;
      }
    },

    async fetchModpackVersion(modpackId: string) {
      try {
        return await toRaw(useBaseFetch(`version/${modpackId}`));
      } catch (error) {
        console.error("Error fetching modpack version:", error);
        throw error;
      }
    },

    async fetchProject(projectId: string) {
      try {
        return await toRaw(useBaseFetch(`project/${projectId}`));
      } catch (error) {
        console.error("Error fetching project:", error);
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

    clearError() {
      this.error = null;
    },

    async updateServerName(serverId: string, newName: string) {
      try {
        const auth = await useAuth();
        await usePyroFetch(
          auth.value.token,
          `servers/${serverId}/name`,
          0,
          "POST",
          "application/json",
          { name: newName },
        );

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
  },

  getters: {
    getServerData:
      (state) =>
      (serverId: string): Server | undefined =>
        state.serverData[serverId],
    hasError: (state): boolean => state.error !== null,
  },
});
