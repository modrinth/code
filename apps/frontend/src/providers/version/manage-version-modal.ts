import type { Labrinth, UploadProgress } from '@modrinth/api-client'
import { SaveIcon, SpinnerIcon } from '@modrinth/assets'
import {
	createContext,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	type MultiStageModal,
	resolveCtxFn,
	type StageButtonConfig,
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
	| 'from-details-loaders'
	| 'from-details-mc-versions'
	| 'from-details-environment'

export type SuggestedDependency = Labrinth.Versions.v3.Dependency & {
	name?: string
	icon?: string
	versionName?: string
}

export interface PrimaryFile {
	name: string
	fileType?: string
	existing?: boolean
}

export interface ManageVersionContextValue {
	// State
	draftVersion: Ref<Labrinth.Versions.v3.DraftVersion>
	filesToAdd: Ref<Labrinth.Versions.v3.DraftVersionFile[]>
	existingFilesToDelete: Ref<Labrinth.Versions.v3.VersionFileHash['sha1'][]>
	inferredVersionData: Ref<InferredVersionInfo | undefined>
	projectType: Ref<Labrinth.Projects.v2.ProjectType | undefined>
	dependencyProjects: Ref<Record<string, Labrinth.Projects.v3.Project>>
	dependencyVersions: Ref<Record<string, Labrinth.Versions.v3.Version>>
	projectsFetchLoading: Ref<boolean>
	handlingNewFiles: Ref<boolean>
	suggestedDependencies: Ref<SuggestedDependency[] | null>
	visibleSuggestedDependencies: ComputedRef<SuggestedDependency[]>
	primaryFile: ComputedRef<PrimaryFile | null>

	// Stage management
	stageConfigs: StageConfigInput<ManageVersionContextValue>[]
	isSubmitting: Ref<boolean>
	isUploading: Ref<boolean>
	uploadProgress: Ref<UploadProgress>
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>

	// Computed state
	editingVersion: ComputedRef<boolean>
	noEnvironmentProject: ComputedRef<boolean>
	noDependenciesProject: ComputedRef<boolean>

	// Stage helpers
	getNextLabel: (currentIndex?: number | null) => string
	saveButtonConfig: () => StageButtonConfig

	// Version methods
	newDraftVersion: (projectId: string, version?: Labrinth.Versions.v3.DraftVersion | null) => void
	handleNewFiles: (newFiles: File[]) => Promise<void>
	swapPrimaryFile: (index: number) => void
	replacePrimaryFile: (file: File) => Promise<void>
	getProject: (projectId: string) => Promise<Labrinth.Projects.v3.Project>
	getVersion: (versionId: string) => Promise<Labrinth.Versions.v3.Version>

	// Submission methods
	handleCreateVersion: () => Promise<void>
	handleSaveVersionEdits: () => Promise<void>
}

const PROJECT_TYPE_LOADERS: Record<string, readonly string[]> = {
	mod: [
		'fabric',
		'neoforge',
		'forge',
		'quilt',
		'liteloader',
		'rift',
		'ornithe',
		'nilloader',
		'risugami',
		'legacy-fabric',
		'bta-babric',
		'babric',
		'modloader',
		'java-agent',
	],
	shader: ['optifine', 'iris', 'canvas', 'vanilla'],
	plugin: [
		'paper',
		'purpur',
		'spigot',
		'bukkit',
		'sponge',
		'folia',
		'bungeecord',
		'velocity',
		'waterfall',
		'geyser',
	],
	datapack: ['datapack'],
	resourcepack: ['minecraft'],
	modpack: ['mrpack'],
} as const

export const fileTypeLabels: Record<Labrinth.Versions.v3.FileType | 'primary', string> = {
	primary: 'Primary',
	unknown: 'Other',
	'required-resource-pack': 'Required RP',
	'optional-resource-pack': 'Optional RP',
	'sources-jar': 'Sources JAR',
	'dev-jar': 'Dev JAR',
	'javadoc-jar': 'Javadoc JAR',
	signature: 'Signature',
}

export const [injectManageVersionContext, provideManageVersionContext] =
	createContext<ManageVersionContextValue>('CreateProjectVersionModal')

export function createManageVersionContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
	onSave?: () => void,
): ManageVersionContextValue {
	const { labrinth } = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { refreshVersions, projectV2 } = injectProjectPageContext()

	// State
	const draftVersion = ref<Labrinth.Versions.v3.DraftVersion>(structuredClone(EMPTY_DRAFT_VERSION))
	const filesToAdd = ref<Labrinth.Versions.v3.DraftVersionFile[]>([])
	const existingFilesToDelete = ref<Labrinth.Versions.v3.VersionFileHash['sha1'][]>([])
	const handlingNewFiles = ref(false)
	const inferredVersionData = ref<InferredVersionInfo>()
	const dependencyProjects = ref<Record<string, Labrinth.Projects.v3.Project>>({})
	const dependencyVersions = ref<Record<string, Labrinth.Versions.v3.Version>>({})
	const projectsFetchLoading = ref(false)
	const suggestedDependencies = ref<SuggestedDependency[] | null>(null)

	const isSubmitting = ref(false)
	const isUploading = ref(false)
	const uploadProgress = ref<UploadProgress>({ loaded: 0, total: 0, progress: 0 })

	const projectType = computed<Labrinth.Projects.v2.ProjectType>(() => {
		const primaryFile = filesToAdd.value[0]?.file
		if (
			(primaryFile && primaryFile.name.toLowerCase().endsWith('.mrpack')) ||
			(primaryFile && primaryFile.name.toLowerCase().endsWith('.mrpack-primary'))
		) {
			return 'modpack'
		}

		const loaders = draftVersion.value.loaders || []

		if (loaders.some((loader) => PROJECT_TYPE_LOADERS.modpack.includes(loader))) {
			return 'modpack'
		}

		if (loaders.some((loader) => PROJECT_TYPE_LOADERS.datapack.includes(loader))) {
			return 'datapack'
		}
		if (loaders.length === 1 && loaders[0] === 'minecraft') {
			return 'resourcepack'
		}
		if (loaders.some((loader) => PROJECT_TYPE_LOADERS.shader.includes(loader))) {
			return 'shader'
		}
		if (loaders.some((loader) => PROJECT_TYPE_LOADERS.plugin.includes(loader))) {
			return 'plugin'
		}
		if (loaders.some((loader) => PROJECT_TYPE_LOADERS.mod.includes(loader))) {
			return 'mod'
		}

		return 'project'
	})

	// Computed state
	const editingVersion = computed(() => Boolean(draftVersion.value.version_id))

	const visibleSuggestedDependencies = computed<SuggestedDependency[]>(() => {
		const existingDeps = draftVersion.value.dependencies ?? []
		const seenKeys = new Set<string>()

		const isDuplicateSuggestion = (dep: SuggestedDependency) => {
			const key = `${dep.project_id ?? ''}:${dep.version_id ?? ''}`
			if (seenKeys.has(key)) return true
			seenKeys.add(key)
			return false
		}

		const isAlreadyAdded = (dep: SuggestedDependency) =>
			existingDeps.some((existing) => {
				if (existing.project_id !== dep.project_id) return false
				if (!existing.version_id && !dep.version_id) return true
				return existing.version_id === dep.version_id
			})

		return (suggestedDependencies.value ?? [])
			.filter((dep) => !isDuplicateSuggestion(dep))
			.filter((dep) => !isAlreadyAdded(dep))
			.sort((a, b) => (a.name ?? '').localeCompare(b.name ?? ''))
	})

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
	}

	async function handleNewFiles(newFiles: File[]) {
		handlingNewFiles.value = true
		// detect primary file if no primary file is set
		const primaryFileIndex = primaryFile.value ? null : detectPrimaryFileIndex(newFiles)

		newFiles.forEach((file) => filesToAdd.value.push({ file }))

		if (primaryFileIndex !== null) {
			if (primaryFileIndex) swapPrimaryFile(primaryFileIndex)
		}

		if (
			filesToAdd.value.length === 1 &&
			!editingVersion.value &&
			modal.value?.currentStageIndex === 0
		) {
			if (await rejectOnRedundantWrappedZip(filesToAdd.value[0].file)) return

			await addDetectedData()
			modal.value?.nextStage()
		}

		handlingNewFiles.value = false
	}

	async function replacePrimaryFile(file: File) {
		if (file && !editingVersion.value) {
			filesToAdd.value[0] = { file }
		}
		if (await rejectOnRedundantWrappedZip(file)) return
		await addDetectedData()
	}

	async function swapPrimaryFile(index: number) {
		const files = filesToAdd.value
		if (index <= 0 || index >= files.length) return
		files[0].fileType = 'unknown'
		files[index].fileType = 'unknown'
		;[files[0], files[index]] = [files[index], files[0]]

		if (await rejectOnRedundantWrappedZip(files[0].file)) return
		await addDetectedData()
	}

	const tags = useGeneratedState()

	const hasFile = (entries: string[], name: string) =>
		entries.some((f) => f === name || f.endsWith(`/${name}`))

	const hasDir = (entries: string[], dir: string) => entries.some((f) => f.startsWith(`${dir}/`))

	async function checkIsResourcePack(file: File): Promise<boolean> {
		try {
			const name = file.name.toLowerCase()
			if (!name.endsWith('.zip')) return false

			const zip = await JSZip.loadAsync(file)
			const entries = Object.keys(zip.files).map((f) => f.toLowerCase())

			return hasFile(entries, 'pack.mcmeta') && hasDir(entries, 'assets')
		} catch {
			return false
		}
	}

	async function checkIsDataPack(file: File): Promise<boolean> {
		try {
			const name = file.name.toLowerCase()
			if (!name.endsWith('.zip')) return false

			const zip = await JSZip.loadAsync(file)
			const entries = Object.keys(zip.files).map((f) => f.toLowerCase())

			return hasFile(entries, 'pack.mcmeta') && hasDir(entries, 'data')
		} catch {
			return false
		}
	}

	async function checkRedundantWrappedZip(file: File): Promise<boolean> {
		const fileName = file.name.toLowerCase()
		if (!fileName.endsWith('.zip')) return false

		const zip = await JSZip.loadAsync(file)
		const entries = Object.keys(zip.files).map((e) => e.toLowerCase())
		const filtered = entries.filter((e) => !e.startsWith('__macosx/') && !e.endsWith('.ds_store'))

		const hasRootEntries = filtered.some((e) => !e.includes('/'))
		if (hasRootEntries) return false

		const topLevelFolders = new Set(filtered.map((e) => e.split('/')[0]).filter(Boolean))
		if (topLevelFolders.size !== 1) return false

		const [folderName] = [...topLevelFolders]

		// Check if the inner folder contents indicate a datapack or resource pack
		const innerEntries = filtered.map((e) => e.substring(folderName.length + 1))
		const hasPackMcmeta = hasFile(innerEntries, 'pack.mcmeta')
		const hasAssets = hasDir(innerEntries, 'assets')
		const hasData = hasDir(innerEntries, 'data')

		return hasPackMcmeta && (hasAssets || hasData)
	}

	async function rejectOnRedundantWrappedZip(file: File): Promise<boolean> {
		if (await checkRedundantWrappedZip(file)) {
			newDraftVersion(projectV2.value.id)
			modal.value?.setStage('add-files')
			addNotification({
				title: 'Invalid ZIP structure',
				text: `The uploaded ZIP file "${file.name}" contains a redundant top-level folder. Please re-zip the contents directly without the extra folder layer.`,
				type: 'error',
			})
			return true
		}
		return false
	}

	async function inferEnvironmentFromVersions(
		projectId: string,
		loaders: string[],
	): Promise<Labrinth.Projects.v3.Environment | undefined> {
		try {
			const versions = await labrinth.versions_v3.getProjectVersions(projectId, {
				loaders,
			})

			if (versions.length > 0) {
				const mostRecentVersion = versions[0]
				const version = await labrinth.versions_v3.getVersion(mostRecentVersion.id)
				return version.environment !== 'unknown' ? version.environment : undefined
			}
		} catch (error) {
			console.error('Error fetching versions for environment inference:', error)
		}
		return undefined
	}

	async function setInferredVersionData(
		file: File,
		project: Labrinth.Projects.v2.Project,
	): Promise<InferredVersionInfo> {
		const inferred = (await inferVersionInfo(
			file,
			project,
			tags.value.gameVersions,
		)) as InferredVersionInfo

		inferred.environment = await inferEnvironmentFromVersions(project.id, inferred.loaders ?? [])

		const noLoaders = !inferred.loaders?.length

		if (noLoaders && (await checkIsResourcePack(file))) {
			inferred.loaders = ['minecraft']
		}

		if (noLoaders && (await checkIsDataPack(file))) {
			inferred.loaders = ['datapack']
		}

		if (noLoaders && projectType.value === 'modpack') {
			inferred.loaders = ['minecraft']
		}

		inferredVersionData.value = inferred

		return inferred
	}

	// Stage visibility computeds (inlined)
	const noEnvironmentProject = computed(
		() => projectType.value !== 'mod' && projectType.value !== 'modpack',
	)
	const noDependenciesProject = computed(() => projectType.value === 'modpack')

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

	// Primary file computed
	const primaryFile = computed<PrimaryFile | null>(() => {
		const existingPrimaryFile = draftVersion.value.existing_files?.[0]
		if (existingPrimaryFile) {
			return {
				name: existingPrimaryFile.filename,
				fileType: existingPrimaryFile.file_type,
				existing: true,
			}
		}

		const addedPrimaryFile = filesToAdd.value[0]
		if (addedPrimaryFile) {
			return {
				name: addedPrimaryFile.file.name,
				fileType: addedPrimaryFile.fileType,
				existing: false,
			}
		}

		return null
	})

	// File handling helpers
	function detectPrimaryFileIndex(files: File[]): number {
		const extensionPriority = ['.jar', '.zip', '.litemod', '.mrpack', '.mrpack-primary']

		for (const ext of extensionPriority) {
			const matches = files.filter((file) => file.name.toLowerCase().endsWith(ext))
			if (matches.length > 0) {
				const shortest = matches.reduce((a, b) => (a.name.length < b.name.length ? a : b))
				return files.indexOf(shortest)
			}
		}

		return 0
	}

	const addDetectedData = async () => {
		if (editingVersion.value) return

		const primaryFileData = filesToAdd.value[0]?.file
		if (!primaryFileData) return

		try {
			const inferredData = await setInferredVersionData(primaryFileData, projectV2.value)
			const mappedInferredData: Partial<Labrinth.Versions.v3.DraftVersion> = {
				...inferredData,
				name: inferredData.name || '',
			}

			draftVersion.value = {
				...draftVersion.value,
				...mappedInferredData,
			}
		} catch (err) {
			console.error('Error parsing version file data', err)
		}
	}

	// Watch draft version dependencies to fetch project/version data
	watch(
		draftVersion,
		async (version) => {
			if (noDependenciesProject.value) return
			const deps = version.dependencies || []

			for (const dep of deps) {
				try {
					if (dep?.project_id) await getProject(dep.project_id)
					if (dep?.version_id) await getVersion(dep.version_id)
				} catch (error: any) {
					addNotification({
						title: 'Could not fetch dependency data',
						text: error.data ? error.data.description : error,
						type: 'error',
					})
				}
			}
			projectsFetchLoading.value = false
		},
		{ immediate: true, deep: true },
	)

	// Watch loaders to infer environment if not set
	watch(
		() => draftVersion.value.loaders,
		async (loaders) => {
			if (noEnvironmentProject.value) return
			if (draftVersion.value.environment) return
			if (!loaders?.length) return

			const projectId = draftVersion.value.project_id
			if (!projectId) return

			const environment = await inferEnvironmentFromVersions(projectId, loaders)
			if (environment && !draftVersion.value.environment) {
				draftVersion.value.environment = environment
				inferredVersionData.value = { ...inferredVersionData.value, environment }
			}
		},
	)

	// Watch loaders to fetch suggested dependencies
	// Gets the most recent version that matches loaders and suggests its dependencies
	watch(
		() => draftVersion.value.loaders,
		async (loaders) => {
			if (noDependenciesProject.value) return
			try {
				const projectId = draftVersion.value.project_id
				if (!projectId) return

				try {
					let versions = await labrinth.versions_v3.getProjectVersions(projectId, {
						loaders,
					})
					if (!versions || versions.length === 0) {
						versions = await labrinth.versions_v3.getProjectVersions(projectId)
					}

					// Get the most recent matching version and extract its dependencies
					if (versions.length > 0) {
						suggestedDependencies.value = []
						const mostRecentVersion = versions[0]
						for (const dep of mostRecentVersion.dependencies) {
							suggestedDependencies.value.push({
								project_id: dep.project_id,
								version_id: dep.version_id,
								dependency_type: dep.dependency_type,
								file_name: dep.file_name,
							})
						}
					} else {
						suggestedDependencies.value = null
					}
				} catch (error: any) {
					console.error(`Failed to get versions for project ${projectId}:`, error)
				}

				for (const dep of suggestedDependencies.value ?? []) {
					try {
						if (dep.project_id) {
							const proj = await getProject(dep.project_id)
							dep.name = proj.name
							dep.icon = proj.icon_url
						}

						if (dep.version_id) {
							const version = await getVersion(dep.version_id)
							dep.versionName = version.name
						}
					} catch (error: any) {
						console.error(`Failed to fetch project/version data for dependency:`, error)
					}
				}
			} catch (error: any) {
				addNotification({
					title: 'Could not fetch suggested dependencies',
					text: error.data ? error.data.description : error,
					type: 'error',
				})
			}
		},
		{ immediate: true },
	)

	// Submission handlers
	async function handleCreateVersion() {
		const version = toRaw(draftVersion.value)
		const files = toRaw(filesToAdd.value)
		isSubmitting.value = true
		isUploading.value = true

		// Reset progress and navigate to uploading stage
		uploadProgress.value = { loaded: 0, total: 0, progress: 0 }

		if (noEnvironmentProject.value) version.environment = undefined

		try {
			const uploadHandle = labrinth.versions_v3.createVersion(
				version,
				files,
				projectType.value ?? null,
			)

			// Subscribe to progress updates
			uploadHandle.onProgress((progress) => {
				uploadProgress.value = progress
			})

			// Wait for upload to complete
			await uploadHandle.promise

			isUploading.value = false
			await nextTick()
			modal.value?.hide()

			addNotification({
				title: 'Project version created',
				text: 'The version has been successfully added to your project.',
				type: 'success',
			})
			await refreshVersions()
			onSave?.()
		} catch (err: any) {
			addNotification({
				title: 'Could not create project version',
				text: err.data ? err.data.description : err,
				type: 'error',
			})
		}
		isUploading.value = false
		isSubmitting.value = false
	}

	async function handleSaveVersionEdits() {
		const version = toRaw(draftVersion.value)
		const files = toRaw(filesToAdd.value)
		const filesToDelete = toRaw(existingFilesToDelete.value)

		isSubmitting.value = true

		// Reset progress if we have files to upload
		if (files.length > 0) {
			isUploading.value = true
			uploadProgress.value = { loaded: 0, total: 0, progress: 0 }
		}

		if (noEnvironmentProject.value) version.environment = undefined

		try {
			if (!version.version_id) throw new Error('Version ID is required to save edits.')

			const data: Labrinth.Versions.v3.ModifyVersionRequest = {
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
			}

			await labrinth.versions_v3.modifyVersion(version.version_id, data)

			if (files.length > 0) {
				const uploadHandle = labrinth.versions_v3.addFilesToVersion(version.version_id, files)

				uploadHandle.onProgress((progress) => {
					uploadProgress.value = progress
				})

				await uploadHandle.promise
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
			onSave?.()
		} catch (err: any) {
			addNotification({
				title: 'An error occurred',
				text: err.data ? err.data.description : err,
				type: 'error',
			})
		}
		isUploading.value = false
		isSubmitting.value = false
	}

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
				return editingVersion.value ? 'Edit game versions' : 'Set game versions'
			case 'add-dependencies':
				return editingVersion.value ? 'Edit dependencies' : 'Set dependencies'
			case 'add-environment':
				return editingVersion.value ? 'Edit environment' : 'Add environment'
			case 'add-changelog':
				return editingVersion.value ? 'Edit changelog' : 'Add changelog'
			case 'metadata':
				return 'Edit metadata'
			default:
				return 'Next'
		}
	}

	const saveButtonConfig = (): StageButtonConfig => ({
		label: 'Save changes',
		icon: isSubmitting.value ? SpinnerIcon : SaveIcon,
		iconPosition: 'before',
		iconClass: isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled: isSubmitting.value,
		onClick: () => handleSaveVersionEdits(),
	})

	const contextValue: ManageVersionContextValue = {
		// State
		draftVersion,
		filesToAdd,
		existingFilesToDelete,
		inferredVersionData,
		projectType,
		dependencyProjects,
		dependencyVersions,
		handlingNewFiles,
		projectsFetchLoading,
		suggestedDependencies,
		visibleSuggestedDependencies,
		primaryFile,

		// Stage management
		stageConfigs,
		isSubmitting,
		isUploading,
		uploadProgress,
		modal,

		// Computed
		editingVersion,
		noEnvironmentProject,
		noDependenciesProject,

		// Stage helpers
		getNextLabel,
		saveButtonConfig,

		// Methods
		newDraftVersion,
		swapPrimaryFile,
		replacePrimaryFile,
		getProject,
		getVersion,
		handleNewFiles,
		handleCreateVersion,
		handleSaveVersionEdits,
	}

	return contextValue
}
