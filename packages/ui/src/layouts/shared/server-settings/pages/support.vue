<template>
	<div v-if="isSiteAdmin" class="flex flex-col gap-2.5">
		<Teleport to="body">
			<div class="relative z-[100]">
				<ConfirmModal
					ref="clearActionLogModal"
					:title="formatMessage(messages.clearActionLogTitle)"
					:description="formatMessage(messages.clearActionLogConfirmation)"
					:proceed-label="formatMessage(messages.clearActionLogButton)"
					:markdown="false"
					@proceed="clearActionLog"
				/>
			</div>
		</Teleport>

		<span class="text-lg font-semibold text-contrast">{{
			formatMessage(messages.actionLogTitle)
		}}</span>
		<div>
			<Button
				v-tooltip="clearActionLogTooltip"
				type="colored"
				color="red"
				:disabled="!canResetServer || isClearingActionLog"
				:loading="isClearingActionLog"
				@click="showClearActionLogModal"
			>
				<TrashIcon class="size-5" />
				{{ formatMessage(messages.clearActionLogButton) }}
			</Button>
		</div>
		<span class="text-primary">{{ formatMessage(messages.clearActionLogDescription) }}</span>
	</div>
</template>

<script setup lang="ts">
import { TrashIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { ConfirmModal } from '#ui/components/modal'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { injectServerSettings } from '#ui/layouts/shared/server-settings/providers/server-settings'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const client = injectModrinthClient()
const { serverId } = injectModrinthServerContext()
const { currentUserRole } = injectServerSettings()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const { canResetServer, permissionDeniedMessage } = useServerPermissions()

const messages = defineMessages({
	actionLogTitle: {
		id: 'servers.settings.support.action-log-title',
		defaultMessage: 'Action log',
	},
	clearActionLogButton: {
		id: 'servers.settings.support.clear-action-log-button',
		defaultMessage: 'Clear action log',
	},
	clearActionLogDescription: {
		id: 'servers.settings.support.clear-action-log-description',
		defaultMessage: 'Delete all action log entries for this server.',
	},
	clearActionLogTitle: {
		id: 'servers.settings.support.clear-action-log-title',
		defaultMessage: 'Clear action log?',
	},
	clearActionLogConfirmation: {
		id: 'servers.settings.support.clear-action-log-confirmation',
		defaultMessage: 'This permanently deletes all action log entries for this server. Continue?',
	},
	clearActionLogSuccess: {
		id: 'servers.settings.support.clear-action-log-success',
		defaultMessage: 'Action log cleared',
	},
	clearActionLogError: {
		id: 'servers.settings.support.clear-action-log-error',
		defaultMessage: 'Failed to clear action log',
	},
})

const isSiteAdmin = computed(() => currentUserRole.value === 'admin')
const clearActionLogModal = ref<InstanceType<typeof ConfirmModal>>()
const isClearingActionLog = ref(false)
const clearActionLogTooltip = computed(() =>
	canResetServer.value ? undefined : permissionDeniedMessage.value,
)

function showClearActionLogModal() {
	if (!isSiteAdmin.value || !canResetServer.value || isClearingActionLog.value) return
	clearActionLogModal.value?.show()
}

async function clearActionLog() {
	if (!isSiteAdmin.value || !canResetServer.value || isClearingActionLog.value) return

	try {
		isClearingActionLog.value = true
		await client.archon.actions_v1.clear(serverId)
		await queryClient.invalidateQueries({
			queryKey: ['servers', 'action-log', 'v1', 'infinite', serverId],
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.clearActionLogSuccess),
		})
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.clearActionLogError),
		})
	} finally {
		isClearingActionLog.value = false
	}
}
</script>
