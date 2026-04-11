<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" :closable="false">
		<div class="flex flex-col gap-4 md:w-[400px]">
			<AppearingProgressBar :max-value="totalBytes" :current-value="uploadedBytes" />
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.warningText) }}</p>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { UploadHandle } from '@modrinth/api-client'
import { ref } from 'vue'

import { AppearingProgressBar } from '#ui/components/base'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import NewModal from './NewModal.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'servers.setup.uploading-modpack.header',
		defaultMessage: 'Uploading modpack',
	},
	warningText: {
		id: 'servers.setup.upload-warning',
		defaultMessage: "Please don't close this page while uploading.",
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const uploadedBytes = ref(0)
const totalBytes = ref(0)

function track<T>(handle: UploadHandle<T>): Promise<T> {
	uploadedBytes.value = 0
	totalBytes.value = 0
	modal.value?.show()
	handle.onProgress(({ loaded, total }) => {
		uploadedBytes.value = loaded
		totalBytes.value = total
	})
	return handle.promise.finally(() => {
		modal.value?.hide()
	})
}

defineExpose({ track })
</script>
