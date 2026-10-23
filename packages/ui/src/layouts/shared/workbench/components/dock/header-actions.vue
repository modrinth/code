<template>
	<div class="flex h-full items-center">
		<component
			:is="workbench.documents.headerActions"
			v-if="workbench.documents?.headerActions"
			:params="params"
		/>
		<AddMenu v-if="addMenuShown" :group="params.group" />
		<CornerToggle v-if="inGrid && host === 'center'" side="right" :params="params" />
		<CornerToggle v-if="inGrid && host === 'bottom'" side="bottom" :params="params" />
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
 * The end of a group's tab bar: the documents' own actions, the add menu (as the settings show it),
 * then the toggle of the right region (in the center's top-right group) or of the bottom region
 * (in its own top-right group). Groups popped out into windows keep the documents' actions and
 * the add menu.
 */
const props = defineProps<{
	params: { group: DockviewGroupPanel; panels: IDockviewPanel[]; location?: DockviewGroupLocation }
}>()

const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const inGrid = computed(
	() => (props.params.location ?? props.params.group.api.location).type === 'grid',
)

const addMenuShown = computed(() => workbench.documents?.addMenuShown?.() ?? true)
</script>
