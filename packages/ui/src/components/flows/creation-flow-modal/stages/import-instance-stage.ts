import { DownloadIcon, LeftArrowIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ImportInstanceStage from '../components/ImportInstanceStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

function getSelectedCount(ctx: CreationFlowContextValue): number {
	let count = 0
	for (const set of Object.values(ctx.importSelectedInstances.value)) {
		count += set.size
	}
	return count
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'import-instance',
	title: 'Import instance',
	stageContent: markRaw(ImportInstanceStage),
	skip: (ctx) => !ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => {
			ctx.isImportMode.value = false
			ctx.modal.value?.setStage('setup-type')
		},
	}),
	rightButtonConfig: (ctx) => {
		const count = getSelectedCount(ctx)
		return {
			label: count > 0 ? `Import ${count} instance${count !== 1 ? 's' : ''}` : 'Import',
			icon: DownloadIcon,
			iconPosition: 'before' as const,
			color: 'brand' as const,
			disabled: count === 0,
			onClick: () => ctx.finish(),
		}
	},
	maxWidth: '520px',
}
