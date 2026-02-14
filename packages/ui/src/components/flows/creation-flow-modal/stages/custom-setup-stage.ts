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
	rightButtonConfig: (ctx) => {
		const goesToNextStage = ctx.flowType === 'world' || ctx.flowType === 'server-onboarding'
		return {
			label: goesToNextStage ? 'Continue' : 'Finish',
			icon: goesToNextStage ? RightArrowIcon : null,
			iconPosition: 'after' as const,
			color: goesToNextStage ? undefined : ('brand' as const),
			disabled:
				!ctx.selectedGameVersion.value ||
				(!ctx.hideLoaderFields.value && !ctx.selectedLoader.value),
			onClick: () => {
				if (goesToNextStage) {
					ctx.modal.value?.nextStage()
				} else {
					ctx.finish()
				}
			},
		}
	},
}
