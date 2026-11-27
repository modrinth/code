import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem, ContentType } from '@modrinth/ui'

export function contentKey(
	item: Pick<ContentItem, 'file_path' | 'file_name' | 'id'> | null | undefined,
) {
	return item?.file_path ?? item?.file_name ?? item?.id ?? ''
}

export function normalizeContentType(type: string): ContentType {
	return (type === 'shaderpack' ? 'shader' : type) as ContentType
}

export function analyticsProjectType(type: string) {
	return type === 'shader' ? 'shaderpack' : type
}

export function normalizeContentItem(item: ContentItem): ContentItem {
	return {
		...item,
		id: contentKey(item),
		project_type: normalizeContentType(item.project_type),
	}
}

export function fileNameFromPath(path: string) {
	return path.split('/').pop() ?? path
}

export function matchesContentItem(
	item: ContentItem,
	target: ContentItem,
	originalFileName: string,
	originalFilePath?: string,
) {
	if (item.file_name === originalFileName || item.file_path === originalFilePath) return true

	const projectId = target.project?.id
	if (!projectId || item.project?.id !== projectId) return false

	const versionId = target.version?.id
	return !versionId || item.version?.id === versionId
}

function isBreakingDependency(dependency: Labrinth.Versions.v2.Dependency) {
	return dependency.dependency_type === 'required' || dependency.dependency_type === 'embedded'
}

function dependencyTargetsItem(dependency: Labrinth.Versions.v2.Dependency, item: ContentItem) {
	return (
		(!!dependency.project_id && dependency.project_id === item.project?.id) ||
		('version_id' in dependency &&
			!!dependency.version_id &&
			dependency.version_id === item.version?.id)
	)
}

export function findDependentItems(
	items: ContentItem[],
	remainingItems: ContentItem[],
	versions: Labrinth.Versions.v2.Version[],
) {
	const versionsById = new Map(versions.map((version) => [version.id, version]))
	return remainingItems.flatMap((candidate) => {
		const version = candidate.version?.id ? versionsById.get(candidate.version.id) : undefined
		if (!version) return []
		const dependencies = items.filter(
			(item) =>
				(!!item.project?.id || !!item.version?.id) &&
				version.dependencies?.some(
					(dependency) =>
						isBreakingDependency(dependency) && dependencyTargetsItem(dependency, item),
				),
		)
		return dependencies.length > 0 ? [{ item: candidate, dependencies }] : []
	})
}
