<template>
	<section
		class="workbench-panel h-full min-h-0 w-full"
		:class="[panel?.scroll === false ? 'overflow-hidden' : 'overflow-auto', { 'p-3': panel?.padded }]"
		:inert="regionHidden"
		:aria-label="panel?.title"
	>
		<component :is="content" v-if="content" />
	</section>
</template>

<script setup lang="ts">
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onBeforeUnmount, ref } from 'vue'

import type { WorkbenchPanelParams } from '../composables/workbench-layout'
import { injectWorkbench, provideWorkbenchPanel } from '../providers/workbench'
import { WORKBENCH_REGIONS, type WorkbenchRegion } from '../types'

const props = defineProps<{
	params: {
		params: WorkbenchPanelParams
		api: DockviewPanelApi
	}
}>()

const workbench = injectWorkbench()
const id = props.params.params.id

const panel = computed(() => workbench.panels.value.find((candidate) => candidate.id === id))
const content = computed(() => workbench.slots[`panel-${id}`])

const host = computed(() => (panel.value ? workbench.regionOf(panel.value) : 'center'))
const regionHidden = computed(
	() =>
		(WORKBENCH_REGIONS as readonly string[]).includes(host.value) &&
		!workbench.visible[host.value as WorkbenchRegion].value,
)

const tabVisible = ref(props.params.api.isVisible)
const visibility = props.params.api.onDidVisibilityChange((event) => (tabVisible.value = event.isVisible))
onBeforeUnmount(() => visibility.dispose())

provideWorkbenchPanel({ id, visible: computed(() => tabVisible.value && !regionHidden.value) })
</script>
