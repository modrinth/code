import type { AbstractModrinthClient, Archon } from '@modrinth/api-client'

import type { ContentDiffItem } from '#ui/layouts/shared/installation-settings/types'

type Change<T> = Archon.Content.v1.SharedContentChange<T>
const before = <T>(change: Change<T>) => 'before' in change ? change.before : undefined
const after = <T>(change: Change<T>) => 'after' in change ? change.after : undefined

export async function resolveServerShareDiff(client: AbstractModrinthClient, diff: Archon.Content.v1.SharedInstancePublishDiff): Promise<ContentDiffItem[]> {
	const versionIds = new Set<string>()
	for (const entry of diff.diffs) {
		if (entry.type !== 'project' && entry.type !== 'modpack') continue
		const values = [before(entry.change), after(entry.change)]
		for (const id of values) if (id) versionIds.add(id)
	}
	const versions = await Promise.all([...versionIds].map((id) => client.labrinth.versions_v3.getVersion(id)))
	const projectIds = [...new Set(versions.map((version) => version.project_id))]
	const projects = projectIds.length ? await client.labrinth.projects_v3.getMultiple(projectIds) : []
	const versionName = (id?: string) => versions.find((version) => version.id === id)?.version_number ?? id
	return diff.diffs.map((entry): ContentDiffItem => {
		if (entry.type === 'external_file') return { type: entry.kind, fileName: encodeURIComponent(entry.file_name) }
		if (entry.type === 'loader') {
			const label = (value?: { name: string; version: string | null }) => value ? [value.name, value.version].filter(Boolean).join(' ') : undefined
			return { type: 'loader_updated', currentVersionName: label(before(entry.change)), newVersionName: label(after(entry.change)) }
		}
		if (entry.type === 'game_version') return { type: 'game_version_updated', currentVersionName: before(entry.change), newVersionName: after(entry.change) }
		const projectId = entry.type === 'project' ? entry.project_id : versions.find((version) => version.id === (after(entry.change) ?? before(entry.change)))?.project_id
		return {
			type: entry.type === 'project' ? entry.change.kind : entry.change.kind === 'added' ? 'modpack_linked' : entry.change.kind === 'removed' ? 'modpack_unlinked' : 'modpack_updated',
			projectName: projects.find((project) => project.id === projectId)?.title ?? projectId,
			currentVersionName: versionName(before(entry.change)),
			newVersionName: versionName(after(entry.change)),
		}
	})
}
