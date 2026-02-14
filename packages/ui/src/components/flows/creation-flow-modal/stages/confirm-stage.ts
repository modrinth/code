import { LeftArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ConfirmStage from '../components/ConfirmStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'confirm',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(ConfirmStage),
	skip: (ctx) => ctx.flowType !== 'server-onboarding',
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.hardReset.value && !ctx.isInitialSetup ? 'Erase and install' : 'Install',
		color: ctx.hardReset.value && !ctx.isInitialSetup ? ('red' as const) : ('brand' as const),
		onClick: () => ctx.finish(),
	}),
}
