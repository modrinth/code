import { LeftArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import SelectInstanceStage from '../components/SelectInstanceStage.vue'
import { type CreationFlowContextValue, creationFlowMessages } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'select-instance',
	title: (ctx) => ctx.formatMessage(creationFlowMessages.selectInstanceTitle),
	stageContent: markRaw(SelectInstanceStage),
	skip: (ctx) => !ctx.isInstancePickMode.value,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		onClick: () => {
			ctx.isInstancePickMode.value = false
			ctx.instanceSelection.value = null
			ctx.modal.value?.setStage('setup-type')
		},
	}),
	rightButtonConfig: null,
	maxWidth: '560px',
}
