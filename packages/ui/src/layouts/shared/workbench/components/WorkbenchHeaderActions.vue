<template>
	<div class="flex h-full items-center">
		<component
			:is="workbench.documents.headerActions"
			v-if="workbench.documents?.headerActions"
			:params="params"
		/>
		<WorkbenchCornerToggle v-if="cornerToggles" side="right" :params="params" />
	</div>
</template>

<script setup lang="ts">
import type { DockviewGroupLocation, DockviewGroupPanel, IDockviewPanel } from 'dockview-vue'
import { computed } from 'vue'

import { injectWorkbench, injectWorkbenchDock } from '../providers/workbench'
import WorkbenchCornerToggle from './WorkbenchCornerToggle.vue'

defineOptions({ inheritAttrs: false })

/** The end of a group's tab bar: the documents' own actions, and the right region's corner toggle. */
const props = defineProps<{
	params: { group: DockviewGroupPanel; panels: IDockviewPanel[]; location?: DockviewGroupLocation }
}>()

const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const cornerToggles = computed(
	() =>
		workbench.mode.cornerToggles &&
		host === 'center' &&
		(props.params.location ?? props.params.group.api.location).type === 'grid',
)
</script>
