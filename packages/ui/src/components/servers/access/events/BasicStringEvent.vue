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
		defaultMessage: 'Server created',
	},
	serverReallocated: {
		id: 'servers.audit-log.event.server-reallocated',
		defaultMessage: 'Server reallocated',
	},
	modpackUnlinked: {
		id: 'servers.audit-log.event.modpack-unlinked',
		defaultMessage: 'Modpack unlinked',
	},
	serverRepaired: {
		id: 'servers.audit-log.event.server-repaired',
		defaultMessage: 'Server repaired',
	},
	serverReset: {
		id: 'servers.audit-log.event.server-reset',
		defaultMessage: 'Server reset',
	},
	serverStarted: {
		id: 'servers.audit-log.event.server-started',
		defaultMessage: 'Server started',
	},
	serverStopped: {
		id: 'servers.audit-log.event.server-stopped',
		defaultMessage: 'Server stopped',
	},
	serverRestarted: {
		id: 'servers.audit-log.event.server-restarted',
		defaultMessage: 'Server restarted',
	},
	serverKilled: {
		id: 'servers.audit-log.event.server-killed',
		defaultMessage: 'Server killed',
	},
	sftpLogin: {
		id: 'servers.audit-log.event.sftp-login',
		defaultMessage: 'SFTP login',
	},
	consoleCleared: {
		id: 'servers.audit-log.event.console-cleared',
		defaultMessage: 'Console cleared',
	},
	unknown: {
		id: 'servers.audit-log.event.unknown-basic',
		defaultMessage: 'Server activity',
	},
})

const actionMessages: Record<string, MessageDescriptor> = {
	server_created: messages.serverCreated,
	server_reallocated: messages.serverReallocated,
	modpack_unlinked: messages.modpackUnlinked,
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
