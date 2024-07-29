import { defineStore } from 'pinia'
import type { Server } from '~/types/servers'

// Define the store using Pinia
export const useServerStore = defineStore('servers', {
  state: () => ({
    serverData: {} as Record<string, Server>
  }),
  actions: {
    async fetchServerData(serverId: string) {
      const auth = await useAuth()
      let data = await usePyroFetch<Server>(auth.value.token, `servers/${serverId}`)

      // Fetch additional project information
      if (data.modpack) {
        const pid: any = await toRaw(useBaseFetch(`version/${await data.modpack}`))
        const project: any = await toRaw(useBaseFetch(`project/${pid.project_id}`))
        // Update server data
        data.modpack_id = pid.id
        data.project = project
      }
      this.serverData[serverId] = data
    },
    updateServerData(serverId: string, data: Partial<Server>) {
      this.serverData[serverId] = {
        ...this.serverData[serverId],
        ...data
      }
    }
  },
  getters: {
    getServerData: (state) => (serverId: string) => state.serverData[serverId]
  }
})