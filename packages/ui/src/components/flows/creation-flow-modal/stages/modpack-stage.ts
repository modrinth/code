import { LeftArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ModpackStage from '../components/ModpackStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'modpack',
	title: 'Choose modpack',
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'modpack' || ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: null,
	maxWidth: '520px',
}
