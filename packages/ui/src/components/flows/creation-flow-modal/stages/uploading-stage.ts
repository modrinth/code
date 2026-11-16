import { LeftArrowIcon, PlusIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import UploadingStage from '../components/UploadingStage.vue'
import {
	type CreationFlowContextValue,
	creationFlowMessages,
	flowTypeHeadingMessages,
} from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'uploading',
	title: (ctx) => ctx.formatMessage(flowTypeHeadingMessages[ctx.flowType]),
	stageContent: markRaw(UploadingStage),
	skip: (ctx) => ctx.uploadProgress.value === null,
	nonProgressStage: true,
	disableClose: (ctx) => ctx.uploadProgress.value !== null,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		disabled: true,
	}),
	rightButtonConfig: (ctx) => {
		const isReset = ctx.flowType === 'reset-server'
		return {
			label: isReset
				? ctx.formatMessage(commonMessages.resetServerButton)
				: ctx.formatMessage(creationFlowMessages.setupServerButton),
			icon: isReset ? RotateCounterClockwiseIcon : PlusIcon,
			iconPosition: 'before' as const,
			color: isReset ? ('red' as const) : ('brand' as const),
			disabled: true,
		}
	},
	maxWidth: '544px',
}
