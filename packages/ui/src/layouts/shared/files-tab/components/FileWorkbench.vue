<template>
	<WorkbenchLayout
		ref="layout"
		:panels="panels"
		:storage-key="storageKey"
		:sizes="sizes"
		:hidden-regions="hiddenRegions"
		:mode="mode"
		:variant="variant"
		:documents="documents"
	>
		<template v-for="(_, name) in $slots" #[name]="slotProps">
			<slot :name="name" v-bind="slotProps ?? {}" />
		</template>
	</WorkbenchLayout>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import WorkbenchLayout from '#ui/layouts/shared/workbench/layout.vue'
import type {
	WorkbenchMode,
	WorkbenchModeName,
	WorkbenchPanel,
	WorkbenchRegion,
	WorkbenchRegionSizes,
	WorkbenchVariant,
} from '#ui/layouts/shared/workbench/types'

import { useFileDocuments } from '../composables/file-documents'

/**
 * A workbench whose docks also host the files layout's tabs, each showing its own navigation and
 * listing or editor. Placed through the files layout's `workspace` slot, with `workbench` set on
 * it. Takes the same panels and slots as `WorkbenchLayout`.
 */
defineProps<{
	panels: WorkbenchPanel[]
	storageKey: string | null
	sizes?: Partial<WorkbenchRegionSizes>
	hiddenRegions?: WorkbenchRegion[]
	mode?: WorkbenchModeName | WorkbenchMode
	variant?: WorkbenchVariant
}>()

const documents = useFileDocuments()

const layout = ref<InstanceType<typeof WorkbenchLayout>>()

defineExpose({ layout })
</script>
