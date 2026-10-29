<template>
	<NewModal ref="modal" :header="formatMessage(serverPlayMigrationMessages.title)">
		<ServerPlayMigrationContent
			:key="openCount"
			:modpack-name="modpackName"
			:disabled="disabled"
			cancel
			@back="hide"
			@confirm="emit('confirm')"
		/>
	</NewModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables/i18n'

import { serverPlayMigrationMessages } from './messages'
import ServerPlayMigrationContent from './ServerPlayMigrationContent.vue'

defineProps<{
	modpackName?: string | null
	disabled?: boolean
}>()

const emit = defineEmits<{
	confirm: []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const openCount = ref(0)

function show() {
	openCount.value++
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
