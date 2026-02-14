import { LeftArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '../../../../base'
import { markRaw } from 'vue'
import ModpackStage from '../components/ModpackStage.vue'
import type { CreateWorldContextValue } from '../create-world-context'

export const stageConfig: StageConfigInput<CreateWorldContextValue> = {
	id: 'modpack',
	title: 'Create world',
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.worldType.value !== 'modpack',
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('world-type'),
	}),
	rightButtonConfig: null,
	maxWidth: '520px',
}
