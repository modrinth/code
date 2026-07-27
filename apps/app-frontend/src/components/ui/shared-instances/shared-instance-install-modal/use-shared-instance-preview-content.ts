import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem } from '@modrinth/ui'

import { get_project_many, get_version, get_version_many } from '@/helpers/cache.js'
import type { SharedInstanceInstallPreview } from '@/helpers/install'

type VersionDependency = Labrinth.Versions.v2.Dependency & { version_id?: string }

export function useSharedInstancePreviewContent() {
	async function load(preview: SharedInstanceInstallPreview): Promise<ContentItem[]> {
		return [
			...preview.externalFiles.map(externalFileContentItem),
			...(await modpackContentItems(preview)),
			...(await contentItemsFromVersionIds(
				preview.contentVersionIds.filter((id) => id !== preview.modpackVersionId),
			)),
		]
	}

	async function modpackContentItems(preview: SharedInstanceInstallPreview) {
		if (!preview.modpackVersionId) return []
		const version = await get_version(preview.modpackVersionId, 'must_revalidate')
		return await contentItemsFromDependencies(version?.dependencies ?? [])
	}

	async function contentItemsFromDependencies(dependencies: Labrinth.Versions.v2.Dependency[]) {
		const deps = dependencies as VersionDependency[]
		const projectIds = unique(deps.map((dep) => dep.project_id).filter((id): id is string => !!id))
		const versionIds = unique(deps.map((dep) => dep.version_id).filter((id): id is string => !!id))
		const [projects, versions]: [Labrinth.Projects.v2.Project[], Labrinth.Versions.v2.Version[]] =
			await Promise.all([
				projectIds.length ? get_project_many(projectIds, 'must_revalidate') : [],
				versionIds.length ? get_version_many(versionIds, 'must_revalidate') : [],
			])
		const projectMap = new Map(projects.map((project) => [project.id, project]))
		const versionMap = new Map(versions.map((version) => [version.id, version]))

		return deps.map((dependency): ContentItem => {
			const project = dependency.project_id ? projectMap.get(dependency.project_id) : null
			const version = dependency.version_id ? versionMap.get(dependency.version_id) : null
			const fileName =
				version?.files?.[0]?.filename ?? dependency.file_name ?? project?.title ?? 'Unknown'
			return contentItem(
				version?.id ?? project?.id ?? fileName,
				fileName,
				project,
				version,
				!project && !version,
				dependency.project_id ?? fileName,
				dependency.file_name ?? fileName,
			)
		})
	}

	async function contentItemsFromVersionIds(versionIds: string[]) {
		const versions: Labrinth.Versions.v2.Version[] = versionIds.length
			? await get_version_many(unique(versionIds), 'must_revalidate')
			: []
		const projectIds = unique(versions.map((version) => version.project_id).filter(Boolean))
		const projects: Labrinth.Projects.v2.Project[] = projectIds.length
			? await get_project_many(projectIds, 'must_revalidate')
			: []
		const projectMap = new Map(projects.map((project) => [project.id, project]))
		return versions.map((version): ContentItem => {
			const project = projectMap.get(version.project_id)
			const fileName = version.files?.[0]?.filename ?? project?.title ?? version.name ?? 'Unknown'
			return contentItem(
				version.id,
				fileName,
				project,
				version,
				false,
				version.project_id,
				version.name,
			)
		})
	}

	return { load }
}

function contentItem(
	id: string,
	fileName: string,
	project?: Labrinth.Projects.v2.Project | null,
	version?: Labrinth.Versions.v2.Version | null,
	external = false,
	fallbackProjectId = id,
	fallbackTitle = fileName,
	projectType = project?.project_type ?? 'mod',
): ContentItem {
	return {
		id,
		file_name: fileName,
		project_type: projectType,
		has_update: false,
		update_version_id: null,
		external,
		project: {
			id: project?.id ?? fallbackProjectId,
			slug: project?.slug ?? fallbackProjectId,
			title: project?.title ?? fallbackTitle,
			icon_url: project?.icon_url ?? undefined,
		},
		...(version
			? {
					version: {
						id: version.id,
						file_name: fileName,
						version_number: version.version_number ?? undefined,
						date_published: version.date_published ?? undefined,
					},
				}
			: {}),
	}
}

function externalFileContentItem(
	file: SharedInstanceInstallPreview['externalFiles'][number],
): ContentItem {
	return contentItem(
		`external:${file.fileType}:${file.fileName}`,
		file.fileName,
		null,
		null,
		true,
		file.fileName,
		file.fileName,
		file.fileType,
	)
}

function unique<T>(values: T[]) {
	return Array.from(new Set(values))
}
