import type { Labrinth } from '@modrinth/api-client'
import JSZip from 'jszip'

import { inferVersionInfo } from '~/helpers/infer'

// this interface should be in infer.js, but gotta refactor that to ts first
export interface InferredVersionInfo {
	name?: string
	version_number?: string
	version_type?: 'alpha' | 'beta' | 'release'
	loaders?: string[]
	game_versions?: string[]
	project_type?: Labrinth.Projects.v2.ProjectType
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

const dependencyProjects = ref<Record<string, Labrinth.Projects.v3.Project>>({})
const dependencyVersions = ref<Record<string, Labrinth.Versions.v3.Version>>({})

const detectedLoaders = computed(() => (inferredVersionData.value?.loaders?.length || 0) > 0)
const detectedVersions = computed(() => (inferredVersionData.value?.game_versions?.length || 0) > 0)

async function getProjectType(
	file: File,
	project: Labrinth.Projects.v2.Project,
): Promise<Labrinth.Projects.v2.ProjectType | undefined> {
	if (project.project_type && project.project_type !== 'project') return project.project_type

	// if file extension is .mrpack, it's a modpack
	if (file.name.toLowerCase().endsWith('.mrpack')) {
		return 'modpack'
	}

	// if inside file zip, has pack.mcmeta and assets directory both in root, its a resource pack
	try {
		const jszip = await JSZip.loadAsync(file)

		const hasMcmeta = Object.keys(jszip.files).some(
			(f) => f.toLowerCase() === 'pack.mcmeta' || f.toLowerCase().endsWith('/pack.mcmeta'),
		)
		const hasAssetsDir = Object.keys(jszip.files).some(
			(f) => f.toLowerCase() === 'assets/' || f.toLowerCase().startsWith('assets/'),
		)

		if (hasMcmeta && hasAssetsDir) return 'resourcepack'
	} catch {
		// not a zip
	}
}

export function useManageVersion() {
	function newDraftVersion(projectId: string) {
		draftVersion.value = structuredClone(EMPTY_DRAFT_VERSION)
		draftVersion.value.project_id = projectId
		inferredVersionData.value = undefined
		projectType.value = undefined
	}

	function setPrimaryFile(index: number) {
		const files = draftVersion.value.files
		if (index <= 0 || index >= files.length) return
		;[files[0], files[index]] = [files[index], files[0]]
	}

	async function setInferredVersionData(file: File, project: Labrinth.Projects.v2.Project) {
		const tags = useGeneratedState()

		const inferred = (await inferVersionInfo(
			file,
			project,
			tags.value.gameVersions,
		)) as InferredVersionInfo

		inferredVersionData.value = inferred

		projectType.value = await getProjectType(file, project)

		return inferred
	}

	return {
		inferredVersionData,
		draftVersion,
		detectedLoaders,
		detectedVersions,
		projectType,
		dependencyProjects,
		dependencyVersions,
		newDraftVersion,
		setPrimaryFile,
		setInferredVersionData,
	}
}
