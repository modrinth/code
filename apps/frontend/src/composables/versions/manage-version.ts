import type { Labrinth } from '@modrinth/api-client'

type DraftVersion = Omit<Labrinth.Versions.v3.CreateVersionRequest, 'file_parts'> & {
	files: File[]
}

const EMPTY_DRAFT_VERSION: DraftVersion = {
	project_id: '',
	name: '',
	version_number: '',
	version_type: 'release',
	loaders: [],
	game_versions: [],
	featured: false,
	status: 'draft',
	changelog: '',
	dependencies: [],
	files: [],
}

const draftVersion = ref<DraftVersion>(EMPTY_DRAFT_VERSION)

export function useManageVersion() {
	function newDraftVersion() {
		draftVersion.value = structuredClone(EMPTY_DRAFT_VERSION)
	}

	function setPrimaryFile(index: number) {
		const files = draftVersion.value.files
		if (index <= 0 || index >= files.length) return
		;[files[0], files[index]] = [files[index], files[0]]
	}

	return { draftVersion, newDraftVersion, setPrimaryFile }
}
