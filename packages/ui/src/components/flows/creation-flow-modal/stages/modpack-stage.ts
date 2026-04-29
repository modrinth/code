import { LeftArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import ModpackStage from '../components/ModpackStage.vue'
import { type CreationFlowContextValue, creationFlowMessages } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'modpack',
	title: (ctx) => ctx.formatMessage(creationFlowMessages.chooseModpackTitle),
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'modpack' || ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: null,
	maxWidth: '520px',
}
