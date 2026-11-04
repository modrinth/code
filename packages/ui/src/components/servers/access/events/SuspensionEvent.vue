<template>
	<BaseEvent>
		<IntlFormatted :message-id="message" :values="{ reason: reason ?? 'other' }">
			<template #until>
				<EventInlineText :text="untilLabel" class="align-middle font-medium text-contrast" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useFormatDateTime } from '../../../../composables/format-date-time'
import { defineMessages } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventInlineText from './EventInlineText.vue'

const props = defineProps<{
	kind: 'suspended' | 'unsuspended' | 'lifted' | 'resuspended'
	reason?: string
	until?: string
	restrictPower?: boolean
	expired?: boolean
}>()

const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const messages = defineMessages({
	suspended: {
		id: 'servers.audit-log.event.server-suspended',
		defaultMessage:
			'{reason, select, moderated {Server suspended by moderation action} paymentfailed {Server suspended due to a failed payment} cancelled {Server suspended due to a cancelled subscription} other {Server suspended}}',
	},
	unsuspended: {
		id: 'servers.audit-log.event.server-unsuspended',
		defaultMessage: 'Server unsuspended',
	},
	lifted: {
		id: 'servers.audit-log.event.server-suspension-lifted',
		defaultMessage: 'Temporarily lifted suspension until <until></until>',
	},
	liftedRestricted: {
		id: 'servers.audit-log.event.server-suspension-lifted-restricted',
		defaultMessage:
			'Temporarily lifted suspension until <until></until> with power actions blocked',
	},
	resuspended: {
		id: 'servers.audit-log.event.server-resuspended',
		defaultMessage: 'Ended suspension lift early, server suspended again',
	},
	resuspendedExpired: {
		id: 'servers.audit-log.event.server-resuspended-expired',
		defaultMessage: 'Suspension lift expired, server suspended again',
	},
})

const message = computed(() => {
	switch (props.kind) {
		case 'suspended':
			return messages.suspended
		case 'unsuspended':
			return messages.unsuspended
		case 'lifted':
			return props.restrictPower ? messages.liftedRestricted : messages.lifted
		case 'resuspended':
			return props.expired ? messages.resuspendedExpired : messages.resuspended
		default:
			return messages.suspended
	}
})

const untilLabel = computed(() => (props.until ? formatDateTime(props.until) : ''))
</script>
