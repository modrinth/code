import type { Labrinth } from '@modrinth/api-client'
import { invoke } from '@tauri-apps/api/core'

import type { CacheBehaviour } from '@/helpers/types'

export function getProjectV3Many(
	ids: string[],
	cacheBehaviour?: CacheBehaviour,
): Promise<Array<Labrinth.Projects.v3.Project | null>> {
	return invoke('plugin:cache|get_project_v3_many', { ids, cacheBehaviour })
}
