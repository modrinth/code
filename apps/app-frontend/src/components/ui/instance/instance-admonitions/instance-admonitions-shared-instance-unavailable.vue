<template>
	<Admonition
		type="warning"
		:header="formatMessage(messages.sharedInstanceUnavailableTitle)"
		:dismissible="dismissible"
		@dismiss="emit('dismiss')"
	>
		{{
			formatMessage(sharedInstanceUnavailableTextMessage(reason ?? null), {
				manager: manager || formatMessage(messages.sharedInstanceUnavailableFallbackManager),
			})
		}}
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import {
	instanceAdmonitionsMessages as messages,
	sharedInstanceUnavailableTextMessage,
} from './instance-admonitions-messages'
import type { SharedInstanceUnavailableReason } from '@/helpers/install'

defineProps<{
	reason?: SharedInstanceUnavailableReason | null
	manager?: string | null
	dismissible?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
}>()

const { formatMessage } = useVIntl()
</script>
