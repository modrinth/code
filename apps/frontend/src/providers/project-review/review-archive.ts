import type { Labrinth } from '@modrinth/api-client'
import { type ArchiveSource, createContext } from '@modrinth/ui'
import { computed, type Ref, ref, watch } from 'vue'

import {
	archiveNameFrom,
	archivePathFrom,
	archiveSourceFrom,
	primaryFile,
} from '~/helpers/archive-viewer'

export const [injectReviewArchive, provideReviewArchive] =
	createContext<ReturnType<typeof createReviewArchive>>('ProjectReviewArchive')

/**
 * The archive the review shows in its archive panels. Until a reviewer opens another file (from
 * the versions, the details' version picker, or technical review), it's the latest version's
 * primary file.
 */
export function createReviewArchive(
	projectId: Readonly<Ref<string | undefined>>,
	versions: Readonly<Ref<Labrinth.Versions.v3.Version[]>>,
) {
	const selectedVersionId = ref<string>()
	const selectedUrl = ref<string>()
	/** The entry to open within the archive, as the archive viewer's path. */
	const path = ref<string | null>(null)
	/** Bumped whenever a file is opened, so the review can bring its archive panels forward. */
	const openRequests = ref(0)

	watch(projectId, () => {
		selectedVersionId.value = undefined
		selectedUrl.value = undefined
		path.value = null
	})

	const version = computed(
		() =>
			(selectedVersionId.value
				? versions.value.find(({ id }) => id === selectedVersionId.value)
				: undefined) ??
			(selectedUrl.value
				? versions.value.find(({ files }) => files.some(({ url }) => url === selectedUrl.value))
				: undefined) ??
			(selectedUrl.value ? undefined : versions.value[0]),
	)

	const file = computed(() => {
		const current = version.value
		if (!current) return undefined
		return (
			(selectedUrl.value
				? current.files.find(({ url }) => url === selectedUrl.value)
				: undefined) ?? primaryFile(current)
		)
	})

	/** The selected file, or a URL opened that none of the project's versions hold. */
	const source = computed<ArchiveSource | null>(() => {
		if (file.value) return archiveSourceFrom(file.value)
		const url = selectedUrl.value
		return url ? { url, name: archiveNameFrom(url) } : null
	})

	/**
	 * Opens a file in the archive panels, at an entry within it when given (a path within the
	 * archive, or a Slicer entry starting with the archive's name).
	 */
	function open(url: string, entry?: string) {
		selectedVersionId.value = undefined
		selectedUrl.value = url
		path.value = archivePathFrom(entry, archiveNameFrom(url))
		openRequests.value++
	}

	function selectVersion(id: string | undefined) {
		selectedVersionId.value = id
		selectedUrl.value = undefined
		path.value = null
	}

	function selectFile(url: string | undefined) {
		selectedVersionId.value = version.value?.id
		selectedUrl.value = url
		path.value = null
	}

	return {
		version,
		file,
		source,
		path,
		open,
		openRequests,
		selectVersion,
		selectFile,
		/** Follows the entry open in the archive panels, e.g. for links to it. */
		setPath: (value: string) => (path.value = value || null),
	}
}
