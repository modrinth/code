<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" />
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiStageModal,
} from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createManageVersionContext,
	provideManageVersionContext,
} from '~/providers/version/manage-version-modal'

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createManageVersionContext(modal)
provideManageVersionContext(ctx)

const { newDraftVersion } = ctx

const { projectV2 } = injectProjectPageContext()
const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

async function openEditVersionModal(versionId: string, projectId: string, stageId?: string | null) {
	try {
		const versionData = await labrinth.versions_v3.getVersion(versionId)

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
}

defineExpose({
	openEditVersionModal,
	openCreateVersionModal,
})
</script>
