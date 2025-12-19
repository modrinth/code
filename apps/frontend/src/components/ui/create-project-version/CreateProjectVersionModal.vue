<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" />
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { injectProjectPageContext, MultiStageModal } from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createManageVersionContext,
	provideManageVersionContext,
} from '~/providers/version/manage-version-modal'

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createManageVersionContext(modal)
provideManageVersionContext(ctx)

const { newDraftVersion, setProjectType } = ctx

const { projectV2 } = injectProjectPageContext()

function showCreateVersionModal(
	version: Labrinth.Versions.v3.DraftVersion | null = null,
	stageId: string | null = null,
) {
	newDraftVersion(projectV2.value.id, version)
	setProjectType(projectV2.value)
	modal.value?.setStage(stageId ?? 0)
	modal.value?.show()
}

defineExpose({
	show: showCreateVersionModal,
})
</script>
