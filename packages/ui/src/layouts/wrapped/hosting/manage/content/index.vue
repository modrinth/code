<script setup lang="ts">
import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import UnknownFileWarningModal from '#ui/components/modal/UnknownFileWarningModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import ManagedContentModal from '#ui/layouts/shared/content-tab/components/ManagedContentModal.vue'
import ConfirmEnvironmentModal from '#ui/layouts/shared/content-tab/components/modals/ConfirmEnvironmentModal.vue'
import ContentPageLayout from '#ui/layouts/shared/content-tab/layout.vue'
import { provideContentManager } from '#ui/layouts/shared/content-tab/providers/content-manager'

import { useServerContentManager } from './use-server-content-manager'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modpackContent: {
		id: 'hosting.content.managed-content.modpack-header',
		defaultMessage: 'Modpack content',
	},
})

const { context, contentReadyPending, busy, upload, modpack, environmentWarningModal } =
	useServerContentManager()

const { actionDisabled, contentBusyMessage } = busy
const {
	unknownFileWarningModal,
	unknownFileName,
	resolveUnknownFileWarning,
	handleUnknownFileContinue,
} = upload
const {
	managedContent,
	modpackContentModal,
	isModpackContentModalOpen,
	toggleItem,
	setItemEnabledFor,
} = modpack

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
				<ManagedContentModal
					ref="modpackContentModal"
					:source-name="managedContent?.card.manager.name"
					:source-icon-url="managedContent?.card.manager.iconUrl"
					:header="formatMessage(messages.modpackContent)"
					enable-enabled-for
					show-environment-warnings
					:action-disabled="actionDisabled"
					:action-disabled-tooltip="contentBusyMessage ?? undefined"
					@update:enabled="toggleItem"
					@update:enabled-for="setItemEnabledFor"
					@hide="isModpackContentModalOpen = false"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
	<ConfirmEnvironmentModal ref="environmentWarningModal" :action-disabled="actionDisabled" />
</template>
