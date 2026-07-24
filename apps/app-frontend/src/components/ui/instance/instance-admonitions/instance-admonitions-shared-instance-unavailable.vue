<template>
	<Admonition
		type="warning"
		:inline-actions="reason === 'quarantined'"
		:header="formatMessage(sharedInstanceUnavailableTitleMessage(reason ?? null))"
		:dismissible="dismissible"
		@dismiss="emit('dismiss')"
	>
		{{ formatSharedInstanceUnavailable(reason ?? null, manager) }}
		<template v-if="reason === 'quarantined'" #actions>
			<ButtonStyled color="orange">
				<button class="!h-10" @click="emit('delete')">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.deleteInstance) }}
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { TrashIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'

import type { SharedInstanceUnavailableReason } from '@/helpers/install'
import {
	sharedInstanceUnavailableTitleMessage,
	useSharedInstanceErrors,
} from '@/helpers/shared-instance-errors'

defineProps<{
	reason?: SharedInstanceUnavailableReason | null
	manager?: string | null
	dismissible?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
	delete: []
}>()

const { formatMessage } = useVIntl()
const { formatSharedInstanceUnavailable } = useSharedInstanceErrors()

const messages = defineMessages({
	deleteInstance: {
		id: 'instance.locked.delete-button',
		defaultMessage: 'Delete instance',
	},
})
</script>
