<template>
	<Admonition
		type="warning"
		:header="formatMessage(sharedInstanceErrorMessages.unavailableTitle)"
		:dismissible="dismissible"
		@dismiss="emit('dismiss')"
	>
		{{ formatSharedInstanceUnavailable(reason ?? null, manager) }}
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import type { SharedInstanceUnavailableReason } from '@/helpers/install'
import {
	sharedInstanceErrorMessages,
	useSharedInstanceErrors,
} from '@/helpers/shared-instance-errors'

defineProps<{
	reason?: SharedInstanceUnavailableReason | null
	manager?: string | null
	dismissible?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
}>()

const { formatMessage } = useVIntl()
const { formatSharedInstanceUnavailable } = useSharedInstanceErrors()
</script>
