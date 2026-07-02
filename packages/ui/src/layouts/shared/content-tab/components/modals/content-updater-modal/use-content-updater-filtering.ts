import type { Labrinth } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { computed } from 'vue'

type UseContentUpdaterFilteringOptions = {
	versions: Readonly<Ref<Labrinth.Versions.v2.Version[]>>
	searchQuery: Ref<string>
	isModpack: ComputedRef<boolean>
	incompatibilityWarningMode: ComputedRef<boolean>
	hideIncompatibleState: Ref<boolean>
	selectedVersion: Ref<Labrinth.Versions.v2.Version | null>
	pinnedInitialVersionId: Ref<string | undefined>
	currentVersionId: Readonly<Ref<string>>
	isVersionCompatible: (version: Labrinth.Versions.v2.Version) => boolean
	debug: (message: string, data?: Record<string, unknown>) => void
}

export function useContentUpdaterFiltering({
	versions: sourceVersions,
	searchQuery,
	isModpack,
	incompatibilityWarningMode,
	hideIncompatibleState,
	selectedVersion,
	pinnedInitialVersionId,
	currentVersionId,
	isVersionCompatible,
	debug,
}: UseContentUpdaterFilteringOptions) {
	return computed(() => {
		let versions = [...sourceVersions.value]

		if (searchQuery.value) {
			const query = searchQuery.value.toLowerCase()
			versions = versions.filter(
				(v) =>
					v.name.toLowerCase().includes(query) ||
					v.version_number.toLowerCase().includes(query) ||
					(incompatibilityWarningMode.value &&
						[...v.loaders, ...v.game_versions].some((value) =>
							value.toLowerCase().includes(query),
						)),
			)
		}

		const beforeFilterCount = versions.length
		if (!incompatibilityWarningMode.value && !isModpack.value && hideIncompatibleState.value) {
			versions = versions.filter(
				(version) =>
					version.id === currentVersionId.value ||
					version.id === selectedVersion.value?.id ||
					version.id === pinnedInitialVersionId.value ||
					isVersionCompatible(version),
			)
		}

		debug('filteredVersions computed', {
			totalVersions: sourceVersions.value.length,
			afterSearchFilter: beforeFilterCount,
			afterCompatibilityFilter: versions.length,
			hiddenByCompatibility: beforeFilterCount - versions.length,
			hideIncompatible: hideIncompatibleState.value,
			filteringCompatibility: !isModpack.value && hideIncompatibleState.value,
		})

		return versions
	})
}
