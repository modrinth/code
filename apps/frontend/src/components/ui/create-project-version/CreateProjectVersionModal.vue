<template>
	<MultiStageModal ref="modal" :stages="stages" />
</template>

<script setup lang="ts">
import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import { commonMessages } from '@modrinth/ui'

import MultiStageModal from '@modrinth/ui/src/components/base/MultiStageModal.vue'
import AddChangelogStage from './stages/AddChangelogStage.vue'
import AddDependenciesStage from './stages/AddDependenciesStage.vue'
import AddDetailsStage from './stages/AddDetailsStage.vue'
import AddFilesStage from './stages/AddFilesStage.vue'
import AddMcVersionsStage from './stages/AddMcVersionsStage.vue'

const { formatMessage } = useVIntl()

const modal = useTemplateRef<InstanceType<typeof MultiStageModal>>('modal')

const defaultNextButton = {
	icon: RightArrowIcon,
	label: formatMessage(commonMessages.nextButton),
	disabled: false,
	color: 'standard' as const,
	iconPosition: 'after' as const,
	onClick: () => modal.value?.nextStage(),
}

const defaultBackButton = {
	icon: LeftArrowIcon,
	label: formatMessage(commonMessages.backButton),
	disabled: false,
	color: 'standard' as const,
	iconPosition: 'before' as const,
	onClick: () => modal.value?.prevStage(),
}

const stages: InstanceType<typeof MultiStageModal>['$props']['stages'] = [
	{
		title: 'Add Files',
		stageContent: AddFilesStage,
		leftButtonConfig: null,
		rightButtonConfig: { ...defaultNextButton },
	},
	{
		title: 'Add Details',
		stageContent: AddDetailsStage,
		leftButtonConfig: { ...defaultBackButton },
		rightButtonConfig: { ...defaultNextButton },
	},
	{
		title: 'Add Minecraft Versions',
		stageContent: AddMcVersionsStage,
		leftButtonConfig: { ...defaultBackButton },
		rightButtonConfig: { ...defaultNextButton },
	},
	{
		title: 'Add Changelog',
		stageContent: AddChangelogStage,
		leftButtonConfig: { ...defaultBackButton },
		rightButtonConfig: { ...defaultNextButton },
	},
	{
		title: 'Add Dependencies',
		stageContent: AddDependenciesStage,
		leftButtonConfig: { ...defaultBackButton },
		rightButtonConfig: {
			...defaultNextButton,
			icon: null,
			label: formatMessage(commonMessages.closeButton),
			onClick: () => modal.value?.hide(),
		},
	},
]

defineExpose({
	show: () => modal.value?.show(),
})
</script>
