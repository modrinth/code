/* eslint-disable @typescript-eslint/no-explicit-any */
import { usePyroFetch } from '@/helpers/pyroFetch'
import { defineStore } from 'pinia'
import type { Project, Server, ServerBackup, Version, WSAuth } from '@/types/servers'
import type { SessionToken } from './credentials'
import { ofetch } from 'ofetch'

interface ServerState {
  serverData: Record<string, Server>
  error: Error | null
}

export const useServerStore = defineStore('servers', {
  state: (): ServerState => ({
    serverData: {},
    error: null,
  }),

  actions: {
    async fetchServerData(auth: SessionToken, serverId: string) {
      try {
        const data = await usePyroFetch<Server>(`servers/${serverId}`, {
          session: auth,
        })

        if (data.modpack) {
          const version: Version = await this.fetchModpackVersion(data.modpack)
          const project = await this.fetchProject(version.project_id as Project['id'])

          data.modpack_id = version.id
          data.project = project as Project | null
        }

        const backups = await this.fetchServerBackups(auth, serverId)
        data.backups = backups

        this.serverData[serverId] = data
        this.error = null
      } catch (error) {
        console.error('Error fetching server data:', error)
        this.error = error instanceof Error ? error : new Error('An unknown error occurred')
        throw this.error
      }
    },

    async listServers(auth: SessionToken) {
      try {
        return await usePyroFetch<{ servers: Server[] }>('servers', {
          session: auth,
        })
      } catch (error) {
        console.error('Error listing servers:', error)
        throw this.error
      }
    },

    async fetchModpackVersion(modpackId: string): Promise<Version> {
      try {
        const result = await ofetch(`https://staging-api.modrinth.com/v2/version/${modpackId}`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            'User-Agent': 'Pyro/1.0 (https://pyro.host)',
          },
        })
        return result as Version
      } catch (error) {
        console.error('Error fetching modpack version:', error)
        throw error
      }
    },

    async fetchProject(projectId: string) {
      try {
        const result = await ofetch(`https://staging-api.modrinth.com/v2/project/${projectId}`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            'User-Agent': 'Pyro/1.0 (https://pyro.host)',
          },
        })
        return result as Project
      } catch (error) {
        console.error('Error fetching project:', error)
        throw error
      }
    },

    async fetchServerBackups(auth: SessionToken, serverId: string) {
      try {
        const result = await usePyroFetch<ServerBackup[]>(`servers/${serverId}/backups`, {
          session: auth,
        })
        return result.sort((a, b) => (a.created_at > b.created_at ? -1 : 1))
      } catch (error) {
        console.error('Error fetching server backups:', error)
        throw error
      }
    },

    updateServerData(serverId: string, data: Partial<Server>) {
      if (!this.serverData[serverId]) {
        console.warn(`Attempting to update non-existent server data for server ID: ${serverId}`)
        return
      }
      this.serverData[serverId] = {
        ...this.serverData[serverId],
        ...data,
      }
    },

    async requestWebsocket(auth: SessionToken, serverId: string): Promise<WSAuth> {
      try {
        return await usePyroFetch(`servers/${serverId}/ws`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error requesting websocket:', error)
        throw error
      }
    },

    async sendPowerAction(auth: SessionToken, serverId: string, action: string) {
      try {
        await usePyroFetch(`servers/${serverId}/power`, {
          session: auth,
          method: 'POST',
          body: { action },
        })
      } catch (error) {
        console.error('Error changing power state:', error)
        throw error
      }
    },

    async updateServerName(auth: SessionToken, serverId: string, newName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/name`, {
          session: auth,
          method: 'POST',
          body: { name: newName },
        })

        if (this.serverData[serverId]) {
          this.serverData[serverId] = {
            ...this.serverData[serverId],
            name: newName,
          }
        } else {
          console.warn(
            `Attempting to update name for non-existent server data. Server ID: ${serverId}`,
          )
        }
      } catch (error) {
        console.error('Error updating server name:', error)
        throw error
      }
    },

    async createBackup(auth: SessionToken, serverId: string, backupName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups`, {
          session: auth,
          method: 'POST',
          body: { name: backupName },
        })
      } catch (error) {
        console.error('Error creating backup:', error)
        throw error
      }
    },

    async renameBackup(auth: SessionToken, serverId: string, backupId: string, newName: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}/rename`, {
          session: auth,
          method: 'POST',
          body: { name: newName },
        })
      } catch (error) {
        console.error('Error renaming backup:', error)
        throw error
      }
    },

    async deleteBackup(auth: SessionToken, serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          session: auth,
          method: 'DELETE',
        })
      } catch (error) {
        console.error('Error deleting backup:', error)
        throw error
      }
    },

    async restoreBackup(auth: SessionToken, serverId: string, backupId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/backups/${backupId}/restore`, {
          session: auth,
          method: 'POST',
        })
      } catch (error) {
        console.error('Error restoring backup:', error)
        throw error
      }
    },

    async downloadBackup(auth: SessionToken, serverId: string, backupId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error downloading backup:', error)
        throw error
      }
    },

    async initiateWorldDownload(auth: SessionToken, serverId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/world`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error initiating world download:', error)
        throw error
      }
    },

    async getWorldDownloadURL(auth: SessionToken, serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/download`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error getting world download URL:', error)
        throw error
      }
    },

    async fetchConfigFile(auth: SessionToken, serverId: string, fileName: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/config/${fileName}`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error fetching config file:', error)
        throw error
      }
    },

    async saveConfigFile(auth: SessionToken, serverId: string, fileName: string, data: any) {
      try {
        await usePyroFetch(`servers/${serverId}/config/${fileName}`, {
          session: auth,
          method: 'PUT',
          body: data,
        })
      } catch (error) {
        console.error('Error saving config file:', error)
        throw error
      }
    },

    async checkSubdomainAvailability(auth: SessionToken, subdomain: string) {
      try {
        return await usePyroFetch(`subdomains/${subdomain}/isavailable`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error checking subdomain availability:', error)
        throw error
      }
    },

    async changeSubdomain(auth: SessionToken, serverId: string, subdomain: string) {
      try {
        await usePyroFetch(`servers/${serverId}/subdomain`, {
          session: auth,
          method: 'POST',
          body: { subdomain },
        })
      } catch (error) {
        console.error('Error changing subdomain:', error)
        throw error
      }
    },

    async installMod(auth: SessionToken, serverId: string, projectId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods`, {
          session: auth,
          method: 'POST',
          body: { rinth_ids: { project_id: projectId, version_id: versionId } },
        })
      } catch (error) {
        console.error('Error installing mod:', error)
        throw error
      }
    },

    async removeMod(auth: SessionToken, serverId: string, modId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods/${modId}`, {
          session: auth,
          method: 'DELETE',
        })
      } catch (error) {
        console.error('Error removing mod:', error)
        throw error
      }
    },

    async reinstallMod(auth: SessionToken, serverId: string, modId: string, versionId: string) {
      try {
        await usePyroFetch(`servers/${serverId}/mods/${modId}`, {
          session: auth,
          method: 'POST',
          body: { version_id: versionId },
        })
      } catch (error) {
        console.error('Error reinstalling mod:', error)
        throw error
      }
    },

    async reinstallServer(
      auth: SessionToken,
      serverId: string,
      projectId: string,
      versionId: string,
    ) {
      try {
        await usePyroFetch(`servers/${serverId}/reinstall`, {
          session: auth,
          method: 'POST',
          body: { project_id: projectId, version_id: versionId },
        })
      } catch (error) {
        console.error('Error reinstalling server:', error)
        throw error
      }
    },

    async suspendServer(auth: SessionToken, serverId: string, status: boolean) {
      try {
        await usePyroFetch(`servers/${serverId}/suspend`, {
          session: auth,
          method: 'POST',
          body: { suspended: status },
        })
      } catch (error) {
        console.error('Error suspending server:', error)
        throw error
      }
    },

    async getFileApiInfo(auth: SessionToken, serverId: string) {
      try {
        return await usePyroFetch(`servers/${serverId}/fs`, {
          session: auth,
        })
      } catch (error) {
        console.error('Error getting file api info:', error)
        throw error
      }
    },

    async listDirContents(
      auth: SessionToken,
      data: any,
      path: string,
      page: number,
      pageSize: number,
    ) {
      try {
        return (await usePyroFetch(`/list?path=${path}&page=${page}&page_size=${pageSize}`, {
          session: auth,
          override: data,
        })) as {
          current: number
          items: {
            count: number
            created: number
            modified: number
            name: string
            path: string
            type: 'directory' | 'file'
            size: number
          }[]
          total: number
        }
      } catch (error) {
        console.error('Error listing dir contents:', error)
        throw error
      }
    },

    createFileOrFolder(
      auth: SessionToken,
      data: any,
      path: string,
      name: string,
      type: 'file' | 'directory',
    ) {
      return usePyroFetch(`/create?path=${path}&type=${type}`, {
        session: auth,
        method: 'POST',
        override: data,
      })
    },

    renameFileOrFolder(auth: SessionToken, data: any, path: string, name: string) {
      return usePyroFetch(`/rename?path=${path}&name=${name}`, {
        session: auth,
        method: 'PUT',
        override: data,
      })
    },

    moveFileOrFolder(auth: SessionToken, data: any, path: string, newPath: string) {
      return usePyroFetch(`/move`, {
        session: auth,
        method: 'PUT',
        override: data,
        body: {
          source: path,
          destination: newPath,
        },
      })
    },

    deleteFileOrFolder(auth: SessionToken, data: any, path: string, recursive: boolean) {
      return usePyroFetch(`/delete?path=${path}&recursive=${recursive}`, {
        session: auth,
        method: 'DELETE',
        override: data,
      })
    },

    downloadFile(auth: SessionToken, data: any, path: string) {
      return usePyroFetch(`/download?path=${path}`, {
        session: auth,
        override: data,
      })
    },

    clearError() {
      this.error = null
    },
  },

  getters: {
    getServerData:
      (state) =>
      (serverId: string): Server | undefined =>
        state.serverData[serverId],
    hasError: (state): boolean => state.error !== null,
  },
})
