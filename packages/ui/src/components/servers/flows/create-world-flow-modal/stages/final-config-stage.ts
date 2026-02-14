import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '../../../../base'
import { markRaw } from 'vue'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import type { CreateWorldContextValue } from '../create-world-context'

export const stageConfig: StageConfigInput<CreateWorldContextValue> = {
	id: 'final-config',
	title: 'Create world',
	stageContent: markRaw(FinalConfigStage),
	cannotNavigateForward: (ctx) => !ctx.worldName.value.trim(),
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: 'Continue',
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled: !ctx.worldName.value.trim(),
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}
