import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '../../../../base'
import { markRaw } from 'vue'
import CustomSetupStage from '../components/CustomSetupStage.vue'
import type { CreateWorldContextValue } from '../create-world-context'

export const stageConfig: StageConfigInput<CreateWorldContextValue> = {
	id: 'custom-setup',
	title: 'Create world',
	stageContent: markRaw(CustomSetupStage),
	skip: (ctx) => ctx.worldType.value === 'modpack',
	cannotNavigateForward: (ctx) =>
		!ctx.selectedGameVersion.value ||
		(!ctx.hideLoaderFields.value && !ctx.selectedLoader.value),
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('world-type'),
	}),
	rightButtonConfig: (ctx) => ({
		label: 'Continue',
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled:
			!ctx.selectedGameVersion.value ||
			(!ctx.hideLoaderFields.value && !ctx.selectedLoader.value),
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}
