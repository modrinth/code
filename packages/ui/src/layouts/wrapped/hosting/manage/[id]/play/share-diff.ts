import type { Archon } from '@modrinth/api-client'

import type { ContentDiffItem } from '#ui/layouts/shared/installation-settings/types'

type Change<T> = Archon.Content.v1.SharedContentChange<T>
const before = <T>(change: Change<T>) => ('before' in change ? change.before : undefined)
const after = <T>(change: Change<T>) => ('after' in change ? change.after : undefined)

export function resolveServerShareDiff(
	diff: Archon.Content.v1.SharedInstancePublishDiff,
): ContentDiffItem[] {
	const versionName = (id?: string) => {
		if (!id) return undefined
		const version = diff.versions?.[id]
		return version?.version_number ?? version?.name ?? id
	}
	return diff.diffs.map((entry): ContentDiffItem => {
		if (entry.type === 'external_file')
			return { type: entry.kind, fileName: encodeURIComponent(entry.file_name) }
		if (entry.type === 'loader') {
			const label = (value?: { name: string; version: string | null }) =>
				value ? [value.name, value.version].filter(Boolean).join(' ') : undefined
			return {
				type: 'loader_updated',
				currentVersionName: label(before(entry.change)),
				newVersionName: label(after(entry.change)),
			}
		}
		if (entry.type === 'game_version')
			return {
				type: 'game_version_updated',
				currentVersionName: before(entry.change),
				newVersionName: after(entry.change),
			}
		const versionId = after(entry.change) ?? before(entry.change)
		const projectId =
			entry.type === 'project'
				? entry.project_id
				: versionId
					? diff.versions?.[versionId]?.project_id
					: undefined
		return {
			type:
				entry.type === 'project'
					? entry.change.kind
					: entry.change.kind === 'added'
						? 'modpack_linked'
						: entry.change.kind === 'removed'
							? 'modpack_unlinked'
							: 'modpack_updated',
			projectName: projectId ? (diff.projects?.[projectId]?.name ?? projectId) : versionId,
			currentVersionName: versionName(before(entry.change)),
			newVersionName: versionName(after(entry.change)),
		}
	})
}
