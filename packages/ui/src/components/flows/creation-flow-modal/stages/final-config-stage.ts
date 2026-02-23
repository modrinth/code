import { DownloadIcon, LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	if (ctx.flowType === 'world' && !ctx.worldName.value.trim()) return true
	if (ctx.setupType.value === 'vanilla' && !ctx.selectedGameVersion.value) return true
	return false
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
		const isOnboarding = ctx.flowType === 'server-onboarding'
		const isFinish = isWorld || isOnboarding
		return {
			label: isWorld ? 'Create world' : isOnboarding ? 'Install' : 'Continue',
			icon: isFinish ? (isWorld ? PlusIcon : DownloadIcon) : RightArrowIcon,
			iconPosition: isFinish ? ('before' as const) : ('after' as const),
			color: isFinish ? ('brand' as const) : undefined,
			disabled: isForwardBlocked(ctx),
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
