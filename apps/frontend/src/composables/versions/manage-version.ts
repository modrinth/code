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

const EMPTY_DRAFT_VERSION: Labrinth.Versions.v3.DraftVersion = {
	project_id: '',
	version_title: '',
	version_number: '',
	version_type: 'release',
	loaders: [],
	game_versions: [],
	featured: false,
	status: 'draft',
	changelog: '',
	dependencies: [],
}

const draftVersion = ref<Labrinth.Versions.v3.DraftVersion>(EMPTY_DRAFT_VERSION)
const filesToAdd = ref<Labrinth.Versions.v3.DraftVersionFile[]>([])
const existingFilesToDelete = ref<Labrinth.Versions.v3.VersionFileHash['sha1'][]>([])

const inferredVersionData = ref<InferredVersionInfo>()

const projectType = ref<Labrinth.Projects.v2.ProjectType>()

const editingVersion = computed(() => Boolean(draftVersion.value.version_id))

const dependencyProjects = ref<Record<string, Labrinth.Projects.v3.Project>>({})
const dependencyVersions = ref<Record<string, Labrinth.Versions.v3.Version>>({})

const detectedLoaders = computed(() => (inferredVersionData.value?.loaders?.length || 0) > 0)
const detectedVersions = computed(() => (inferredVersionData.value?.game_versions?.length || 0) > 0)

async function setProjectType(
	project: Labrinth.Projects.v2.Project,
	file: File | null = null,
): Promise<Labrinth.Projects.v2.ProjectType | undefined> {
	if (project.project_type && project.project_type !== 'project') {
		projectType.value = project.project_type
		return projectType.value
	}

	// if file extension is .mrpack, it's a modpack
	if (
		(file && file.name.toLowerCase().endsWith('.mrpack')) ||
		(file && file.name.toLowerCase().endsWith('.mrpack-primary'))
	) {
		projectType.value = 'modpack'
		return projectType.value
	}

	if (
		draftVersion.value.loaders?.some((loader) =>
			['fabric', 'forge', 'quilt', 'neoforge'].includes(loader),
		)
	) {
		projectType.value = 'mod'
		return projectType.value
	}

	// if inside file zip, has pack.mcmeta and assets directory both in root, its a resource pack
	try {
		if (!file) return undefined

		const jszip = await JSZip.loadAsync(file)

		const hasMcmeta = Object.keys(jszip.files).some(
			(f) => f.toLowerCase() === 'pack.mcmeta' || f.toLowerCase().endsWith('/pack.mcmeta'),
		)
		const hasAssetsDir = Object.keys(jszip.files).some(
			(f) => f.toLowerCase() === 'assets/' || f.toLowerCase().startsWith('assets/'),
		)

		if (hasMcmeta && hasAssetsDir) {
			projectType.value = 'resourcepack'
			return projectType.value
		}
	} catch {
		// not a zip
	}

	projectType.value = undefined
	return undefined
}

export function useManageVersion() {
	function newDraftVersion(
		projectId: string,
		version: Labrinth.Versions.v3.DraftVersion | null = null,
	) {
		draftVersion.value = structuredClone(version ?? EMPTY_DRAFT_VERSION)
		draftVersion.value.project_id = projectId
		filesToAdd.value = []
		existingFilesToDelete.value = []
		inferredVersionData.value = undefined
		projectType.value = undefined
	}

	function setPrimaryFile(index: number) {
		const files = filesToAdd.value
		if (index <= 0 || index >= files.length) return
		files[0].fileType = 'unknown'
		files[index].fileType = 'unknown'
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

		projectType.value = await setProjectType(project, file)

		return inferred
	}

	return {
		inferredVersionData,
		draftVersion,
		filesToAdd,
		existingFilesToDelete,
		detectedLoaders,
		detectedVersions,
		projectType,
		dependencyProjects,
		dependencyVersions,
		editingVersion,
		newDraftVersion,
		setPrimaryFile,
		setInferredVersionData,
		setProjectType,
	}
}
