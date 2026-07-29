<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClientIcon, GlobeIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

const { formatMessage } = useVIntl()

export type LegacyProjectEnvironment = {
	clientSide: Labrinth.Projects.v2.Environment
	serverSide: Labrinth.Projects.v2.Environment
}

export type ProjectCardEnvironmentValue =
	| Labrinth.Projects.v3.Environment
	| LegacyProjectEnvironment

export type ProjectCardEnvironmentProps = {
	environment?: ProjectCardEnvironmentValue
	clientSide?: Labrinth.Projects.v2.Environment
	serverSide?: Labrinth.Projects.v2.Environment
}

const props = defineProps<ProjectCardEnvironmentProps>()

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
	singleplayer: {
		id: 'project-card.environment.singleplayer',
		defaultMessage: 'Singleplayer',
	},
	dedicatedServer: {
		id: 'project-card.environment.dedicated-server',
		defaultMessage: 'Dedicated server',
	},
})

const displayEnvironment = computed(() => {
	const environment =
		props.environment ??
		(props.clientSide && props.serverSide
			? { clientSide: props.clientSide, serverSide: props.serverSide }
			: undefined)

	if (typeof environment === 'string') {
		switch (environment) {
			case 'client_or_server':
			case 'client_or_server_prefers_both':
				return 'client-or-server'
			case 'client_and_server':
				return 'client-and-server'
			case 'client_only':
			case 'client_only_server_optional':
				return 'client'
			case 'server_only':
			case 'server_only_client_optional':
				return 'server'
			case 'singleplayer_only':
				return 'singleplayer'
			case 'dedicated_server_only':
				return 'dedicated-server'
			default:
				return undefined
		}
	}

	if (!environment) {
		return undefined
	}

	const { clientSide, serverSide } = environment

	if (clientSide === 'optional' && serverSide === 'optional') {
		return 'client-or-server'
	}
	if (clientSide === 'required' && serverSide === 'required') {
		return 'client-and-server'
	}
	if (
		(clientSide === 'optional' || clientSide === 'required') &&
		(serverSide === 'optional' || serverSide === 'unsupported')
	) {
		return 'client'
	}
	if (
		(serverSide === 'optional' || serverSide === 'required') &&
		(clientSide === 'optional' || clientSide === 'unsupported')
	) {
		return 'server'
	}

	return undefined
})
</script>

<template>
	<TagItem class="empty:hidden">
		<template v-if="displayEnvironment === 'client-or-server'">
			<GlobeIcon aria-hidden="true" />
			{{ formatMessage(messages.clientOrServer) }}
		</template>
		<template v-else-if="displayEnvironment === 'client-and-server'">
			<GlobeIcon aria-hidden="true" />
			{{ formatMessage(messages.clientAndServer) }}
		</template>
		<template v-else-if="displayEnvironment === 'client'">
			<ClientIcon aria-hidden="true" />
			{{ formatMessage(messages.client) }}
		</template>
		<template v-else-if="displayEnvironment === 'server'">
			<ServerIcon aria-hidden="true" />
			{{ formatMessage(messages.server) }}
		</template>
		<template v-else-if="displayEnvironment === 'singleplayer'">
			<UserIcon aria-hidden="true" />
			{{ formatMessage(messages.singleplayer) }}
		</template>
		<template v-else-if="displayEnvironment === 'dedicated-server'">
			<ServerIcon aria-hidden="true" />
			{{ formatMessage(messages.dedicatedServer) }}
		</template>
	</TagItem>
</template>
