<script setup lang="ts">
import {
	commonMessages,
	defineMessages,
	injectModrinthServerContext,
	ServersManageInstancesPage,
	useVIntl,
} from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'servers.manage.instances.meta.title',
		defaultMessage: 'Instances - {server} - Modrinth',
	},
})

useHead({
	title: () =>
		formatMessage(messages.title, {
			server: server.value?.name ?? formatMessage(commonMessages.serverLabel),
		}),
})
</script>

<template>
	<ServersManageInstancesPage />
</template>
