<template>
	<section
		class="workbench-panel h-full min-h-0 w-full"
		:class="[panel?.scroll === false ? '' : 'overflow-auto', { 'p-3': panel?.padded }]"
		:inert="regionHidden"
		:aria-label="panel?.title"
	>
		<LinkedPanels v-if="panel?.members" :tab="panel" />
		<SlotOutlet v-else-if="workbench.slots[slotName]" :slots="workbench.slots" :name="slotName" />
	</section>
</template>

<script setup lang="ts">
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onBeforeUnmount, ref, shallowRef } from 'vue'

import type { WorkbenchPanelParams } from '../../composables/create-workbench'
import { injectWorkbench, provideWorkbenchPanel } from '../../providers/workbench'
import { WORKBENCH_REGIONS, type WorkbenchRegion } from '../../types'
import { SlotOutlet } from '../slot-outlet'
import LinkedPanels from './linked-panels.vue'

const props = defineProps<{
	params: {
		params: WorkbenchPanelParams
		api: DockviewPanelApi
	}
}>()

const workbench = injectWorkbench()
const id = props.params.params.id

const panel = computed(() => workbench.panels.value.find((candidate) => candidate.id === id))
const slotName = `panel-${id}` as const

const host = computed(() => (panel.value ? workbench.regionOf(panel.value) : 'center'))
const regionHidden = computed(
	() =>
		(WORKBENCH_REGIONS as readonly string[]).includes(host.value) &&
		(!workbench.visible[host.value as WorkbenchRegion].value ||
			workbench.collapsed[host.value as WorkbenchRegion].value),
)

const tabVisible = ref(props.params.api.isVisible)
const visibility = props.params.api.onDidVisibilityChange(
	(event) => (tabVisible.value = event.isVisible),
)
const ownerWindow = shallowRef(props.params.api.getWindow())
const location = props.params.api.onDidLocationChange(
	() => (ownerWindow.value = props.params.api.getWindow()),
)
onBeforeUnmount(() => {
	visibility.dispose()
	location.dispose()
})

provideWorkbenchPanel({
	id,
	visible: computed(() => tabVisible.value && !regionHidden.value),
	window: ownerWindow,
})
</script>
