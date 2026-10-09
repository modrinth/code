import { ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import CurseForgeStage from '../components/curseforge-stage/index.vue'
import { type CreationFlowContextValue, flowTypeHeadingMessages } from '../creation-flow-context'
import { isCurseForgeModpackUrl } from '../curseforge'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	return (
		ctx.finishDisabled.value ||
		(!ctx.curseforgeModpackFile.value && !isCurseForgeModpackUrl(ctx.curseforgeUrl.value))
	)
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'curseforge',
	title: (ctx) => ctx.formatMessage(flowTypeHeadingMessages[ctx.flowType]),
	stageContent: markRaw(CurseForgeStage),
	skip: (ctx) => ctx.setupType.value !== 'curseforge' || ctx.flowType === 'instance',
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: ChevronLeftIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.continueButton),
		icon: ChevronRightIcon,
		iconPosition: 'after' as const,
		disabled: isForwardBlocked(ctx),
		tooltip: ctx.finishDisabled.value ? ctx.finishDisabledTooltip.value : undefined,
		onClick: () => {
			if (!isForwardBlocked(ctx)) ctx.modal.value?.setStage('final-config')
		},
	}),
	maxWidth: '544px',
}
