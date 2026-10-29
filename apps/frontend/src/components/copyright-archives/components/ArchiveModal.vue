<script setup lang="ts">
import { NewModal } from '@modrinth/ui'
import { ref } from 'vue'

import type { ArchiveInput } from '../types'
import ArchiveJsonForm from './ArchiveJsonForm.vue'

defineProps<{
	persist: (entries: ArchiveInput[]) => Promise<boolean>
	saving: boolean
	saveError: string
}>()
const modal = ref<InstanceType<typeof NewModal>>()
const json = ref('')
const formKey = ref(0)

function show() {
	json.value = JSON.stringify({ name: '', license: '', files: [] }, null, '\t')
	formKey.value++
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		header="New copyright archive"
		width="860px"
		max-width="calc(100vw - 2rem)"
		:close-on-click-outside="false"
		:disable-close="saving"
	>
		<ArchiveJsonForm
			:key="formKey"
			v-model="json"
			:persist="persist"
			:saving="saving"
			:save-error="saveError"
			@saved="modal?.hide()"
			@cancel="modal?.hide()"
		/>
	</NewModal>
</template>
