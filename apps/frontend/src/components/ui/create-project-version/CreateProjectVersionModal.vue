<template>
	<MultiStageModal ref="modal" :stages="stages" />
</template>

<script setup lang="ts">
import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { commonMessages, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import MultiStageModal from '@modrinth/ui/src/components/base/MultiStageModal.vue'
import { defineMessages } from '@vintl/vintl'

import { useManageVersion } from '~/composables/versions/manage-version'

import { watch } from 'vue'
import { injectVersionsContext } from '~/providers/versions'
import AddChangelogStage from './stages/AddChangelogStage.vue'
import AddDependenciesStage from './stages/AddDependenciesStage.vue'
import AddDetailsStage from './stages/AddDetailsStage.vue'
import AddFilesStage from './stages/AddFilesStage.vue'
import AddMcVersionsStage from './stages/AddMcVersionsStage.vue'

const { newDraftVersion, draftVersion } = useManageVersion()

watch(
	draftVersion,
	() => {
		console.log(toRaw(draftVersion.value))
	},
	{ deep: true },
)

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

const addFilesDisabled = computed(() => draftVersion.value.files.length === 0)

const addDetailsDisabled = computed(
	() =>
		draftVersion.value.version_title.trim().length === 0 ||
		draftVersion.value.version_number.trim().length === 0 ||
		draftVersion.value.loaders.length === 0,
)

const addMcVersionsDisabled = computed(() => draftVersion.value.game_versions.length === 0)

const stages = computed<InstanceType<typeof MultiStageModal>['$props']['stages']>(
	() =>
		[
			{
				title: formatMessage(messages.addFilesTitle),
				stageContent: AddFilesStage,
				leftButtonConfig: null,
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addFilesDisabled.value,
					label: formatMessage(messages.addDetailsButton),
				},
			},
			{
				title: formatMessage(messages.addDetailsTitle),
				stageContent: AddDetailsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addDetailsDisabled.value,
					label: formatMessage(messages.addMcVersionsButton),
				},
			},
			{
				title: formatMessage(messages.addMcVersionsTitle),
				stageContent: AddMcVersionsStage,
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addMcVersionsDisabled.value,
					label: formatMessage(messages.addChangelogButton),
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
			{
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
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const { project } = injectVersionsContext()

defineExpose({
	show: () => {
		newDraftVersion(project.id)
		modal.value?.setStage(0)
		modal.value?.show()
	},
})
</script>
