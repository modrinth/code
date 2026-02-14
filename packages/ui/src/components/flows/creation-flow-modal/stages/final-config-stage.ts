import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'final-config',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(FinalConfigStage),
	skip: (ctx) => ctx.flowType !== 'world',
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
