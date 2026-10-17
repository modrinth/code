<template>
	<MultiStageModal
		ref="modal"
		:stages="ctx.stageConfigs"
		:context="ctx"
		:breadcrumbs="!editingVersion"
		:close-on-click-outside="false"
		@hide="() => (modalOpen = false)"
	/>
	<DropArea
		v-if="enableDropArea && !modalOpen"
		:accept="acceptFileFromProjectType(projectV2.project_type)"
		@change="handleDropArea"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	DropArea,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiStageModal,
} from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createManageVersionContext,
	type ManageVersionHost,
	provideManageVersionContext,
} from '~/providers/version/manage-version-modal'

const props = withDefaults(
	defineProps<{
		host?: ManageVersionHost
		enableDropArea?: boolean
	}>(),
	{ enableDropArea: true },
)
const projectPageContext = injectProjectPageContext(null)
const host = props.host ?? projectPageContext
if (!host) throw new Error('Version editor requires a project context')
const { projectV2 } = host
let editRequest = 0
onBeforeUnmount(() => {
	editRequest += 1
})

const emit = defineEmits<{
	(e: 'save'): void
}>()

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')
const modalOpen = ref(false)

const ctx = createManageVersionContext(modal, host, () => emit('save'))
provideManageVersionContext(ctx)

const { newDraftVersion, editingVersion, handleNewFiles } = ctx

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

async function openEditVersionModal(versionId: string, projectId: string, stageId?: string | null) {
	const request = ++editRequest
	try {
		const versionData = await labrinth.versions_v3.getVersion(versionId)

		if (request !== editRequest || projectV2.value.id !== projectId) return

		const draftVersionData: Labrinth.Versions.v3.DraftVersion = {
			project_id: projectId,
			version_id: versionId,
			name: versionData.name ?? '',
			version_number: versionData.version_number ?? '',
			changelog: versionData.changelog ?? '',
			game_versions: versionData.game_versions ?? [],
			version_type: versionData.version_type ?? 'release',
			loaders: versionData.loaders ?? [],
			dependencies: versionData.dependencies ?? [],
			existing_files: versionData.files ?? [],
			environment: versionData.environment,
			mrpack_loaders: versionData.mrpack_loaders,
		}

		openCreateVersionModal(draftVersionData, stageId)
	} catch (err: any) {
		if (request !== editRequest) return
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

function openCreateVersionModal(
	version: Labrinth.Versions.v3.DraftVersion | null = null,
	stageId: string | null = null,
) {
	newDraftVersion(projectV2.value.id, version)
	modal.value?.setStage(stageId ?? 0)
	modal.value?.show()
	modalOpen.value = true
}

async function handleDropArea(files: FileList) {
	newDraftVersion(projectV2.value.id, null)
	modal.value?.setStage(0)
	await handleNewFiles(Array.from(files))
	modal.value?.show()
	modalOpen.value = true
}

defineExpose({
	openEditVersionModal,
	openCreateVersionModal,
})
</script>
