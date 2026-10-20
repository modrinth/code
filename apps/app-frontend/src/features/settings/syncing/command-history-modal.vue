<script setup lang="ts">
import { SaveIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	CodeEditor,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { nextTick, ref } from 'vue'

import { set_command_history } from '@/helpers/instance'
import { commandHistoryQueryOptions, syncedOptionsKeys } from '@/helpers/synced-options'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const commandHistory = ref('')
const historyQuery = useQuery({ ...commandHistoryQueryOptions(), enabled: false })
const saveMutation = useMutation({
	mutationFn: set_command_history,
	onSuccess: (history) => {
		queryClient.setQueryData(syncedOptionsKeys.commandHistory, history)
	},
	onError: handleError,
})

const messages = defineMessages({
	commandHistoryEditorTitle: {
		id: 'app.settings.synced-options.command-history.editor-title',
		defaultMessage: 'Edit command history',
	},
})

async function show() {
	const result = await historyQuery.refetch()
	if (result.isError) {
		handleError(result.error)
		return
	}
	if (!result.isSuccess) return
	commandHistory.value = result.data
	modal.value?.show()
}

async function saveCommandHistory() {
	if (saveMutation.isPending.value) return
	try {
		await saveMutation.mutateAsync(commandHistory.value)
		await nextTick()
		modal.value?.hide()
	} catch {
		return
	}
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.commandHistoryEditorTitle)"
		:disable-close="saveMutation.isPending.value"
		no-padding
		actions-divider
		max-width="700px"
		width="700px"
	>
		<CodeEditor
			v-model="commandHistory"
			language="mcfunction"
			class="h-[420px] text-sm"
			@save="saveCommandHistory"
		/>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" :loading="saveMutation.isPending.value" @click="saveCommandHistory">
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.saveButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
