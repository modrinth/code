import { LeftArrowIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { createContext, type MultiStageModal, type StageConfigInput } from '@modrinth/ui'
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
	onSave?: () => void,
): ServerCompatibilityContextValue {
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
			// TODO: implement save logic
			onSave?.()
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
