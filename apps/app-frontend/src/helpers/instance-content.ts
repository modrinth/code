import type {
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '@modrinth/ui'

import {
	get_content_items,
	get_linked_modpack_info,
	type LinkedModpackInfo,
} from '@/helpers/instance'
import { get_categories } from '@/helpers/tags.js'
import type { CacheBehaviour } from '@/helpers/types'

export type InstanceContentData = {
	path: string
	contentItems: ContentItem[] | null
	modpack: InstanceContentModpackData | null
}

export type InstanceContentModpackData = {
	project: ContentModpackCardProject
	version: ContentModpackCardVersion
	owner: ContentOwner | null
	categories: ContentModpackCardCategory[]
	hasUpdate: boolean
	updateVersionId: string | null
}

export async function loadInstanceContentData(
	path: string,
	cacheBehaviour?: CacheBehaviour,
	onError?: (error: Error) => unknown,
): Promise<InstanceContentData> {
	const [contentItems, modpackInfo, allCategories] = await Promise.all([
		get_content_items(path, cacheBehaviour).catch((error) => handleLoadError(error, onError)),
		get_linked_modpack_info(path, cacheBehaviour).catch((error) => handleLoadError(error, onError)),
		get_categories().catch((error) => handleLoadError(error, onError)),
	])

	return {
		path,
		contentItems: (contentItems as ContentItem[] | null | undefined) ?? null,
		modpack: normalizeLinkedModpackInfo(
			modpackInfo as LinkedModpackInfo | null | undefined,
			allCategories as ContentModpackCardCategory[] | null | undefined,
		),
	}
}

function handleLoadError(error: unknown, onError?: (error: Error) => unknown) {
	onError?.(error as Error)
	return null
}

function normalizeLinkedModpackInfo(
	modpackInfo: LinkedModpackInfo | null | undefined,
	allCategories: ContentModpackCardCategory[] | null | undefined,
): InstanceContentModpackData | null {
	if (!modpackInfo) return null

	return {
		project: {
			...modpackInfo.project,
			slug: modpackInfo.project.slug ?? modpackInfo.project.id,
			icon_url: modpackInfo.project.icon_url ?? undefined,
		},
		version: {
			...modpackInfo.version,
			date_published: modpackInfo.version.date_published.toString(),
		},
		owner: modpackInfo.owner
			? {
					...modpackInfo.owner,
					avatar_url: modpackInfo.owner.avatar_url ?? undefined,
				}
			: null,
		categories: resolveLinkedModpackCategories(modpackInfo, allCategories),
		hasUpdate: modpackInfo.has_update,
		updateVersionId: modpackInfo.update_version_id,
	}
}

function resolveLinkedModpackCategories(
	modpackInfo: LinkedModpackInfo,
	allCategories: ContentModpackCardCategory[] | null | undefined,
) {
	if (!allCategories || !modpackInfo.project.categories) return []

	const seen = new Set<string>()
	return allCategories.filter((category) => {
		if (modpackInfo.project.categories.includes(category.name) && !seen.has(category.name)) {
			seen.add(category.name)
			return true
		}
		return false
	})
}
