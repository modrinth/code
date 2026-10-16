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
		v-if="!modalOpen && !isVersionCreateRestricted"
		:accept="acceptFileFromProjectType(projectV2.project_type)"
		@change="handleDropArea"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	defineMessages,
	DropArea,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	isScopeRemovedForUser,
	MultiStageModal,
	useVIntl,
} from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createManageVersionContext,
	provideManageVersionContext,
} from '~/providers/version/manage-version-modal'

const emit = defineEmits<{
	(e: 'save'): void
}>()

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')
const modalOpen = ref(false)

const ctx = createManageVersionContext(modal, () => emit('save'))
provideManageVersionContext(ctx)

const { newDraftVersion, editingVersion, handleNewFiles } = ctx

const { projectV2 } = injectProjectPageContext()
const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()
const auth = injectAuth()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	restrictedTitle: {
		id: 'create.version.restricted-title',
		defaultMessage: 'Action restricted',
	},
	restrictedText: {
		id: 'create.version.restricted-text',
		defaultMessage:
			'A moderator has removed this permission from your account. See Account standing in your account settings for details.',
	},
})

const isVersionCreateRestricted = computed(() =>
	isScopeRemovedForUser(auth.user.value, 'VERSION_CREATE'),
)

function rejectIfRestricted(scopeId: string): boolean {
	if (!isScopeRemovedForUser(auth.user.value, scopeId)) return false
	addNotification({
		title: formatMessage(messages.restrictedTitle),
		text: formatMessage(messages.restrictedText),
		type: 'error',
	})
	return true
}

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
	if (rejectIfRestricted(version?.version_id ? 'VERSION_WRITE' : 'VERSION_CREATE')) return
	newDraftVersion(projectV2.value.id, version)
	modal.value?.setStage(stageId ?? 0)
	modal.value?.show()
	modalOpen.value = true
}

async function handleDropArea(files: FileList) {
	if (rejectIfRestricted('VERSION_CREATE')) return
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
