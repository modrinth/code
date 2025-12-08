<template>
	<MultiStageModal ref="modal" :stages="stages" />
</template>

<script setup lang="ts">
import { LeftArrowIcon, PlusIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import {
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiStageModal,
} from '@modrinth/ui'
import { provide } from 'vue'

import { useManageVersion } from '~/composables/versions/manage-version'

import AddChangelogStage from './stages/AddChangelogStage.vue'
import AddDependenciesStage from './stages/AddDependenciesStage.vue'
import AddDetailsStage from './stages/AddDetailsStage.vue'
import AddFilesStage from './stages/AddFilesStage.vue'
import AddLoadersStage from './stages/AddLoadersStage.vue'
import AddMcVersionsStage from './stages/AddMcVersionsStage.vue'

const { newDraftVersion, draftVersion, detectedLoaders, detectedVersions, projectType } =
	useManageVersion()

const modal = useTemplateRef<InstanceType<typeof MultiStageModal>>('modal')

provide('createVersionModal', modal)

const defaultNextButton = {
	icon: RightArrowIcon,
	label: 'Next',
	disabled: false,
	color: 'standard' as const,
	iconPosition: 'after' as const,
	onClick: () => modal.value?.nextStage(),
}

const defaultBackButton = {
	icon: LeftArrowIcon,
	label: 'Back',
	disabled: false,
	color: 'standard' as const,
	iconPosition: 'before' as const,
	onClick: () => modal.value?.prevStage(),
}

const addFilesNextDisabled = computed(() => draftVersion.value.files.length === 0)

const addDetailsNextDisabled = computed(() => draftVersion.value.version_number.trim().length === 0)

const addLoadersNextDisabled = computed(() => draftVersion.value.loaders.length === 0)
const addMcVersionsNextDisabled = computed(() => draftVersion.value.game_versions.length === 0)

const hideAddLoadersStage = computed(
	() => projectType.value === 'resourcepack' || detectedLoaders.value,
)
const hideAddMcVersionsStage = computed(() => detectedVersions.value)

const hideAddDependenciesStage = computed(() => projectType.value === 'modpack')

function getNextLabel() {
	const currentStageIndex = modal.value?.currentStageIndex || 0
	const visibleStages = stages.value
	if (!visibleStages) return 'Next'

	const next = visibleStages[currentStageIndex + 1]
	if (!next) return 'Done'

	switch (next.title) {
		case 'Add Details':
			return 'Add details'
		case 'Add Files':
			return 'Add files'
		case 'Add loaders':
			return 'Set loaders'
		case 'Add MC Versions':
			return 'Set MC versions'
		case 'Add Dependencies':
			return 'Set dependencies'
		case 'Add Changelog':
			return 'Add changelog'
		default:
			return 'Next'
	}
}

const stages = computed<InstanceType<typeof MultiStageModal>['$props']['stages']>(
	() =>
		[
			{
				title: 'Add files',
				stageContent: AddFilesStage,
				leftButtonConfig: {
					...defaultBackButton,
					label: 'Cancel',
					icon: XIcon,
					iconPosition: 'before' as const,
					onClick: () => modal.value?.hide(),
				},
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addFilesNextDisabled.value,
					label: getNextLabel(),
				},
			},
			{
				title: 'Add details',
				stageContent: AddDetailsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addDetailsNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddLoadersStage.value === false && {
				title: 'Add loaders',
				stageContent: AddLoadersStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addLoadersNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddMcVersionsStage.value === false && {
				title: 'Add MC versions',
				stageContent: AddMcVersionsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addMcVersionsNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddDependenciesStage.value === false && {
				title: 'Add dependencies',
				stageContent: AddDependenciesStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					label: getNextLabel(),
				},
			},
			{
				title: 'Add changelog',
				stageContent: AddChangelogStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					icon: PlusIcon,
					iconPosition: 'before' as const,
					color: 'green' as const,
					label: 'Create version',
					onClick: handleCreateVersion,
				},
			},
			{
				title: 'Edit loaders',
				stageContent: AddLoadersStage,
				leftButtonConfig: {
					...defaultBackButton,
					label: 'Save',
					onClick: () => modal.value?.setStage(1),
				},
				rightButtonConfig: {
					...defaultNextButton,
					label: 'Save & continue',
					onClick: () => modal.value?.setStage(2),
				},
			},
			{
				title: 'Edit MC versions',
				stageContent: AddMcVersionsStage,
				leftButtonConfig: {
					...defaultBackButton,
					label: 'Save',
					onClick: () => modal.value?.setStage(1),
				},
				rightButtonConfig: {
					...defaultNextButton,
					label: 'Save & continue',
					onClick: () => modal.value?.setStage(2),
				},
			},
		].filter(Boolean) as InstanceType<typeof MultiStageModal>['$props']['stages'],
)

watch(stages, () => console.log(stages.value))

const client = injectModrinthClient()

const { addNotification } = injectNotificationManager()

async function handleCreateVersion() {
	const version = toRaw(draftVersion.value)
	const files = version.files
	try {
		await client.labrinth.versions_v3.createVersion(version, files)
		modal.value?.hide()
		addNotification({
			title: 'Project version created',
			text: 'The version has been successfully added to your project.',
			type: 'success',
		})
		// TODO: refetch versions here for project versions table
		// (will have to not use page prop to get versions for table, instead use own state)
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const { projectV2 } = injectProjectPageContext()

defineExpose({
	show: () => {
		newDraftVersion(projectV2.value.id)
		modal.value?.setStage(0)
		modal.value?.show()
	},
})
</script>
