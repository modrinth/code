import { LeftArrowIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import {
	createContext,
	injectProjectPageContext,
	type MultiStageModal,
	type StageConfigInput,
} from '@modrinth/ui'
import type { Ref, ShallowRef } from 'vue'
import { markRaw } from 'vue'
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

	// Actions
	resetContext: () => void
	handleSave: () => Promise<void>
}

export const [injectServerCompatibilityContext, provideServerCompatibilityContext] =
	createContext<ServerCompatibilityContextValue>('ServerCompatibilityModal')

export function createServerCompatibilityContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
): ServerCompatibilityContextValue {
	const { patchProjectV3 } = injectProjectPageContext()

	const isSubmitting = ref(false)
	const compatibilityType = ref<CompatibilityType | null>(null)
	const selectedProjectId = ref('')
	const selectedVersionId = ref('')
	const supportedGameVersions = ref<string[]>([])
	const recommendedGameVersion = ref<string | null>(null)
	const customModpackFile = ref<File | null>(null)

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
				case 'custom-modpack':
					// TODO: implement custom modpack save
					// upload modpack file
					// if modpack upload fails, show error and don't patch project
					// otherwise, patch project to still be kind: "modpack"
					// and have version_id point to the newly uploaded modpack version
					// if patch project fails, show error, delete the uploaded modpack version
					break
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
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.isSubmitting.value ? 'Saving…' : 'Save',
		icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
		iconPosition: 'before',
		iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled:
			ctx.isSubmitting.value ||
			ctx.supportedGameVersions.value.length === 0 ||
			!ctx.recommendedGameVersion.value,
		onClick: () => ctx.handleSave(),
	}),
}

const selectPublishedModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-published-modpack',
	stageContent: markRaw(SelectPublishedModpack),
	title: 'Select modpack',
	skip: (ctx) => ctx.compatibilityType.value !== 'published-modpack',
	cannotNavigateForward: (ctx) => !ctx.selectedProjectId.value || !ctx.selectedVersionId.value,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => {
		return {
			label: ctx.isSubmitting.value ? 'Saving…' : 'Save',
			icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
			iconPosition: 'before' as const,
			iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
			color: 'green' as const,
			disabled:
				ctx.isSubmitting.value || !ctx.selectedProjectId.value || !ctx.selectedVersionId.value,
			onClick: () => ctx.handleSave(),
		}
	},
}

const uploadCustomModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'upload-custom-modpack',
	stageContent: markRaw(UploadCustomModpack),
	title: 'Custom modpack',
	skip: (ctx) => ctx.compatibilityType.value !== 'custom-modpack',
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.isSubmitting.value ? 'Saving…' : 'Save',
		icon: ctx.isSubmitting.value ? SpinnerIcon : SaveIcon,
		iconPosition: 'before',
		iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled: ctx.isSubmitting.value,
		onClick: () => ctx.handleSave(),
	}),
}

const stageConfigs: StageConfigInput<ServerCompatibilityContextValue>[] = [
	selectCompatibilityTypeStage,
	selectVanillaVersionsStage,
	selectPublishedModpackStage,
	uploadCustomModpackStage,
]
