<template>
	<BaseEvent>
		{{ formatMessage(messages.sftpLogin) }}
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	sftpLogin: {
		id: 'servers.audit-log.event.sftp-login',
		defaultMessage: 'Logged in via SFTP',
	},
})
</script>
