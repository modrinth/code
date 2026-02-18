import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import CustomSetupStage from '../components/CustomSetupStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	const baseCheck =
		!ctx.selectedGameVersion.value ||
		(!ctx.hideLoaderChips.value && !ctx.selectedLoader.value)
	if (ctx.flowType === 'instance') {
		return baseCheck || !ctx.instanceName.value.trim()
	}
	return baseCheck
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'custom-setup',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(CustomSetupStage),
	skip: (ctx) => ctx.setupType.value === 'modpack' || ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: (ctx) => {
		const isInstance = ctx.flowType === 'instance'
		const goesToNextStage = ctx.flowType === 'world' || ctx.flowType === 'server-onboarding'
		const disabled = isForwardBlocked(ctx)

		if (isInstance) {
			return {
				label: 'Create instance',
				icon: PlusIcon,
				iconPosition: 'before' as const,
				color: 'brand' as const,
				disabled,
				onClick: () => ctx.finish(),
			}
		}

		return {
			label: goesToNextStage ? 'Continue' : 'Finish',
			icon: goesToNextStage ? RightArrowIcon : null,
			iconPosition: 'after' as const,
			color: goesToNextStage ? undefined : ('brand' as const),
			disabled,
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
