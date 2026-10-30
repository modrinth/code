<template>
	<NewModal ref="modal" :header="formatMessage(serverPlayMigrationMessages.title)">
		<ServerPlayMigrationFlow
			:key="openCount"
			:kind="kind"
			:modpack-name="modpackName"
			:has-uploaded-mrpack="hasUploadedMrpack"
			:world-id="worldId"
			:disabled="disabled"
			cancel
			@back="hide"
			@confirm="emit('confirm', $event)"
		/>
	</NewModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables/i18n'

import { serverPlayMigrationMessages } from './messages'
import ServerPlayMigrationFlow from './ServerPlayMigrationFlow.vue'
import type { ServerPlayMigrationKind, ServerPlayMigrationSource } from './types'

defineProps<{
	kind: ServerPlayMigrationKind | null
	modpackName?: string | null
	hasUploadedMrpack?: boolean
	worldId: string | null
	disabled?: boolean
}>()

const emit = defineEmits<{
	confirm: [source: ServerPlayMigrationSource]
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
