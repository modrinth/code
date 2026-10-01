import type { ContentItem, ManagedContentProject, ManagedContentVersion } from '@modrinth/ui'

import {
	get_content_items,
	get_linked_modpack_info,
	type LinkedModpackInfo,
} from '@/helpers/instance'
import type { CacheBehaviour } from '@/helpers/types'

export type InstanceContentData = {
	path: string
	contentItems: ContentItem[]
	modpack: InstanceContentModpackData | null
}

export type InstanceContentModpackData = {
	project: ManagedContentProject
	version: ManagedContentVersion | null
	updateVersionId: string | null
}

export async function loadInstanceContentData(
	path: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<InstanceContentData> {
	const [contentItems, modpackInfo] = await Promise.all([
		get_content_items(path, cacheBehaviour),
		get_linked_modpack_info(path, cacheBehaviour),
	])

	return {
		path,
		contentItems,
		modpack: normalizeLinkedModpackInfo(modpackInfo as LinkedModpackInfo | null | undefined),
	}
}

function normalizeLinkedModpackInfo(
	modpackInfo: LinkedModpackInfo | null | undefined,
): InstanceContentModpackData | null {
	if (!modpackInfo) return null

	return {
		project: {
			...modpackInfo.project,
			slug: modpackInfo.project.slug ?? modpackInfo.project.id,
			icon_url: modpackInfo.project.icon_url ?? undefined,
		},
		version: modpackInfo.version
			? {
					...modpackInfo.version,
					date_published: modpackInfo.version.date_published.toString(),
				}
			: null,
		updateVersionId: modpackInfo.update_version_id,
	}
}
