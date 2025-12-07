<template>
	<MultiStageModal ref="modal" :stages="stages" />
</template>

<script setup lang="ts">
import { LeftArrowIcon, PlusIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import {
	commonMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiStageModal,
} from '@modrinth/ui'
import { defineMessages } from '@vintl/vintl'

import { useManageVersion } from '~/composables/versions/manage-version'

import AddChangelogStage from './stages/AddChangelogStage.vue'
import AddDependenciesStage from './stages/AddDependenciesStage.vue'
import AddDetailsStage from './stages/AddDetailsStage.vue'
import AddFilesStage from './stages/AddFilesStage.vue'
import AddLoadersStage from './stages/AddLoadersStage.vue'
import AddMcVersionsStage from './stages/AddMcVersionsStage.vue'

const { newDraftVersion, draftVersion } = useManageVersion()

const { formatMessage } = useVIntl()

const modal = useTemplateRef<InstanceType<typeof MultiStageModal>>('modal')

const messages = defineMessages({
	addFilesTitle: {
		id: 'create-project-version.stage.add-files.title',
		defaultMessage: 'Add Files',
	},
	addDetailsTitle: {
		id: 'create-project-version.stage.add-details.title',
		defaultMessage: 'Add Details',
	},
	addMcVersionsTitle: {
		id: 'create-project-version.stage.add-mc-versions.title',
		defaultMessage: 'Add MC Versions',
	},
	addChangelogTitle: {
		id: 'create-project-version.stage.add-changelog.title',
		defaultMessage: 'Add Changelog',
	},
	addDependenciesTitle: {
		id: 'create-project-version.stage.add-dependencies.title',
		defaultMessage: 'Add Dependencies',
	},
	addDetailsButton: {
		id: 'create-project-version.button.add-details',
		defaultMessage: 'Add details',
	},
	addMcVersionsButton: {
		id: 'create-project-version.button.add-mc-versions',
		defaultMessage: 'Add MC versions',
	},
	addChangelogButton: {
		id: 'create-project-version.button.add-changelog',
		defaultMessage: 'Add changelog',
	},
	addDependenciesButton: {
		id: 'create-project-version.button.add-dependencies',
		defaultMessage: 'Add dependencies',
	},
})

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

const addFilesNextDisabled = computed(() => draftVersion.value.files.length === 0)

const addDetailsNextDisabled = computed(() => draftVersion.value.version_number.trim().length === 0)

const addLoadersNextDisabled = computed(() => draftVersion.value.loaders.length === 0)
const addMcVersionsNextDisabled = computed(() => draftVersion.value.game_versions.length === 0)

const stages = computed<InstanceType<typeof MultiStageModal>['$props']['stages']>(
	() =>
		[
			{
				title: formatMessage(messages.addFilesTitle),
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
					label: formatMessage(messages.addDetailsButton),
				},
			},
			{
				title: formatMessage(messages.addDetailsTitle),
				stageContent: AddDetailsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addDetailsNextDisabled.value,
					label: 'Set loaders',
					onClick: () => {
						// if has detected loaders, skip step (jump straight to next step)
						// check if has detected versions
						// -true: go to dependencies
						// -else: go to version picker
						// if has detected versions skip step (jump straight to next step)
						// check if has detected loaders
						// -true: go to dependencies
						// -else: go to loaders select
					},
				},
			},
			{
				// dont show for resource packs
				title: 'Add loaders',
				stageContent: AddLoadersStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addLoadersNextDisabled.value,
					label: 'Set MC versions',
				},
			},
			{
				title: formatMessage(messages.addMcVersionsTitle),
				stageContent: AddMcVersionsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addMcVersionsNextDisabled.value,
					label: formatMessage(messages.addChangelogButton),
				},
			},
			{
				// skip this step for modpacks
				title: formatMessage(messages.addDependenciesTitle),
				stageContent: AddDependenciesStage,
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
				title: formatMessage(messages.addChangelogTitle),
				stageContent: AddChangelogStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					label: formatMessage(messages.addDependenciesButton),
				},
			},
		] as InstanceType<typeof MultiStageModal>['$props']['stages'],
)
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
