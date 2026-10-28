<template>
	<div class="flex flex-col gap-4">
		<AppearingProgressBar
			:max-value="ctx.uploadProgress.value?.total ?? 0"
			:current-value="ctx.uploadProgress.value?.loaded ?? 0"
		/>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.warningText) }}</p>
	</div>
</template>

<script setup lang="ts">
import { AppearingProgressBar } from '#ui/components/base'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	warningText: {
		id: 'servers.setup.upload-warning',
		defaultMessage: "Please don't close this page while uploading.",
	},
})
</script>
