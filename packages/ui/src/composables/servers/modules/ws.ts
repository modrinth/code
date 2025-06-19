import type { JWTAuth } from '@modrinth/utils'
import { ServerModule } from './base.js'

export class WSModule extends ServerModule implements JWTAuth {
  url!: string
  token!: string

  async fetch(): Promise<void> {
    const data = await this.server.request<JWTAuth>(`servers/${this.serverId}/ws`, {}, 'ws')
    Object.assign(this, data)
  }
}
