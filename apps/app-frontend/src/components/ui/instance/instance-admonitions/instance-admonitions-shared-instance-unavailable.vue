<template>
	<Admonition type="warning" :header="formatMessage(messages.sharedInstanceUnavailableTitle)">
		{{
			formatMessage(sharedInstanceUnavailableTextMessage(reason ?? null), {
				manager: manager || formatMessage(messages.sharedInstanceUnavailableFallbackManager),
			})
		}}
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import type { SharedInstanceUnavailableReason } from '@/helpers/install'

import {
	instanceAdmonitionsMessages as messages,
	sharedInstanceUnavailableTextMessage,
} from './instance-admonitions-messages'

defineProps<{
	reason?: SharedInstanceUnavailableReason | null
	manager?: string | null
}>()

const { formatMessage } = useVIntl()
</script>
