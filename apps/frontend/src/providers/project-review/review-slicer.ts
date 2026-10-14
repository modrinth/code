import type { Labrinth } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import { computed, type Ref, shallowRef, watch } from 'vue'

interface SlicerFile {
	url: string
	name: string
}

export interface SlicerTarget {
	key: string
	files: SlicerFile[]
}

type VersionFile = Labrinth.Versions.v3.Version['files'][number]

export const [injectReviewSlicer, provideReviewSlicer] =
	createContext<ReturnType<typeof createReviewSlicer>>('ProjectReviewSlicer')

function toSlicerFile(file: VersionFile): SlicerFile {
	return { url: file.url, name: file.filename }
}

/**
 * Tracks the files the slicer tab should display for the current project. Until a reviewer picks
 * files explicitly, the latest version's primary file is used.
 */
export function createReviewSlicer(
	projectId: Readonly<Ref<string | undefined>>,
	versions: Readonly<Ref<Labrinth.Versions.v3.Version[]>>,
) {
	const selected = shallowRef<SlicerTarget>()
	let nextId = 0

	watch(projectId, () => {
		selected.value = undefined
	})

	const target = computed<SlicerTarget | undefined>(() => {
		if (selected.value) return selected.value
		const latest = versions.value[0]
		const file = latest?.files.find(({ primary }) => primary) ?? latest?.files[0]
		return file ? { key: `default:${file.url}`, files: [toSlicerFile(file)] } : undefined
	})

	function open(files: VersionFile[]) {
		if (!files.length) return
		selected.value = { key: `selected:${++nextId}`, files: files.map(toSlicerFile) }
	}

	return { target, open }
}
