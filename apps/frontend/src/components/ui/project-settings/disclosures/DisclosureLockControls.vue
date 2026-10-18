<script setup lang="ts">
import { CircleSlashIcon, LockIcon, LockOpenIcon } from '@modrinth/assets'
import { Button, ButtonGroup, defineMessages, useVIntl } from '@modrinth/ui'

import type { DisclosureLockStatus } from './types'

defineProps<{ lockStatus: DisclosureLockStatus; disabled?: boolean }>()
const emit = defineEmits<{ setLockStatus: [status: DisclosureLockStatus] }>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	label: { id: 'project.disclosures.lock-status.label', defaultMessage: 'Lock status' },
	unlocked: { id: 'project.disclosures.lock-status.unlocked', defaultMessage: 'Unlocked' },
	cannotDisable: {
		id: 'project.disclosures.lock-status.cannot-disable',
		defaultMessage: 'Cannot disable',
	},
	fullyLocked: {
		id: 'project.disclosures.lock-status.fully-locked',
		defaultMessage: 'Fully locked',
	},
})
const statuses = [
	{ status: 'unlocked', label: messages.unlocked },
	{ status: 'cannot_disable', label: messages.cannotDisable },
	{ status: 'fully_locked', label: messages.fullyLocked },
] as const
</script>

<template>
	<ButtonGroup :label="formatMessage(messages.label)">
		<Button
			v-for="{ status, label } in statuses"
			:key="status"
			:color="
				status === 'cannot_disable' ? 'orange' : status === 'fully_locked' ? 'red' : undefined
			"
			:type="status !== 'unlocked' ? 'colored-text' : undefined"
			:disabled="disabled || lockStatus === status"
			@click="emit('setLockStatus', status)"
		>
			<LockOpenIcon v-if="status === 'unlocked'" />
			<CircleSlashIcon v-else-if="status === 'cannot_disable'" />
			<LockIcon v-else />
			{{ formatMessage(label) }}
		</Button>
	</ButtonGroup>
</template>
