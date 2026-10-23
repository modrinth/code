<template>
	<div
		class="workbench-toolbar flex min-h-10 shrink-0 flex-wrap items-center gap-2 border-0 border-t border-solid border-surface-4 bg-surface-3 px-2 py-1"
	>
		<SlotOutlet v-if="workbench.slots.toolbar" :slots="workbench.slots" name="toolbar" />
		<div class="ml-auto flex items-center gap-1">
			<SlotOutlet
				v-if="workbench.slots['toolbar-end']"
				:slots="workbench.slots"
				name="toolbar-end"
			/>
			<ToolbarControls />
		</div>
	</div>
</template>

<script setup lang="ts">
import { injectWorkbench } from '../providers/workbench'
import { SlotOutlet } from './slot-outlet'
import ToolbarControls from './toolbar-controls.vue'

/**
 * The workbench's toolbar, along the bottom of the center like a review's queue bar: the host's
 * controls, then its end controls and the layout menu.
 */
const workbench = injectWorkbench()
</script>
