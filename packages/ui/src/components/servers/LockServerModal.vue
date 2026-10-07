<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { name: serverName })"
		:closable="!isLocking"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="critical" :header="formatMessage(messages.warningHeader)">
				{{ formatMessage(messages.warningBody, { name: serverName }) }}
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="lock-server-reason">
					{{ formatMessage(messages.reasonLabel) }}
				</label>
				<Textarea
					id="lock-server-reason"
					v-model="reason"
					:placeholder="formatMessage(messages.reasonPlaceholder)"
					:disabled="isLocking"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isLocking" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					native-type="button"
					:disabled="!reason.trim() || isLocking"
					@click="lock"
				>
					<SpinnerIcon v-if="isLocking" class="animate-spin" />
					<LockIcon v-else />
					{{ formatMessage(messages.lockButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LockIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Textarea from '#ui/components/base/inputs/Textarea.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

const props = defineProps<{
	serverId: string
	serverName: string
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'servers.lock.modal.header',
		defaultMessage: 'Lock {name}',
	},
	warningHeader: {
		id: 'servers.lock.modal.warning-header',
		defaultMessage: 'This server will become read-only',
	},
	warningBody: {
		id: 'servers.lock.modal.warning-body',
		defaultMessage:
			'{name} will be read-only for its owner and members until it is unlocked by an admin.',
	},
	reasonLabel: {
		id: 'servers.lock.modal.reason-label',
		defaultMessage: 'Reason',
	},
	reasonPlaceholder: {
		id: 'servers.lock.modal.reason-placeholder',
		defaultMessage: 'Visible to other admins',
	},
	lockButton: {
		id: 'servers.lock.modal.lock-button',
		defaultMessage: 'Lock server',
	},
	successTitle: {
		id: 'servers.lock.modal.success-title',
		defaultMessage: 'Server locked',
	},
	successText: {
		id: 'servers.lock.modal.success-text',
		defaultMessage: '{name} has been locked.',
	},
	errorTitle: {
		id: 'servers.lock.modal.error-title',
		defaultMessage: 'Failed to lock server',
	},
	errorText: {
		id: 'servers.lock.modal.error-text',
		defaultMessage: 'An error occurred while locking this server. Please try again.',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const reason = ref('')
const isLocking = ref(false)

function show(): void {
	reason.value = ''
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function lock(): Promise<void> {
	if (!reason.value.trim() || isLocking.value) return

	isLocking.value = true
	try {
		await client.archon.servers_internal.lock(props.serverId, { reason: reason.value.trim() })
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'locks'] }),
		])
		hide()
		addNotification({
			type: 'success',
			title: formatMessage(messages.successTitle),
			text: formatMessage(messages.successText, { name: props.serverName }),
		})
	} catch {
		addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: formatMessage(messages.errorText),
		})
	} finally {
		isLocking.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
