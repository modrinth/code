<template>
	<NewModal ref="modal" :scrollable="true" max-content-height="72vh" :closable="true">
		<EnvironmentMigration />
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
