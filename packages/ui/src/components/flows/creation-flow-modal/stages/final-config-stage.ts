import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import {
	creationFlowMessages,
	type CreationFlowContextValue,
	flowTypeHeadingMessages,
} from '../creation-flow-context'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	if (ctx.flowType === 'world' && !ctx.worldName.value.trim()) return true
	if (ctx.setupType.value === 'vanilla' && !ctx.selectedGameVersion.value) return true
	return false
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'final-config',
	title: (ctx) => ctx.formatMessage(flowTypeHeadingMessages[ctx.flowType]),
	stageContent: markRaw(FinalConfigStage),
	skip: (ctx) => ctx.flowType === 'instance' || ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		onClick: () => {
			if (ctx.onBack) {
				ctx.onBack()
			} else {
				ctx.modal.value?.prevStage()
			}
		},
	}),
	rightButtonConfig: (ctx) => {
		const isWorld = ctx.flowType === 'world'
		const isOnboarding = ctx.flowType === 'server-onboarding'
		const isReset = ctx.flowType === 'reset-server'
		const isFinish = isWorld || isOnboarding || isReset
		const label = isWorld
			? ctx.formatMessage(creationFlowMessages.createWorldButton)
			: isReset
				? ctx.formatMessage(commonMessages.resetServerButton)
				: isOnboarding
					? ctx.formatMessage(creationFlowMessages.setupServerButton)
					: ctx.formatMessage(commonMessages.continueButton)
		return {
			label,
			icon: isFinish ? PlusIcon : RightArrowIcon,
			iconPosition: isFinish ? ('before' as const) : ('after' as const),
			color: isReset ? ('red' as const) : isFinish ? ('brand' as const) : undefined,
			disabled: isForwardBlocked(ctx) || ctx.isBackingUp.value,
			loading: isFinish && ctx.loading.value,
			onClick: () => {
				if (isFinish) {
					ctx.finish()
				} else {
					ctx.modal.value?.nextStage()
				}
			},
		}
	},
	maxWidth: '520px',
}
