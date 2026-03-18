<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClientIcon, GlobeIcon, ServerIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

const { formatMessage } = useVIntl()

export type ProjectCardEnvironmentProps = {
	clientSide: Labrinth.Projects.v2.Environment
	serverSide: Labrinth.Projects.v2.Environment
}

defineProps<ProjectCardEnvironmentProps>()

const messages = defineMessages({
	clientOrServer: {
		id: 'project-card.environment.client-or-server',
		defaultMessage: 'Client or server',
	},
	clientAndServer: {
		id: 'project-card.environment.client-and-server',
		defaultMessage: 'Client and server',
	},
	client: {
		id: 'project-card.environment.client',
		defaultMessage: 'Client',
	},
	server: {
		id: 'project-card.environment.server',
		defaultMessage: 'Server',
	},
})
</script>

<template>
	<TagItem class="empty:hidden">
		<template v-if="clientSide === 'optional' && serverSide === 'optional'">
			<GlobeIcon aria-hidden="true" />
			{{ formatMessage(messages.clientOrServer) }}
		</template>
		<template v-else-if="clientSide === 'required' && serverSide === 'required'">
			<GlobeIcon aria-hidden="true" />
			{{ formatMessage(messages.clientAndServer) }}
		</template>
		<template
			v-else-if="
				(clientSide === 'optional' || clientSide === 'required') &&
				(serverSide === 'optional' || serverSide === 'unsupported')
			"
		>
			<ClientIcon aria-hidden="true" />
			{{ formatMessage(messages.client) }}
		</template>
		<template
			v-else-if="
				(serverSide === 'optional' || serverSide === 'required') &&
				(clientSide === 'optional' || clientSide === 'unsupported')
			"
		>
			<ServerIcon aria-hidden="true" />
			{{ formatMessage(messages.server) }}
		</template>
	</TagItem>
</template>
