<script setup lang="ts">
import { computed } from 'vue'

import { injectContentManager } from '../../providers/content-manager'
import type { ContentCardTableItem, ContentItem } from '../../types'
import ConfirmDeletionModal from '../modals/ConfirmDeletionModal.vue'
import ContentDependencyWarningModal from '../modals/ContentDependencyWarningModal.vue'
import type { ContentFlowState } from './types'
import { useDeleteFlow } from './use-delete-flow'

const props = defineProps<{
	state: ContentFlowState
	toDisplayRow: (item: ContentItem) => ContentCardTableItem
	disableItems: (items: ContentItem[]) => Promise<void>
}>()

const ctx = injectContentManager()

const {
	confirmDeletionModal,
	dependencyWarningModal,
	pendingItems,
	pendingWarning,
	dependencyItems,
	dependencyDependents,
	canRemove,
	promptDelete,
	confirmDelete,
	confirmDependencyWarningDelete,
} = useDeleteFlow(ctx, props.state, {
	toDisplayRow: (item) => props.toDisplayRow(item),
	disableItems: (items) => props.disableItems(items),
})

const variant = computed(() => ctx.target.value.kind)
const backupTip = computed(() =>
	pendingItems.value.map((item) => item.project?.title ?? item.file_name).join(', '),
)

defineExpose({ canRemove, promptDelete })
</script>

<template>
	<ConfirmDeletionModal
		ref="confirmDeletionModal"
		:count="pendingItems.length"
		:item-type="state.itemType.value"
		:warning="pendingWarning"
		:variant="variant"
		:backup-tip="backupTip"
		:action-disabled="!!ctx.busy.value"
		:action-disabled-tooltip="ctx.busy.value?.message"
		@delete="confirmDelete"
	/>
	<ContentDependencyWarningModal
		ref="dependencyWarningModal"
		:items="dependencyItems"
		:dependents="dependencyDependents"
		:item-type="state.itemType.value"
		:variant="variant"
		:backup-tip="backupTip"
		:action-disabled="!!ctx.busy.value"
		:action-disabled-tooltip="ctx.busy.value?.message"
		@delete="confirmDependencyWarningDelete"
	/>
</template>
