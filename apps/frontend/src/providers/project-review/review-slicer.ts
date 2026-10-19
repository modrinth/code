import type { Labrinth } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import { computed, type Ref, ref, shallowRef, watch } from 'vue'

interface SlicerFile {
	url: string
	name: string
}

export interface SlicerTarget {
	key: string
	files: SlicerFile[]
	/** Workspace entry to open once the files are loaded, e.g. `mod.jar/com/example/Mod.class`. */
	entry?: string
}

export const [injectReviewSlicer, provideReviewSlicer] =
	createContext<ReturnType<typeof createReviewSlicer>>('ProjectReviewSlicer')

/** Names a file the same way slicer does when loading it from its `url` parameter. */
export function slicerFileName(url: string) {
	const name = new URL(url).pathname.split('/').pop()
	return name?.includes('.') ? name : 'input.jar'
}

function createTarget(urls: string[], entry?: string): SlicerTarget {
	return {
		key: JSON.stringify([urls, entry ?? null]),
		files: urls.map((url) => ({ url, name: slicerFileName(url) })),
		entry,
	}
}

/**
 * Tracks the files the slicer tab should display for the current project. Until a reviewer opens
 * files explicitly, the latest version's primary file is used.
 */
export function createReviewSlicer(
	projectId: Readonly<Ref<string | undefined>>,
	versions: Readonly<Ref<Labrinth.Versions.v3.Version[]>>,
) {
	const selected = shallowRef<SlicerTarget>()
	const openRequests = ref(0)
	const loadingFiles = ref(false)

	watch(projectId, () => {
		selected.value = undefined
		loadingFiles.value = false
	})

	const target = computed<SlicerTarget | undefined>(() => {
		if (selected.value) return selected.value
		const latest = versions.value[0]
		const file = latest?.files.find(({ primary }) => primary) ?? latest?.files[0]
		return file ? createTarget([file.url]) : undefined
	})

	/**
	 * Loads the given file URLs into the slicer tab and brings it to the front. Reopening the same
	 * files with a different entry only opens that entry, without reloading the workspace.
	 */
	function open(urls: string[], entry?: string) {
		if (!urls.length) return
		const next = createTarget(urls, entry)
		if (next.key !== target.value?.key) loadingFiles.value = true
		selected.value = next
		openRequests.value++
	}

	return { target, open, openRequests, loadingFiles }
}
