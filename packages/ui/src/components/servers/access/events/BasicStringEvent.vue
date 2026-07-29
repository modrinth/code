<template>
	<BaseEvent>
		{{ formatMessage(message) }}
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'

const props = defineProps<{
	action: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	serverCreated: {
		id: 'servers.audit-log.event.server-created',
		defaultMessage: 'Created server',
	},
	serverReallocated: {
		id: 'servers.audit-log.event.server-reallocated',
		defaultMessage: 'Reallocated server',
	},
	serverRepaired: {
		id: 'servers.audit-log.event.server-repaired',
		defaultMessage: 'Repaired server',
	},
	serverReset: {
		id: 'servers.audit-log.event.server-reset',
		defaultMessage: 'Reset server',
	},
	serverStarted: {
		id: 'servers.audit-log.event.server-started',
		defaultMessage: 'Started server',
	},
	serverStopped: {
		id: 'servers.audit-log.event.server-stopped',
		defaultMessage: 'Stopped server',
	},
	serverRestarted: {
		id: 'servers.audit-log.event.server-restarted',
		defaultMessage: 'Restarted server',
	},
	serverKilled: {
		id: 'servers.audit-log.event.server-killed',
		defaultMessage: 'Killed server',
	},
	sftpLogin: {
		id: 'servers.audit-log.event.sftp-login',
		defaultMessage: 'Logged in via SFTP',
	},
	consoleCleared: {
		id: 'servers.audit-log.event.console-cleared',
		defaultMessage: 'Cleared console',
	},
	unknown: {
		id: 'servers.audit-log.event.unknown-basic',
		defaultMessage: 'Recorded server activity',
	},
})

const actionMessages: Record<string, MessageDescriptor> = {
	server_created: messages.serverCreated,
	server_reallocated: messages.serverReallocated,
	server_repaired: messages.serverRepaired,
	server_reset: messages.serverReset,
	server_started: messages.serverStarted,
	server_stopped: messages.serverStopped,
	server_restarted: messages.serverRestarted,
	server_killed: messages.serverKilled,
	sftp_login: messages.sftpLogin,
	console_cleared: messages.consoleCleared,
}

const message = computed(() => actionMessages[props.action] ?? messages.unknown)
</script>
