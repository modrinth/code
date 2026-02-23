import { DownloadIcon, LeftArrowIcon, TrashIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ConfirmStage from '../components/ConfirmStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'confirm',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(ConfirmStage),
	skip: () => true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => {
		const isErase = ctx.hardReset.value && !ctx.isInitialSetup
		return {
			label: isErase ? 'Erase and install' : 'Install',
			icon: isErase ? TrashIcon : DownloadIcon,
			iconPosition: 'before' as const,
			color: isErase ? ('red' as const) : ('brand' as const),
			loading: ctx.loading.value,
			onClick: () => ctx.finish(),
		}
	},
	maxWidth: '520px',
}
