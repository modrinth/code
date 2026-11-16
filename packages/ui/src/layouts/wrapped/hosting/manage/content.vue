<script setup lang="ts">
import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import UnknownFileWarningModal from '#ui/components/modal/UnknownFileWarningModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import ManagedContentModal from '../../../shared/content-tab/components/managed-content-modal/index.vue'
import ConfirmEnvironmentModal from '../../../shared/content-tab/components/modals/ConfirmEnvironmentModal.vue'
import ConfirmModpackUpdateModal from '../../../shared/content-tab/components/modals/ConfirmModpackUpdateModal.vue'
import ConfirmUnlinkModal from '../../../shared/content-tab/components/modals/ConfirmUnlinkModal.vue'
import ContentUpdaterModal from '../../../shared/content-tab/components/modals/content-updater-modal/index.vue'
import ContentPageLayout from '../../../shared/content-tab/layout.vue'
import { provideContentManager } from '../../../shared/content-tab/providers/content-manager'
import { useServerContentManager } from './composables/use-server-content-manager'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modpackContent: {
		id: 'hosting.content.managed-content.modpack-header',
		defaultMessage: 'Modpack content',
	},
})

const {
	context,
	contentReadyPending,
	managedContent,
	unknownFileWarningModal,
	unknownFileName,
	resolveUnknownFileWarning,
	handleUnknownFileContinue,
	modpackUnlinkModal,
	handleModpackUnlinkConfirm,
	modpackContentModal,
	isModpackContentModalOpen,
	handleModpackToggleEnabled,
	handleModpackSetEnabledFor,
	contentUpdaterModal,
	updatingProject,
	updatingModpack,
	updatingProjectVersions,
	currentGameVersion,
	currentLoader,
	currentModpackVersionId,
	loadingVersions,
	loadingChangelog,
	handleModalUpdate,
	resetUpdateState,
	handleVersionSelect,
	handleVersionHover,
	environmentWarningModal,
	modpackUpdateModal,
	isModpackUpdateDowngrade,
	pendingModpackUpdateVersion,
	handleModpackUpdateConfirm,
	handleModpackUpdateCancel,
	setupActionDisabled,
	setupActionBusyMessage,
	contentActionDisabled,
	contentActionBusyMessage,
} = useServerContentManager()

provideContentManager(context)
</script>

<template>
	<ReadyTransition :pending="contentReadyPending">
		<ContentPageLayout :bottom-padding="false">
			<template #modals>
				<UnknownFileWarningModal
					ref="unknownFileWarningModal"
					mode="mod"
					:file-name="unknownFileName"
					@cancel="resolveUnknownFileWarning(false)"
					@continue="handleUnknownFileContinue"
				/>
				<ConfirmUnlinkModal
					ref="modpackUnlinkModal"
					server
					:action-disabled="setupActionDisabled"
					:action-disabled-tooltip="setupActionBusyMessage ?? undefined"
					@unlink="handleModpackUnlinkConfirm"
				/>
				<ManagedContentModal
					ref="modpackContentModal"
					:source-name="managedContent?.card.manager.name"
					:source-icon-url="managedContent?.card.manager.iconUrl"
					:header="formatMessage(messages.modpackContent)"
					enable-enabled-for
					show-environment-warnings
					:action-disabled="contentActionDisabled"
					:action-disabled-tooltip="contentActionBusyMessage ?? undefined"
					@update:enabled="handleModpackToggleEnabled"
					@update:enabled-for="handleModpackSetEnabledFor"
					@hide="isModpackContentModalOpen = false"
				/>
				<ContentUpdaterModal
					v-if="updatingProject || updatingModpack"
					ref="contentUpdaterModal"
					:versions="updatingProjectVersions"
					:current-game-version="currentGameVersion"
					:current-loader="currentLoader"
					:current-version-id="
						updatingModpack ? (currentModpackVersionId ?? '') : (updatingProject?.version?.id ?? '')
					"
					:is-app="false"
					:project-type="updatingModpack ? 'modpack' : updatingProject?.project_type"
					:project-icon-url="
						updatingModpack
							? managedContent?.card.manager.iconUrl
							: updatingProject?.project?.icon_url
					"
					:project-name="
						updatingModpack
							? (managedContent?.card.manager.name ?? formatMessage(commonMessages.modpackLabel))
							: (updatingProject?.project?.title ?? updatingProject?.file_name)
					"
					:loading="loadingVersions"
					:loading-changelog="loadingChangelog"
					:action-disabled="updatingModpack ? setupActionDisabled : contentActionDisabled"
					:action-disabled-tooltip="
						(updatingModpack ? setupActionBusyMessage : contentActionBusyMessage) ?? undefined
					"
					@update="handleModalUpdate"
					@cancel="resetUpdateState"
					@version-select="handleVersionSelect"
					@version-hover="handleVersionHover"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
	<ConfirmEnvironmentModal ref="environmentWarningModal" :action-disabled="contentActionDisabled" />
	<ConfirmModpackUpdateModal
		ref="modpackUpdateModal"
		:downgrade="isModpackUpdateDowngrade"
		:backup-tip="
			[managedContent?.card.manager.name, pendingModpackUpdateVersion?.version_number]
				.filter(Boolean)
				.join(' ')
		"
		server
		:action-disabled="setupActionDisabled"
		:action-disabled-tooltip="setupActionBusyMessage ?? undefined"
		@confirm="handleModpackUpdateConfirm"
		@cancel="handleModpackUpdateCancel"
	/>
</template>
