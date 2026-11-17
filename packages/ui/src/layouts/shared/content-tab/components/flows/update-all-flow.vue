<script setup lang="ts">
import UpdateAllModal from '#ui/components/modal/update-all-modal/index.vue'

import { injectContentManager } from '../../providers/content-manager'
import type { ContentItem } from '../../types'
import type { ContentFlowState } from './types'
import { useUpdateAllFlow } from './use-update-all-flow'

const props = defineProps<{
	state: ContentFlowState
	updateItem: (item: ContentItem) => void
}>()

const ctx = injectContentManager()

const {
	updateAllModal,
	updateAllItems,
	loadingUpdateAll,
	loadingUpdateAllChangelog,
	bulkItemCount,
	bulkUpdateItems,
	hasBulkUpdateSupport,
	loadUpdateAllChangelog,
	preloadUpdateAllChangelog,
	updateAllSelected,
	promptUpdate,
	cancelUpdateAll,
} = useUpdateAllFlow(ctx, props.state, (item) => props.updateItem(item))

defineExpose({
	loading: loadingUpdateAll,
	bulkItemCount,
	bulkUpdateItems,
	supported: hasBulkUpdateSupport,
	promptUpdate,
})
</script>

<template>
	<UpdateAllModal
		v-if="hasBulkUpdateSupport"
		ref="updateAllModal"
		:items="updateAllItems"
		:server="ctx.target.value.kind === 'server'"
		:loading="loadingUpdateAll"
		:loading-changelog="loadingUpdateAllChangelog"
		:action-disabled="!!ctx.busy.value"
		@changelog="loadUpdateAllChangelog"
		@preload-changelog="preloadUpdateAllChangelog"
		@cancel="cancelUpdateAll"
		@update="updateAllSelected"
	/>
</template>
