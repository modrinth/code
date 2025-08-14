import type { ContentType, Mod } from '@modrinth/utils'

import { useServersFetch } from '../servers-fetch.ts'
import { ServerModule } from './base.ts'

export class ContentModule extends ServerModule {
	data: Mod[] = []

	async fetch(): Promise<void> {
		const mods = await useServersFetch<Mod[]>(`servers/${this.serverId}/mods`, {}, 'content')
		this.data = mods.sort((a, b) => (a?.name ?? '').localeCompare(b?.name ?? ''))
	}

	async install(contentType: ContentType, projectId: string, versionId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/mods`, {
			method: 'POST',
			body: {
				rinth_ids: { project_id: projectId, version_id: versionId },
				install_as: contentType,
			},
		})
	}

	async remove(path: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/deleteMod`, {
			method: 'POST',
			body: { path },
		})
	}

	async reinstall(replace: string, projectId: string, versionId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/mods/update`, {
			method: 'POST',
			body: { replace, project_id: projectId, version_id: versionId },
		})
	}
}
