import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import CustomSetupStage from '../components/CustomSetupStage.vue'
import {
	creationFlowMessages,
	type CreationFlowContextValue,
	flowTypeHeadingMessages,
} from '../creation-flow-context'

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	if (!ctx.selectedGameVersion.value) return true
	if (!ctx.hideLoaderChips.value && !ctx.selectedLoader.value) return true
	if (!ctx.hideLoaderVersion.value && !ctx.selectedLoaderVersion.value) return true
	return false
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'custom-setup',
	title: (ctx) => ctx.formatMessage(flowTypeHeadingMessages[ctx.flowType]),
	stageContent: markRaw(CustomSetupStage),
	skip: (ctx) =>
		ctx.setupType.value === 'modpack' ||
		ctx.setupType.value === 'vanilla' ||
		ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: (ctx) => {
		const isInstance = ctx.flowType === 'instance'
		const goesToNextStage =
			ctx.flowType === 'world' ||
			ctx.flowType === 'server-onboarding' ||
			ctx.flowType === 'reset-server'
		const disabled = isForwardBlocked(ctx)

		if (isInstance) {
			return {
				label: ctx.formatMessage(creationFlowMessages.createInstanceButton),
				icon: PlusIcon,
				iconPosition: 'before' as const,
				color: 'brand' as const,
				disabled,
				loading: ctx.loading.value,
				onClick: () => ctx.finish(),
			}
		}

		return {
			label: ctx.formatMessage(
				goesToNextStage ? commonMessages.continueButton : creationFlowMessages.finishButton,
			),
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
	maxWidth: '520px',
}
