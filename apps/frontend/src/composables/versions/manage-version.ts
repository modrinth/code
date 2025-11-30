import type { Labrinth } from '@modrinth/api-client'

type DraftVersion = Omit<Labrinth.Versions.v3.CreateVersionRequest, 'file_parts'> & {
	files: File[]
}

const EMPTY_DRAFT_VERSION: DraftVersion = {
	project_id: '',
	name: '',
	version_number: '',
	version_type: 'alpha',
	loaders: [],
	game_versions: [],
	featured: false,
	status: 'draft',
	changelog: null,
	dependencies: [],
	files: [],
}

const draftVersion = ref<DraftVersion>(EMPTY_DRAFT_VERSION)

export const useManageVersion = () => {
	function newDraftVersion() {
		draftVersion.value = structuredClone(EMPTY_DRAFT_VERSION)
	}

	return { draftVersion, newDraftVersion }
}
