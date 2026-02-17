import type { Labrinth } from '@modrinth/api-client'
import { ArrowLeftRightIcon, LeftArrowIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	createContext,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	type MultiStageModal,
	type StageConfigInput,
} from '@modrinth/ui'
import JSZip from 'jszip'
import type { Ref, ShallowRef } from 'vue'
import { markRaw, toRaw } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import SelectCompatibilityType from './stages/SelectCompatibilityType.vue'
import SelectPublishedModpack from './stages/SelectPublishedModpack.vue'
import SelectVanillaVersions from './stages/SelectVanillaVersions.vue'
import UploadCustomModpack from './stages/UploadCustomModpack.vue'

export type CompatibilityType = 'vanilla' | 'published-modpack' | 'custom-modpack'

export interface ServerCompatibilityContextValue {
	// Stage management
	stageConfigs: StageConfigInput<ServerCompatibilityContextValue>[]
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	isSubmitting: Ref<boolean>

	// State
	compatibilityType: Ref<CompatibilityType | null>
	selectedProjectId: Ref<string>
	selectedVersionId: Ref<string>
	supportedGameVersions: Ref<string[]>
	recommendedGameVersion: Ref<string | null>
	customModpackFile: Ref<File | null>
	hasLicensePermission: Ref<boolean>
	isEditingExistingCompatibility: Ref<boolean>
	isSwitchingCompatibilityType: Ref<boolean>

	// Actions
	resetContext: () => void
	handleSave: () => Promise<void>
}

export const [injectServerCompatibilityContext, provideServerCompatibilityContext] =
	createContext<ServerCompatibilityContextValue>('ServerCompatibilityModal')

export function createServerCompatibilityContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
): ServerCompatibilityContextValue {
	const { projectV3, patchProjectV3 } = injectProjectPageContext()
	const { labrinth } = injectModrinthClient()
	const { addNotification } = injectNotificationManager()

	const isSubmitting = ref(false)
	const compatibilityType = ref<CompatibilityType | null>(null)
	const selectedProjectId = ref('')
	const selectedVersionId = ref('')
	const supportedGameVersions = ref<string[]>([])
	const recommendedGameVersion = ref<string | null>(null)
	const customModpackFile = ref<File | null>(null)
	const hasLicensePermission = ref(false)
	const isEditingExistingCompatibility = ref(false)
	const isSwitchingCompatibilityType = ref(false)

	async function uploadCustomModpackFile(file: File): Promise<Labrinth.Versions.v3.Version> {
		const rawFile = toRaw(file)

		// Default to filename if we can't parse the mrpack
		let versionName = rawFile.name.replace(/\.(zip|mrpack)$/i, '')
		let versionNumber = versionName
		const loaders: string[] = []
		let gameVersions: string[] = []

		try {
			const zip = await JSZip.loadAsync(rawFile)
			const indexFile = zip.file('modrinth.index.json')

			if (indexFile) {
				const indexContent = await indexFile.async('text')
				const metadata = JSON.parse(indexContent) as {
					name?: string
					versionId?: string
					dependencies?: Record<string, string>
				}
				if (metadata.name) {
					versionName = metadata.name
				}
				if (metadata.versionId) {
					versionNumber = metadata.versionId
				}
				if (metadata.dependencies) {
					if ('forge' in metadata.dependencies) loaders.push('forge')
					if ('neoforge' in metadata.dependencies) loaders.push('neoforge')
					if ('fabric-loader' in metadata.dependencies) loaders.push('fabric')
					if ('quilt-loader' in metadata.dependencies) loaders.push('quilt')
					if (metadata.dependencies.minecraft) {
						gameVersions = [metadata.dependencies.minecraft]
					}
				}
			}
		} catch {
			console.warn('Could not parse modrinth.index.json from mrpack')
		}

		const draftVersion: Labrinth.Versions.v3.DraftVersion = {
			project_id: projectV3.value.id,
			name: versionName,
			version_number: versionNumber,
			version_type: 'release',
			loaders,
			game_versions: gameVersions,
			featured: false,
			status: 'listed',
			changelog: '',
			dependencies: [],
			environment: 'client_and_server',
		}

		const files: Labrinth.Versions.v3.DraftVersionFile[] = [{ file: rawFile, fileType: undefined }]

		const uploadHandle = labrinth.versions_v3.createVersion(draftVersion, files, 'modpack')
		return await uploadHandle.promise
	}

	async function handleSave() {
		isSubmitting.value = true
		try {
			let patchSuccess
			switch (compatibilityType.value) {
				case 'vanilla':
					patchSuccess = await patchProjectV3({
						minecraft_java_server: {
							content: {
								kind: 'vanilla',
								supported_game_versions: supportedGameVersions.value,
								recommended_game_version: recommendedGameVersion.value,
							},
						},
					})

					break
				case 'published-modpack':
					patchSuccess = await patchProjectV3({
						minecraft_java_server: {
							content: {
								kind: 'modpack',
								version_id: selectedVersionId.value,
							},
						},
					})

					break
				case 'custom-modpack': {
					if (!customModpackFile.value) break

					// Upload the modpack file as a new version
					let uploadedVersion: Labrinth.Versions.v3.Version
					try {
						uploadedVersion = await uploadCustomModpackFile(customModpackFile.value)
					} catch (err: unknown) {
						const error = err as { data?: { description?: string } }
						addNotification({
							title: 'Failed to upload modpack',
							text: error.data?.description || String(err),
							type: 'error',
						})
						return
					}

					// Patch the project to point to the newly uploaded version
					patchSuccess = await patchProjectV3({
						minecraft_java_server: {
							content: {
								kind: 'modpack',
								version_id: uploadedVersion.id,
							},
						},
					})

					// If patch fails, clean up the uploaded version
					if (!patchSuccess) {
						try {
							await labrinth.versions_v3.deleteVersion(uploadedVersion.id)
						} catch {
							console.error('Failed to clean up uploaded version after patch failure')
						}
					}

					break
				}
			}
			if (!patchSuccess) {
				throw new Error('Failed to patch project with new server compatibility settings')
			}
			modal.value?.hide()
		} finally {
			isSubmitting.value = false
		}
	}

	function resetContext() {
		compatibilityType.value = null
		selectedProjectId.value = ''
		selectedVersionId.value = ''
		supportedGameVersions.value = []
		recommendedGameVersion.value = null
		customModpackFile.value = null
		hasLicensePermission.value = false
		isEditingExistingCompatibility.value = false
		isSwitchingCompatibilityType.value = false
	}

	return {
		stageConfigs,
		modal,
		isSubmitting,
		compatibilityType,
		selectedProjectId,
		selectedVersionId,
		supportedGameVersions,
		recommendedGameVersion,
		customModpackFile,
		hasLicensePermission,
		isEditingExistingCompatibility,
		isSwitchingCompatibilityType,
		resetContext,
		handleSave,
	}
}

const selectCompatibilityTypeStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-compatibility-type',
	stageContent: markRaw(SelectCompatibilityType),
	title: 'Compatibility type',
	cannotNavigateForward: (ctx) => !ctx.compatibilityType.value,
	leftButtonConfig: null,
	rightButtonConfig: null,
}

const selectVanillaVersionsStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-vanilla-versions',
	stageContent: markRaw(SelectVanillaVersions),
	title: 'Vanilla versions',
	skip: (ctx) => ctx.compatibilityType.value !== 'vanilla' && !!ctx.compatibilityType.value,
	leftButtonConfig: (ctx) =>
		ctx.isEditingExistingCompatibility.value
			? {
					label: 'Cancel',
					icon: XIcon,
					onClick: () => ctx.modal.value?.hide(),
				}
			: {
					label: 'Back',
					icon: LeftArrowIcon,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) =>
		ctx.isSwitchingCompatibilityType.value
			? {
					label: 'Change type',
					icon: ArrowLeftRightIcon,
					iconPosition: 'before' as const,
					color: 'red' as const,
					disabled:
						ctx.isSubmitting.value ||
						ctx.supportedGameVersions.value.length === 0 ||
						!ctx.recommendedGameVersion.value,
					onClick: () => ctx.handleSave(),
				}
			: {
					label: ctx.isSubmitting.value
						? ctx.isEditingExistingCompatibility.value
							? 'Updating…'
							: 'Saving…'
						: ctx.isEditingExistingCompatibility.value
							? 'Save changes'
							: 'Save',
					icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
					iconPosition: 'before' as const,
					iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
					color: 'green' as const,
					disabled:
						ctx.isSubmitting.value ||
						ctx.supportedGameVersions.value.length === 0 ||
						!ctx.recommendedGameVersion.value,
					onClick: () => ctx.handleSave(),
				},
	nonProgressStage: (ctx) => ctx.isEditingExistingCompatibility.value,
}

const selectPublishedModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-published-modpack',
	stageContent: markRaw(SelectPublishedModpack),
	title: 'Select modpack',
	skip: (ctx) => ctx.compatibilityType.value !== 'published-modpack',
	cannotNavigateForward: (ctx) => !ctx.selectedProjectId.value || !ctx.selectedVersionId.value,
	leftButtonConfig: (ctx) =>
		ctx.isEditingExistingCompatibility.value
			? {
					label: 'Cancel',
					icon: XIcon,
					onClick: () => ctx.modal.value?.hide(),
				}
			: {
					label: 'Back',
					icon: LeftArrowIcon,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) =>
		ctx.isSwitchingCompatibilityType.value
			? {
					label: 'Change type',
					icon: ArrowLeftRightIcon,
					iconPosition: 'before' as const,
					color: 'red' as const,
					disabled:
						ctx.isSubmitting.value || !ctx.selectedProjectId.value || !ctx.selectedVersionId.value,
					onClick: () => ctx.handleSave(),
				}
			: {
					label: ctx.isSubmitting.value
						? ctx.isEditingExistingCompatibility.value
							? 'Updating…'
							: 'Saving…'
						: ctx.isEditingExistingCompatibility.value
							? 'Save changes'
							: 'Save',
					icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
					iconPosition: 'before' as const,
					iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
					color: 'green' as const,
					disabled:
						ctx.isSubmitting.value || !ctx.selectedProjectId.value || !ctx.selectedVersionId.value,
					onClick: () => ctx.handleSave(),
				},
	nonProgressStage: (ctx) => ctx.isEditingExistingCompatibility.value,
}

const uploadCustomModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'upload-custom-modpack',
	stageContent: markRaw(UploadCustomModpack),
	title: 'Custom modpack',
	skip: (ctx) => ctx.compatibilityType.value !== 'custom-modpack',
	leftButtonConfig: (ctx) =>
		ctx.isEditingExistingCompatibility.value
			? {
					label: 'Cancel',
					icon: XIcon,
					onClick: () => ctx.modal.value?.hide(),
				}
			: {
					label: 'Back',
					icon: LeftArrowIcon,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) =>
		ctx.isSwitchingCompatibilityType.value
			? {
					label: 'Change type',
					icon: ArrowLeftRightIcon,
					iconPosition: 'before' as const,
					color: 'red' as const,
					disabled:
						ctx.isSubmitting.value ||
						!ctx.customModpackFile.value ||
						!ctx.hasLicensePermission.value,
					onClick: () => ctx.handleSave(),
				}
			: {
					label: ctx.isSubmitting.value
						? ctx.isEditingExistingCompatibility.value
							? 'Updating…'
							: 'Saving…'
						: ctx.isEditingExistingCompatibility.value
							? 'Save changes'
							: 'Save',
					icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
					iconPosition: 'before' as const,
					iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
					color: 'green' as const,
					disabled:
						ctx.isSubmitting.value ||
						!ctx.customModpackFile.value ||
						!ctx.hasLicensePermission.value,
					onClick: () => ctx.handleSave(),
				},
	nonProgressStage: (ctx) => ctx.isEditingExistingCompatibility.value,
}

const stageConfigs: StageConfigInput<ServerCompatibilityContextValue>[] = [
	selectCompatibilityTypeStage,
	selectVanillaVersionsStage,
	selectPublishedModpackStage,
	uploadCustomModpackStage,
]
