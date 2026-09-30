import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem } from '@modrinth/ui'

import { get_project_many, get_version_many } from '@/helpers/cache.js'
import type { SharedInstanceInstallPreview } from '@/helpers/install'

export function useSharedInstancePreviewContent() {
	async function load(preview: SharedInstanceInstallPreview): Promise<ContentItem[]> {
		return [
			...preview.externalFiles.map(externalFileContentItem),
			...(await contentItemsFromVersionIds(preview.contentVersionIds)),
		]
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
