<template>
	<MultiStageModal ref="modal" :stages="stages" />
</template>

<script setup lang="ts">
import { LeftArrowIcon, PlusIcon, RightArrowIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiStageModal,
} from '@modrinth/ui'
import { provide } from 'vue'

import { useManageVersion } from '~/composables/versions/manage-version'

import type { Labrinth } from '@modrinth/api-client'
import AddChangelogStage from './stages/AddChangelogStage.vue'
import AddDependenciesStage from './stages/AddDependenciesStage.vue'
import AddDetailsStage from './stages/AddDetailsStage.vue'
import AddEnvironmentStage from './stages/AddEnvironmentStage.vue'
import AddFilesStage from './stages/AddFilesStage.vue'
import AddLoadersStage from './stages/AddLoadersStage.vue'
import AddMcVersionsStage from './stages/AddMcVersionsStage.vue'

const {
	newDraftVersion,
	filesToAdd,
	editingVersion,
	draftVersion,
	detectedLoaders,
	detectedVersions,
	projectType,
	existingFilesToDelete,
	setProjectType,
} = useManageVersion()

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

const addFilesNextDisabled = computed(
	() => filesToAdd.value.length === 0 && (draftVersion.value.existing_files?.length ?? 0) === 0,
)

const addDetailsNextDisabled = computed(() => draftVersion.value.version_number.trim().length === 0)

const addLoadersNextDisabled = computed(() => draftVersion.value.loaders.length === 0)
const addMcVersionsNextDisabled = computed(() => draftVersion.value.game_versions.length === 0)

const hideAddLoadersStage = computed(
	() =>
		projectType.value === 'resourcepack' || detectedLoaders.value || editingVersion.value === true,
)
const hideAddMcVersionsStage = computed(
	() => detectedVersions.value || editingVersion.value === true,
)

const hideAddDependenciesStage = computed(() => projectType.value === 'modpack')

const hideAddEnvironmentStage = computed(
	() => projectType.value !== 'mod' && projectType.value !== 'modpack',
)

function getNextLabel() {
	const currentStageIndex = modal.value?.currentStageIndex || 0
	const visibleStages = stages.value
	if (!visibleStages) return 'Next'

	const next = visibleStages[currentStageIndex + 1]
	if (!next) return 'Done'

	switch (next.id) {
		case 'add-details':
			return editingVersion.value ? 'Edit details' : 'Add details'
		case 'add-files':
			return editingVersion.value ? 'Edit files' : 'Add files'
		case 'add-loaders':
			return editingVersion.value ? 'Edit loaders' : 'Set loaders'
		case 'add-mc-versions':
			return editingVersion.value ? 'Edit MC versions' : 'Set MC versions'
		case 'add-dependencies':
			return editingVersion.value ? 'Edit dependencies' : 'Set dependencies'
		case 'add-environment':
			return editingVersion.value ? 'Edit environment' : 'Add environment'
		case 'add-changelog':
			return editingVersion.value ? 'Edit changelog' : 'Add changelog'
		default:
			return 'Next'
	}
}

const stages = computed<InstanceType<typeof MultiStageModal>['$props']['stages']>(
	() =>
		[
			{
				title: editingVersion.value ? 'Edit files' : 'Add files',
				id: 'add-files',
				stageContent: markRaw(AddFilesStage),
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
				title: editingVersion.value ? 'Edit details' : 'Add details',
				id: 'add-details',
				stageContent: markRaw(AddDetailsStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addDetailsNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddLoadersStage.value === false && {
				title: editingVersion.value ? 'Edit loaders' : 'Add loaders',
				id: 'add-loaders',
				stageContent: markRaw(AddLoadersStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addLoadersNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddMcVersionsStage.value === false && {
				title: editingVersion.value ? 'Edit MC versions' : 'Add MC versions',
				id: 'add-mc-versions',
				stageContent: markRaw(AddMcVersionsStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					disabled: addMcVersionsNextDisabled.value,
					label: getNextLabel(),
				},
			},
			hideAddEnvironmentStage.value === false && {
				title: editingVersion.value ? 'Edit environment' : 'Add environment',
				id: 'add-environment',
				stageContent: markRaw(AddEnvironmentStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					label: getNextLabel(),
				},
			},
			hideAddDependenciesStage.value === false && {
				title: editingVersion.value ? 'Edit dependencies' : 'Add dependencies',
				id: 'add-dependencies',
				stageContent: markRaw(AddDependenciesStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					label: getNextLabel(),
				},
			},
			{
				title: editingVersion.value ? 'Edit changelog' : 'Add changelog',
				id: 'add-changelog',
				stageContent: markRaw(AddChangelogStage),
				leftButtonConfig: { ...defaultBackButton },
				rightButtonConfig: {
					...defaultNextButton,
					iconPosition: 'before' as const,
					color: 'green' as const,
					label: editingVersion.value ? 'Save changes' : 'Create version',
					icon: isSubmitting.value ? SpinnerIcon : PlusIcon,
					iconClass: isSubmitting.value ? 'animate-spin' : undefined,
					disabled: isSubmitting.value,
					onClick: () => (editingVersion.value ? handleSaveVersionEdits() : handleCreateVersion()),
				},
			},
			{
				title: 'Edit loaders',
				id: 'edit-loaders',
				stageContent: AddLoadersStage,
				leftButtonConfig: {
					...defaultBackButton,
					label: 'Back',
					onClick: () => modal.value?.setStage('add-details'),
				},
				rightButtonConfig: {
					...defaultNextButton,
					label: 'Continue',
					onClick: () => modal.value?.setStage(2),
				},
				nonProgressStage: true,
			},
			{
				title: 'Edit MC versions',
				id: 'edit-mc-versions',
				stageContent: AddMcVersionsStage,
				leftButtonConfig: {
					...defaultBackButton,
					label: 'Back',
					onClick: () => modal.value?.setStage('add-details'),
				},
				rightButtonConfig: {
					...defaultNextButton,
					label: 'Continue',
					onClick: () => modal.value?.setStage(2),
				},
				nonProgressStage: true,
			},
		].filter(Boolean) as InstanceType<typeof MultiStageModal>['$props']['stages'],
)

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { refreshVersions } = injectProjectPageContext()

const isSubmitting = ref(false)

async function handleCreateVersion() {
	const version = toRaw(draftVersion.value)
	const files = toRaw(filesToAdd.value)
	isSubmitting.value = true

	try {
		await client.labrinth.versions_v3.createVersion(version, files)
		modal.value?.hide()
		addNotification({
			title: 'Project version created',
			text: 'The version has been successfully added to your project.',
			type: 'success',
		})
		await refreshVersions()
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	isSubmitting.value = false
}

async function handleSaveVersionEdits() {
	const version = toRaw(draftVersion.value)
	const files = toRaw(filesToAdd.value)
	const filesToDelete = toRaw(existingFilesToDelete.value)

	isSubmitting.value = true

	try {
		if (!version.version_id) throw new Error('Version ID is required to save edits.')
		// TODO DEV-595 need to properly pass version.environment into request body for creating and modifying a version

		await client.labrinth.versions_v3.modifyVersion(version.version_id, {
			version_title: version.version_title || version.version_number,
			version_number: version.version_number,
			changelog: version.changelog,
			version_type: version.version_type,
			dependencies: version.dependencies || [],
			game_versions: version.game_versions,
			loaders: version.loaders,
		})

		if (files.length > 0) {
			await client.labrinth.versions_v3.addFilesToVersion(version.version_id, files)
		}

		// Delete files that were marked for deletion
		for (const hash of filesToDelete) {
			await useBaseFetch(`version_file/${hash}?version_id=${version.version_id}`, {
				method: 'DELETE',
			})
		}

		modal.value?.hide()
		addNotification({
			title: 'Project version saved',
			text: 'The version has been successfully saved to your project.',
			type: 'success',
		})
		await refreshVersions()
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	isSubmitting.value = false
}

const { projectV2 } = injectProjectPageContext()

defineExpose({
	show: (version: Labrinth.Versions.v3.DraftVersion | null = null) => {
		newDraftVersion(projectV2.value.id, version)
		setProjectType(projectV2.value)
		modal.value?.setStage(0)
		modal.value?.show()
	},
})
</script>
