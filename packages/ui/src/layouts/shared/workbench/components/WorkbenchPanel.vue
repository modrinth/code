<template>
	<div class="h-full min-h-0 w-full overflow-auto bg-surface-2">
		<component :is="content" v-if="content" />
	</div>
</template>

<script setup lang="ts">
import type { DockviewPanelApi } from 'dockview-vue'
import { computed } from 'vue'

import type { WorkbenchPanelParams } from '../composables/workbench-layout'
import { injectWorkbench } from '../providers/workbench'

const props = defineProps<{
	params: {
		params: WorkbenchPanelParams
		api: DockviewPanelApi
	}
}>()

const workbench = injectWorkbench()

const content = computed(() => workbench.slots[`panel-${props.params.params.id}`])
</script>
