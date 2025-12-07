import type { Labrinth } from '@modrinth/api-client'

import { inferVersionInfo } from '~/helpers/infer'

// this interface should be in infer.js, but gotta refactor that to ts first
export interface InferredVersionInfo {
	name?: string
	version_number?: string
	version_type?: 'alpha' | 'beta' | 'release'
	loaders?: string[]
	game_versions?: string[]
}

export type DraftVersion = Omit<Labrinth.Versions.v3.CreateVersionRequest, 'file_parts'> & {
	files: File[]
}

const EMPTY_DRAFT_VERSION: DraftVersion = {
	project_id: '',
	version_title: '',
	version_number: '',
	release_channel: 'release',
	loaders: [],
	game_versions: [],
	featured: false,
	status: 'draft',
	version_body: '',
	dependencies: [],
	files: [],
}

const draftVersion = ref<DraftVersion>(EMPTY_DRAFT_VERSION)

const inferredVersionData = ref<InferredVersionInfo>()

const projectType = ref<Labrinth.Projects.v2.ProjectType>()

export function useManageVersion() {
	function newDraftVersion(projectId: string) {
		draftVersion.value = structuredClone(EMPTY_DRAFT_VERSION)
		draftVersion.value.project_id = projectId
	}

	function setPrimaryFile(index: number) {
		const files = draftVersion.value.files
		if (index <= 0 || index >= files.length) return
		;[files[0], files[index]] = [files[index], files[0]]
	}

	function setProjectType(project: Labrinth.Projects.v2.Project) {
		if (project.project_type) {
			projectType.value = project.project_type
		} else {
			projectType.value = project.project_type
		}
	}

	async function setInferredVersionData(file: File, project: Labrinth.Projects.v2.Project) {
		const tags = useGeneratedState()

		const inferred = (await inferVersionInfo(
			file,
			project,
			tags.value.gameVersions,
		)) as InferredVersionInfo

		inferredVersionData.value = inferred

		setProjectType(project)

		return inferred
	}

	return {
		inferredVersionData,
		draftVersion,
		newDraftVersion,
		setPrimaryFile,
		setInferredVersionData,
	}
}
