import { LeftArrowIcon, RightArrowIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { createContext, type MultiStageModal, type StageConfigInput } from '@modrinth/ui'
import type { Ref, ShallowRef } from 'vue'
import { markRaw } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import SelectPublishedModpack from './stages/SelectPublishedModpack.vue'
import SelectVanillaVersions from './stages/SelectVanillaVersions.vue'
import UploadCustomModpack from './stages/UploadCustomModpack.vue'

// ──────────────────────────────────────────────
// Context value
// ──────────────────────────────────────────────

export interface ServerCompatibilityContextValue {
	// Stage management
	stageConfigs: StageConfigInput<ServerCompatibilityContextValue>[]
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	isSubmitting: Ref<boolean>

	// State – extend as needed
	selectedModpackId: Ref<string | null>
	selectedVanillaVersions: Ref<string[]>
	customModpackFile: Ref<File | null>

	// Actions
	handleSave: () => Promise<void>
}

// ──────────────────────────────────────────────
// Stage configs
// ──────────────────────────────────────────────

const selectPublishedModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'select-published-modpack',
	stageContent: markRaw(SelectPublishedModpack),
	title: 'Select modpack',
	leftButtonConfig: (ctx) => ({
		label: 'Cancel',
		icon: XIcon,
		onClick: () => ctx.modal.value?.hide(),
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

const uploadCustomModpackStage: StageConfigInput<ServerCompatibilityContextValue> = {
	id: 'upload-custom-modpack',
	stageContent: markRaw(UploadCustomModpack),
	title: 'Custom modpack',
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
	selectPublishedModpackStage,
	selectVanillaVersionsStage,
	uploadCustomModpackStage,
]

// ──────────────────────────────────────────────
// Provide / inject
// ──────────────────────────────────────────────

export const [injectServerCompatibilityContext, provideServerCompatibilityContext] =
	createContext<ServerCompatibilityContextValue>('ServerCompatibilityModal')

// ──────────────────────────────────────────────
// Factory
// ──────────────────────────────────────────────

export function createServerCompatibilityContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
	onSave?: () => void,
): ServerCompatibilityContextValue {
	const isSubmitting = ref(false)
	const selectedModpackId = ref<string | null>(null)
	const selectedVanillaVersions = ref<string[]>([])
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
		selectedModpackId,
		selectedVanillaVersions,
		customModpackFile,
		handleSave,
	}
}
