import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import CustomSetupStage from '../components/CustomSetupStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'custom-setup',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(CustomSetupStage),
	skip: (ctx) => ctx.worldType.value === 'modpack',
	cannotNavigateForward: (ctx) =>
		!ctx.selectedGameVersion.value || (!ctx.hideLoaderFields.value && !ctx.selectedLoader.value),
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('world-type'),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.flowType === 'world' ? 'Continue' : 'Finish',
		icon: ctx.flowType === 'world' ? RightArrowIcon : null,
		iconPosition: 'after',
		color: ctx.flowType === 'world' ? undefined : ('brand' as const),
		disabled:
			!ctx.selectedGameVersion.value || (!ctx.hideLoaderFields.value && !ctx.selectedLoader.value),
		onClick: () => {
			if (ctx.flowType === 'world') {
				ctx.modal.value?.nextStage()
			} else {
				ctx.finish()
			}
		},
	}),
}
