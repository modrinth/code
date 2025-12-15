import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
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
	environment?: Labrinth.Projects.v3.Environment
}

const EMPTY_DRAFT_VERSION: Labrinth.Versions.v3.DraftVersion = {
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
const detectedEnvironment = computed(() => !!inferredVersionData.value?.environment)

const noLoadersProject = computed(() => projectType.value === 'resourcepack')
const noEnvironmentProject = computed(
	() => projectType.value !== 'mod' && projectType.value !== 'modpack',
)
const noDependenciesProject = computed(() => projectType.value === 'modpack')

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
	const { labrinth } = injectModrinthClient()

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

	const tags = useGeneratedState()

	async function setInferredVersionData(
		file: File,
		project: Labrinth.Projects.v2.Project,
	): Promise<InferredVersionInfo> {
		const inferred = (await inferVersionInfo(
			file,
			project,
			tags.value.gameVersions,
		)) as InferredVersionInfo

		try {
			const versions = await labrinth.versions_v3.getProjectVersions(project.id, {
				loaders: inferred.loaders ?? [],
			})

			if (versions.length > 0) {
				const mostRecentVersion = versions[0]
				const version = await labrinth.versions_v3.getVersion(mostRecentVersion.id)
				inferred.environment = version.environment !== 'unknown' ? version.environment : undefined
			}
		} catch (error) {
			console.error('Error fetching versions for environment inference:', error)
		}

		inferredVersionData.value = inferred
		projectType.value = await setProjectType(project, file)

		return inferred
	}

	const getProject = async (projectId: string) => {
		if (dependencyProjects.value[projectId]) {
			return dependencyProjects.value[projectId]
		}
		const proj = await labrinth.projects_v3.get(projectId)
		dependencyProjects.value[projectId] = proj
		return proj
	}

	const getVersion = async (versionId: string) => {
		if (dependencyVersions.value[versionId]) {
			return dependencyVersions.value[versionId]
		}
		const version = await labrinth.versions_v3.getVersion(versionId)
		dependencyVersions.value[versionId] = version
		return version
	}

	return {
		inferredVersionData,
		draftVersion,
		filesToAdd,
		existingFilesToDelete,
		detectedLoaders,
		detectedVersions,
		detectedEnvironment,
		projectType,
		dependencyProjects,
		dependencyVersions,
		editingVersion,
		noLoadersProject,
		noEnvironmentProject,
		noDependenciesProject,
		newDraftVersion,
		setPrimaryFile,
		setInferredVersionData,
		setProjectType,
		getProject,
		getVersion,
	}
}
