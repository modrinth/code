import { LeftArrowIcon, RightArrowIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
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
	selectedModpackId: Ref<string | null>
	supportedGameVersions: Ref<string[]>
	recommendedGameVersion: Ref<string | null>
	customModpackFile: Ref<File | null>

	// Actions
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
	const selectedModpackId = ref<string | null>(null)
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

	return {
		stageConfigs,
		modal,
		isSubmitting,
		compatibilityType,
		selectedModpackId,
		supportedGameVersions,
		recommendedGameVersion,
		customModpackFile,
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

const selectPublishedModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-published-modpack',
	stageContent: markRaw(SelectPublishedModpack),
	title: 'Select modpack',
	skip: (ctx) => ctx.compatibilityType.value !== 'published-modpack',
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: 'Next',
		icon: RightArrowIcon,
		iconPosition: 'after',
		onClick: () => ctx.modal.value?.nextStage(),
	}),
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
		disabled: ctx.isSubmitting.value,
		onClick: () => ctx.handleSave(),
	}),
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
