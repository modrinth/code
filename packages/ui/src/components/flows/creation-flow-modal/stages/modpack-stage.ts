import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ModpackStage from '../components/ModpackStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

function hasModpackSelected(ctx: CreationFlowContextValue): boolean {
	return !!ctx.modpackSelection.value || !!ctx.modpackFilePath.value
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'modpack',
	title: 'Choose Modpack',
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'modpack' || ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: (ctx) => {
		const selected = hasModpackSelected(ctx)
		if (ctx.flowType === 'instance') {
			return {
				label: 'Create instance',
				icon: PlusIcon,
				iconPosition: 'before' as const,
				color: 'brand' as const,
				disabled: !selected,
				onClick: () => ctx.finish(),
			}
		}
		return {
			label: 'Continue',
			icon: RightArrowIcon,
			iconPosition: 'after' as const,
			disabled: !selected,
			onClick: () => ctx.modal.value?.setStage('final-config'),
		}
	},
	maxWidth: '520px',
}
