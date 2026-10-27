import type { ContentDiffItem } from './types'

type DependencyDiffType = Extract<ContentDiffItem['type'], 'added' | 'removed' | 'updated'>
type ConfigurationDiffType = Exclude<ContentDiffItem['type'], DependencyDiffType>

const configurationDiffTypes = new Set<ConfigurationDiffType>([
	'modpack_linked',
	'modpack_updated',
	'modpack_unlinked',
	'game_version_updated',
	'loader_updated',
	'config_files_updated',
])

export function isDependencyDiff(
	diff: ContentDiffItem,
): diff is ContentDiffItem & { type: DependencyDiffType } {
	return diff.type === 'added' || diff.type === 'removed' || diff.type === 'updated'
}

export function isConfigurationDiff(
	diff: ContentDiffItem,
): diff is ContentDiffItem & { type: ConfigurationDiffType } {
	return configurationDiffTypes.has(diff.type as ConfigurationDiffType)
}

export function showsExternalWarning(diff: ContentDiffItem, showExternalWarnings?: boolean) {
	return Boolean(
		showExternalWarnings && diff.external && isDependencyDiff(diff) && diff.type !== 'removed',
	)
}
