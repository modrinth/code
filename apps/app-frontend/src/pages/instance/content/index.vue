<template>
	<ReadyTransition :pending="loading">
		<ContentPageLayout :highlighted-item-id="highlightedItemId">
			<template #modals="{ switchVersion }">
				<SyncedContentModal ref="syncedContentModal" />
				<UnknownFileWarningModal
					ref="unknownFileWarningModal"
					mode="mod"
					:file-name="unknownFileName"
					@cancel="resolveUnknownFileWarning(false)"
					@continue="handleUnknownFileContinue"
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
						isServerInstance || isSharedMember || isContentLocked ? undefined : switchVersion
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
				<ExportModal
					v-if="installedItems.length > 0 && !instance.quarantined"
					ref="exportModal"
					:instance="instance"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
</template>

<script setup lang="ts">
import {
	ConfirmDisableModal,
	ContentPageLayout,
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
import { useInstanceContentManager } from '@/features/instances/content/use-instance-content-manager'

const messages = defineMessages({
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
	managedContentModal,
	managedState,
	upload,
	menu,
} = useInstanceContentManager()

provideContentManager(context)

const {
	managedDisableConfirmModal,
	pendingManagedDisableItems,
	managed,
	managedContentModalHeader,
	toggleManagedItem,
	toggleManagedItems,
	confirmManagedDisable,
} = managedState

const {
	unknownFileWarningModal,
	unknownFileName,
	resolveUnknownFileWarning,
	handleUnknownFileContinue,
} = upload

const exportModal = ref(null)
</script>
