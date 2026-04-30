<script setup lang="ts">
import {
	commonMessages,
	defineMessages,
	injectModrinthServerContext,
	ServersManageWorldsPage,
	useVIntl,
} from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'servers.manage.worlds.meta.title',
		defaultMessage: 'Worlds - {server} - Modrinth',
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
	<ServersManageWorldsPage />
</template>
