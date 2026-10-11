<template>
	<Admonition
		type="critical"
		inline-actions
		:header="formatMessage(messages.linkedServerFailedHeader)"
	>
		{{
			formatMessage(messages.linkedServerFailedBody, {
				count: servers.length,
				name: servers[0]?.name,
			})
		}}
		<template #actions>
			<InstancePushButton :instance="instance" primary />
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import InstancePushButton from '@/components/ui/linked-servers/InstancePushButton.vue'
import type { LinkedServer } from '@/helpers/linked-servers'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './messages'

/** Shown while a server linked to this instance failed to take it, e.g. the upload never arrived. */
defineProps<{
	instance: GameInstance
	servers: LinkedServer[]
}>()

const { formatMessage } = useVIntl()
</script>
