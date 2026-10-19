<template>
	<div class="flex h-full items-center">
		<component
			:is="workbench.documents.headerActions"
			v-if="workbench.documents?.headerActions"
			:params="params"
		/>
		<AddMenu v-if="inGrid" :group="params.group" />
		<CornerToggle v-if="cornerToggles" side="right" :params="params" />
	</div>
</template>

<script setup lang="ts">
import type { DockviewGroupLocation, DockviewGroupPanel, IDockviewPanel } from 'dockview-vue'
import { computed } from 'vue'

import { injectWorkbench, injectWorkbenchDock } from '../../providers/workbench'
import AddMenu from './add-menu.vue'
import CornerToggle from './corner-toggle.vue'

defineOptions({ inheritAttrs: false })

/**
 * The end of a group's tab bar: the documents' own actions, the add menu, then the right region's
 * corner toggle. Groups popped out into windows only keep the documents' actions.
 */
const props = defineProps<{
	params: { group: DockviewGroupPanel; panels: IDockviewPanel[]; location?: DockviewGroupLocation }
}>()

const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const inGrid = computed(
	() => (props.params.location ?? props.params.group.api.location).type === 'grid',
)

const cornerToggles = computed(
	() => workbench.cornerToggles.value && host === 'center' && inGrid.value,
)
</script>
