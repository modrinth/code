<template>
	<NewModal ref="modal" :scrollable="true" max-content-height="82vh" :closable="true">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Edit project Environment</span>
		</template>
		<div class="mb-24 max-w-[600px]">
			<EnvironmentMigration />
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'

import { NewModal } from '../../../modal'
import EnvironmentMigration from './EnvironmentMigration.vue'

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>
