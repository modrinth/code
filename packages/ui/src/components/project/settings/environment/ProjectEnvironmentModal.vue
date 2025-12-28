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
import { onMounted, useTemplateRef } from 'vue'
import { useRoute } from 'vue-router'

import { NewModal } from '../../../modal'
import EnvironmentMigration from './EnvironmentMigration.vue'

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

onMounted(() => {
	const route = useRoute()
	if (route.query.showEnvironmentMigrationWarning === 'true') {
		show()
	}
})

defineExpose({
	show,
	hide,
})
</script>
