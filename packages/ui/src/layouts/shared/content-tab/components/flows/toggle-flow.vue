<script setup lang="ts">
import { injectContentManager } from '../../providers/content-manager'
import ConfirmDisableModal from '../modals/ConfirmDisableModal.vue'
import type { ContentFlowState } from './types'
import { useToggleFlow } from './use-toggle-flow'

const props = defineProps<{
	state: ContentFlowState
}>()

const ctx = injectContentManager()

const {
	confirmDisableModal,
	pendingDisableItems,
	pendingDisableWarning,
	canToggle,
	toggle,
	disableItems,
	enableItems,
	disableEnabledItems,
	confirmDisable,
} = useToggleFlow(ctx, props.state)

defineExpose({ canToggle, toggle, disableItems, enableItems, disableEnabledItems })
</script>

<template>
	<ConfirmDisableModal
		ref="confirmDisableModal"
		:count="pendingDisableItems.length"
		:item-type="state.itemType.value"
		:warning="pendingDisableWarning"
		:action-disabled="!!ctx.busy.value"
		:action-disabled-tooltip="ctx.busy.value?.message"
		@disable="confirmDisable"
	/>
</template>
