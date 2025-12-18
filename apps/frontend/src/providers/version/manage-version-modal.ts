import type { Labrinth } from '@modrinth/api-client'
import {
	createContext,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	type MultiStageModal,
	resolveCtxFn,
	type StageConfigInput,
} from '@modrinth/ui'
import JSZip from 'jszip'
import type { ComputedRef, Ref, ShallowRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import { useGeneratedState } from '~/composables/generated'
import { inferVersionInfo } from '~/helpers/infer'

import { stageConfigs } from './stages'

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

export type VersionStage =
	| 'add-files'
	| 'add-details'
	| 'add-loaders'
	| 'add-mc-versions'
	| 'add-environment'
	| 'add-dependencies'
	| 'add-changelog'
	| 'edit-loaders'
	| 'edit-mc-versions'
	| 'edit-environment'

export interface ManageVersionContextValue {
	// State
	draftVersion: Ref<Labrinth.Versions.v3.DraftVersion>
	filesToAdd: Ref<Labrinth.Versions.v3.DraftVersionFile[]>
	existingFilesToDelete: Ref<Labrinth.Versions.v3.VersionFileHash['sha1'][]>
	inferredVersionData: Ref<InferredVersionInfo | undefined>
	projectType: Ref<Labrinth.Projects.v2.ProjectType | undefined>
	dependencyProjects: Ref<Record<string, Labrinth.Projects.v3.Project>>
	dependencyVersions: Ref<Record<string, Labrinth.Versions.v3.Version>>

	// Stage management
	stageConfigs: StageConfigInput<ManageVersionContextValue>[]
	isSubmitting: Ref<boolean>
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>

	// Computed state
	editingVersion: ComputedRef<boolean>
	noLoadersProject: ComputedRef<boolean>
	noEnvironmentProject: ComputedRef<boolean>

	// Stage helpers
	getNextLabel: (currentIndex?: number | null) => string

	// Version methods
	newDraftVersion: (projectId: string, version?: Labrinth.Versions.v3.DraftVersion | null) => void
	setPrimaryFile: (index: number) => void
	setInferredVersionData: (
		file: File,
		project: Labrinth.Projects.v2.Project,
	) => Promise<InferredVersionInfo>
	setProjectType: (
		project: Labrinth.Projects.v2.Project,
		file?: File | null,
	) => Promise<Labrinth.Projects.v2.ProjectType | undefined>
	getProject: (projectId: string) => Promise<Labrinth.Projects.v3.Project>
	getVersion: (versionId: string) => Promise<Labrinth.Versions.v3.Version>

	// Submission methods
	handleCreateVersion: () => Promise<void>
	handleSaveVersionEdits: () => Promise<void>
}

export const [injectManageVersionContext, provideManageVersionContext] =
	createContext<ManageVersionContextValue>('CreateProjectVersionModal')

export function createManageVersionContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
): ManageVersionContextValue {
	const { labrinth } = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { refreshVersions } = injectProjectPageContext()

	// State
	const draftVersion = ref<Labrinth.Versions.v3.DraftVersion>(structuredClone(EMPTY_DRAFT_VERSION))
	const filesToAdd = ref<Labrinth.Versions.v3.DraftVersionFile[]>([])
	const existingFilesToDelete = ref<Labrinth.Versions.v3.VersionFileHash['sha1'][]>([])
	const inferredVersionData = ref<InferredVersionInfo>()
	const projectType = ref<Labrinth.Projects.v2.ProjectType>()
	const dependencyProjects = ref<Record<string, Labrinth.Projects.v3.Project>>({})
	const dependencyVersions = ref<Record<string, Labrinth.Versions.v3.Version>>({})
	const isSubmitting = ref(false)

	// Computed state
	const editingVersion = computed(() => Boolean(draftVersion.value.version_id))

	// Helper functions for project type detection
	// TODO: move to infer.js
	async function setProjectType(
		project: Labrinth.Projects.v2.Project,
		file: File | null = null,
	): Promise<Labrinth.Projects.v2.ProjectType | undefined> {
		if (project.project_type && project.project_type !== 'project') {
			projectType.value = project.project_type
			return projectType.value
		}

		if (
			(file && file.name.toLowerCase().endsWith('.mrpack')) ||
			(file && file.name.toLowerCase().endsWith('.mrpack-primary'))
		) {
			projectType.value = 'modpack'
			return projectType.value
		}

		if (
			draftVersion.value.loaders?.some((loader) =>
				[
					'fabric',
					'neoforge',
					'forge',
					'quilt',
					'liteloader',
					'rift',
					'ornithe',
					'nilloader',
					'legacy-fabric',
					'bta-babric',
					'babric',
					'modloader',
					'java-agent',
				].includes(loader),
			)
		) {
			projectType.value = 'mod'
			return projectType.value
		}

		try {
			if (file) {
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
			}
		} catch {
			// not a zip
		}

		projectType.value = undefined
		return undefined
	}

	// Version management methods
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

	// Submission handlers
	async function handleCreateVersion() {
		const version = toRaw(draftVersion.value)
		const files = toRaw(filesToAdd.value)
		isSubmitting.value = true

		if (noEnvironmentProject.value) version.environment = undefined

		try {
			await labrinth.versions_v3.createVersion(version, files)
			modal.value?.hide()
			addNotification({
				title: 'Project version created',
				text: 'The version has been successfully added to your project.',
				type: 'success',
			})
			await refreshVersions()
		} catch (err: any) {
			addNotification({
				title: 'An error occurred',
				text: err.data ? err.data.description : err,
				type: 'error',
			})
		}
		isSubmitting.value = false
	}

	async function handleSaveVersionEdits() {
		const version = toRaw(draftVersion.value)
		const files = toRaw(filesToAdd.value)
		const filesToDelete = toRaw(existingFilesToDelete.value)

		isSubmitting.value = true

		if (noEnvironmentProject.value) version.environment = undefined

		try {
			if (!version.version_id) throw new Error('Version ID is required to save edits.')

			await labrinth.versions_v3.modifyVersion(version.version_id, {
				name: version.name || version.version_number,
				version_number: version.version_number,
				changelog: version.changelog,
				version_type: version.version_type,
				dependencies: version.dependencies || [],
				game_versions: version.game_versions,
				loaders: version.loaders,
				environment: version.environment,
				file_types: version.existing_files
					?.filter((file) => file.file_type)
					.map((file) => ({
						algorithm: 'sha1',
						hash: file.hashes.sha1,
						file_type: file.file_type ?? null,
					})),
			})

			if (files.length > 0) {
				await labrinth.versions_v3.addFilesToVersion(version.version_id, files)
			}

			// Delete files that were marked for deletion
			for (const hash of filesToDelete) {
				await useBaseFetch(`version_file/${hash}?version_id=${version.version_id}`, {
					method: 'DELETE',
				})
			}

			modal.value?.hide()
			addNotification({
				title: 'Project version saved',
				text: 'The version has been successfully saved to your project.',
				type: 'success',
			})
			await refreshVersions()
		} catch (err: any) {
			addNotification({
				title: 'An error occurred',
				text: err.data ? err.data.description : err,
				type: 'error',
			})
		}
		isSubmitting.value = false
	}

	// Stage visibility computeds (inlined)
	const noLoadersProject = computed(() => projectType.value === 'resourcepack')
	const noEnvironmentProject = computed(
		() => projectType.value !== 'mod' && projectType.value !== 'modpack',
	)

	// Dynamic next button label
	function getNextLabel(currentIndex: number | null = null) {
		const currentStageIndex = currentIndex ? currentIndex : modal.value?.currentStageIndex || 0

		let nextIndex = currentStageIndex + 1
		while (nextIndex < stageConfigs.length) {
			const skip = stageConfigs[nextIndex]?.skip
			if (!skip || !resolveCtxFn(skip, contextValue)) break
			nextIndex++
		}

		const next = stageConfigs[nextIndex]
		if (!next) return 'Done'

		switch (next.id) {
			case 'add-details':
				return editingVersion.value ? 'Edit details' : 'Add details'
			case 'add-files':
				return editingVersion.value ? 'Edit files' : 'Add files'
			case 'add-loaders':
				return editingVersion.value ? 'Edit loaders' : 'Set loaders'
			case 'add-mc-versions':
				return editingVersion.value ? 'Edit MC versions' : 'Set MC versions'
			case 'add-dependencies':
				return editingVersion.value ? 'Edit dependencies' : 'Set dependencies'
			case 'add-environment':
				return editingVersion.value ? 'Edit environment' : 'Add environment'
			case 'add-changelog':
				return editingVersion.value ? 'Edit changelog' : 'Add changelog'
			default:
				return 'Next'
		}
	}

	const contextValue: ManageVersionContextValue = {
		// State
		draftVersion,
		filesToAdd,
		existingFilesToDelete,
		inferredVersionData,
		projectType,
		dependencyProjects,
		dependencyVersions,

		// Stage management
		stageConfigs,
		isSubmitting,
		modal,

		// Computed
		editingVersion,
		noLoadersProject,
		noEnvironmentProject,

		// Stage helpers
		getNextLabel,

		// Methods
		newDraftVersion,
		setPrimaryFile,
		setInferredVersionData,
		setProjectType,
		getProject,
		getVersion,
		handleCreateVersion,
		handleSaveVersionEdits,
	}

	return contextValue
}
