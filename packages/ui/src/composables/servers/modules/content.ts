import type { Mod, ContentType } from '@modrinth/utils'
import { ServerModule } from './base.js'

export class ContentModule extends ServerModule {
  data: Mod[] = []

  async fetch(): Promise<void> {
    const mods = await this.server.request<Mod[]>(`servers/${this.serverId}/mods`, {}, 'content')
    this.data = mods.sort((a, b) => (a?.name ?? '').localeCompare(b?.name ?? ''))
  }

  async install(contentType: ContentType, projectId: string, versionId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/mods`, {
      method: 'POST',
      body: {
        rinth_ids: { project_id: projectId, version_id: versionId },
        install_as: contentType,
      },
    })
  }

  async remove(path: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/deleteMod`, {
      method: 'POST',
      body: { path },
    })
  }

  async reinstall(replace: string, projectId: string, versionId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/mods/update`, {
      method: 'POST',
      body: { replace, project_id: projectId, version_id: versionId },
    })
  }
}
