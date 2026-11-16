<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { name: serverName })"
		:disable-close="isSuspending"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="critical" :header="formatMessage(messages.warningHeader)">
				{{ formatMessage(messages.warningBody, { name: serverName }) }}
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.reasonLabel) }}
				</span>
				<Chips
					v-model="reason"
					:items="reasonOptions"
					:format-label="reasonLabel"
					:capitalize="false"
					:aria-label="formatMessage(messages.reasonLabel)"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isSuspending" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					native-type="button"
					:disabled="isSuspending"
					@click="suspend"
				>
					<SpinnerIcon v-if="isSuspending" class="animate-spin" />
					<BanIcon v-else />
					{{ formatMessage(messages.suspendButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { BanIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Chips from '#ui/components/base/Chips.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

type Reason = Archon.Servers.Internal.SuspendServerRequest['reason']

const props = defineProps<{
	serverId: string
	serverName: string
}>()

const emit = defineEmits<{
	suspended: []
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'servers.suspend.modal.header',
		defaultMessage: 'Suspend {name}',
	},
	warningHeader: {
		id: 'servers.suspend.modal.warning-header',
		defaultMessage: 'This server will stop running',
	},
	warningBody: {
		id: 'servers.suspend.modal.warning-body',
		defaultMessage:
			"{name} will stop and its subdomain will be unassigned. Its backups will be deleted in 60 days unless it's unsuspended.",
	},
	reasonLabel: {
		id: 'servers.suspend.modal.reason-label',
		defaultMessage: 'Reason',
	},
	reasonModerated: {
		id: 'servers.suspend.modal.reason-moderated',
		defaultMessage: 'Moderation',
	},
	reasonOther: {
		id: 'servers.suspend.modal.reason-other',
		defaultMessage: 'Other',
	},
	suspendButton: {
		id: 'servers.suspend.modal.suspend-button',
		defaultMessage: 'Suspend server',
	},
	successTitle: {
		id: 'servers.suspend.modal.success-title',
		defaultMessage: 'Server suspended',
	},
	successText: {
		id: 'servers.suspend.modal.success-text',
		defaultMessage: '{name} has been suspended.',
	},
	errorTitle: {
		id: 'servers.suspend.modal.error-title',
		defaultMessage: 'Failed to suspend server',
	},
})

const reasonOptions: Reason[] = ['moderated', 'other']

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const reason = ref<Reason>('moderated')
const isSuspending = ref(false)

function reasonLabel(option: Reason) {
	return formatMessage(option === 'moderated' ? messages.reasonModerated : messages.reasonOther)
}

function show(): void {
	reason.value = 'moderated'
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function suspend(): Promise<void> {
	if (isSuspending.value) return

	isSuspending.value = true
	try {
		await client.archon.servers_internal.suspend(props.serverId, { reason: reason.value })
		// The modal can't be hidden until it re-renders with closing enabled. Hide it before refetching, as the new status
		// replaces the page behind it.
		isSuspending.value = false
		await nextTick()
		hide()
		addNotification({
			type: 'success',
			title: formatMessage(messages.successTitle),
			text: formatMessage(messages.successText, { name: props.serverName }),
		})
		emit('suspended')
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] }),
			queryClient.invalidateQueries({
				queryKey: ['servers', 'action-log', 'v1', 'infinite', props.serverId],
			}),
		])
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: error instanceof Error ? error.message : undefined,
		})
	} finally {
		isSuspending.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
