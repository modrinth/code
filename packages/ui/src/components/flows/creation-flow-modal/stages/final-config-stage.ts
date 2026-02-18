import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	return ctx.flowType === 'world' && !ctx.worldName.value.trim()
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'final-config',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(FinalConfigStage),
	skip: (ctx) => ctx.flowType === 'instance' || ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => {
		const isWorld = ctx.flowType === 'world'
		return {
			label: isWorld ? 'Create' : 'Continue',
			icon: isWorld ? null : RightArrowIcon,
			iconPosition: 'after' as const,
			color: isWorld ? ('brand' as const) : undefined,
			disabled: isForwardBlocked(ctx),
			onClick: () => {
				if (isWorld) {
					ctx.finish()
				} else {
					ctx.modal.value?.nextStage()
				}
			},
		}
	},
	maxWidth: '520px',
}
