<template>
	<ReadyTransition :pending="loading">
		<ContentPageLayout :highlighted-item-id="highlightedItemId">
			<template #modals>
				<SyncedContentModal ref="syncedContentModal" />
				<UnknownFileWarningModal
					ref="unknownFileWarningModal"
					mode="mod"
					:file-name="unknownFileName"
					@cancel="resolveUnknownFileWarning(false)"
					@continue="handleUnknownFileContinue"
				/>
				<ShareModalWrapper
					ref="shareModal"
					:share-title="formatMessage(messages.shareTitle)"
					:share-text="formatMessage(messages.shareText)"
					:open-in-new-tab="false"
				/>
				<ManagedContentModal
					ref="managedContentModal"
					:header="managedContentModalHeader"
					:source-name="managed?.card.manager.name"
					:source-icon-url="managed?.card.manager.iconUrl"
					:enable-toggle="!isServerInstance && !isSharedMember && !isContentLocked"
					:action-disabled="isBlocking || isInstanceBusy"
					override-files
					:get-overflow-options="menu"
					:switch-version="
						isServerInstance || isSharedMember || isContentLocked
							? undefined
							: updater.handleSwitchVersion
					"
					@update:enabled="toggleManagedItem"
					@bulk:enable="(items) => toggleManagedItems(items, true)"
					@bulk:disable="(items) => toggleManagedItems(items, false)"
				/>
				<ConfirmDisableModal
					ref="managedDisableConfirmModal"
					:count="pendingManagedDisableItems.length"
					:item-type="formatMessage(messages.contentTypeProject)"
					:warning="policy.disableWarning(pendingManagedDisableItems)"
					:action-disabled="isInstanceBusy"
					@disable="confirmManagedDisable"
				/>
				<ConfirmModpackUpdateModal
					ref="modpackUpdateConfirmModal"
					:downgrade="isModpackUpdateDowngrade"
					:backup-tip="
						[displayedModpackProject?.title, pendingModpackUpdateVersion?.version_number]
							.filter(Boolean)
							.join(' ')
					"
					@confirm="updater.handleModpackUpdateConfirm"
					@cancel="updater.handleModpackUpdateCancel"
				/>
				<ExportModal
					v-if="installedItems.length > 0 && !instance.quarantined"
					ref="exportModal"
					:instance="instance"
				/>
				<ContentUpdaterModal
					v-if="updatingProject || updatingModpack"
					ref="contentUpdaterModal"
					:versions="updatingProjectVersions"
					:current-game-version="instance.game_version"
					:current-loader="instance.loader"
					:current-version-id="
						updatingModpack
							? (instance.link?.version_id ?? '')
							: (updatingProject?.version?.id ?? '')
					"
					:is-app="true"
					:project-type="updatingModpack ? 'modpack' : updatingProject?.project_type"
					:project-icon-url="
						updatingModpack ? displayedModpackProject?.icon_url : updatingProject?.project?.icon_url
					"
					:project-name="
						updatingModpack
							? (displayedModpackProject?.title ?? formatMessage(commonMessages.modpackLabel))
							: (updatingProject?.project?.title ?? updatingProject?.file_name)
					"
					:loading="loadingVersions"
					:loading-changelog="loadingChangelog"
					@update="updater.handleModalUpdate"
					@cancel="updater.resetUpdateState"
					@version-select="updater.handleVersionSelect"
					@version-hover="updater.handleVersionHover"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
</template>

<script setup lang="ts">
import {
	commonMessages,
	ConfirmDisableModal,
	ConfirmModpackUpdateModal,
	ContentPageLayout,
	ContentUpdaterModal,
	defineMessages,
	ManagedContentModal,
	provideContentManager,
	ReadyTransition,
	UnknownFileWarningModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import ExportModal from '@/components/ui/ExportModal.vue'
import SyncedContentModal from '@/components/ui/instance/SyncedContentModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { useInstanceContentManager } from '@/features/instances/content/use-instance-content-manager'

const messages = defineMessages({
	shareTitle: {
		id: 'app.instance.mods.share-title',
		defaultMessage: 'Sharing modpack content',
	},
	shareText: {
		id: 'app.instance.mods.share-text',
		defaultMessage: "Check out the projects I'm using in my modpack!",
	},
	contentTypeProject: {
		id: 'app.instance.mods.content-type-project',
		defaultMessage: 'project',
	},
})

const { formatMessage } = useVIntl()

const {
	context,
	instance,
	installedItems,
	loading,
	highlightedItemId,
	isInstanceBusy,
	isContentLocked,
	isSharedMember,
	isServerInstance,
	isBlocking,
	policy,
	syncedContentModal,
	shareModal,
	managedContentModal,
	managedState,
	updater,
	upload,
	menu,
} = useInstanceContentManager()

provideContentManager(context)

const {
	managedDisableConfirmModal,
	pendingManagedDisableItems,
	managed,
	managedContentModalHeader,
	displayedModpackProject,
	toggleManagedItem,
	toggleManagedItems,
	confirmManagedDisable,
} = managedState

const {
	contentUpdaterModal,
	modpackUpdateConfirmModal,
	updatingProject,
	updatingProjectVersions,
	loadingVersions,
	loadingChangelog,
	updatingModpack,
	pendingModpackUpdateVersion,
	isModpackUpdateDowngrade,
} = updater

const {
	unknownFileWarningModal,
	unknownFileName,
	resolveUnknownFileWarning,
	handleUnknownFileContinue,
} = upload

const exportModal = ref(null)
</script>
