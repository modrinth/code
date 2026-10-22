<script setup lang="ts">
import { CheckIcon, CircleSlashIcon, LockIcon, LockOpenIcon } from '@modrinth/assets'
import { Button, ButtonGroup, Combobox, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import type { DisclosureLockStatus } from './types'

defineProps<{
	lockStatus: DisclosureLockStatus
	disabled?: boolean
	compact?: boolean
}>()
const emit = defineEmits<{ setLockStatus: [status: DisclosureLockStatus] }>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	label: {
		id: 'project.disclosures.lock-status.label',
		defaultMessage: 'Lock status',
	},
	changeLockStatus: {
		id: 'project.disclosures.lock-status.change',
		defaultMessage: 'Change lock status',
	},
	unlocked: {
		id: 'project.disclosures.lock-status.unlocked',
		defaultMessage: 'Unlocked',
	},
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
const options = computed(() =>
	statuses.map(({ status, label }) => ({
		value: status,
		label: formatMessage(label),
		class:
			status === 'cannot_disable'
				? '!text-orange [&[aria-selected=true]]:!bg-orange-highlight'
				: status === 'fully_locked'
					? '!text-red [&[aria-selected=true]]:!bg-red-highlight'
					: '!text-primary [&[aria-selected=true]]:!bg-surface-5',
		icon:
			status === 'unlocked'
				? LockOpenIcon
				: status === 'cannot_disable'
					? CircleSlashIcon
					: LockIcon,
	})),
)
</script>

<template>
	<Combobox
		v-if="compact"
		v-tooltip="formatMessage(messages.changeLockStatus)"
		:model-value="lockStatus"
		:options="options"
		:aria-label="formatMessage(messages.changeLockStatus)"
		:disabled="disabled"
		class="!w-8 shrink-0"
		:trigger-class="
			[
				'!grid !grid-cols-1 !size-8 !min-w-0 !place-items-center !gap-0 !rounded-full !p-0 [&>div:last-child]:hidden',
				lockStatus === 'cannot_disable'
					? '!bg-orange !text-[var(--color-accent-contrast)]'
					: lockStatus === 'fully_locked'
						? '!bg-red !text-[var(--color-accent-contrast)]'
						: '',
			].join(' ')
		"
		trigger-type="base"
		trigger-interaction="surface"
		:show-chevron="false"
		:dropdown-min-width="'210px'"
		dropdown-align="right"
		trigger-size="sm"
		@update:model-value="emit('setLockStatus', $event)"
	>
		<template #option="{ item, isSelected }">
			<div class="flex w-full items-center gap-2 font-semibold">
				<component :is="item.icon" class="size-5 shrink-0" aria-hidden="true" />
				<span class="flex-1">{{ item.label }}</span>
				<CheckIcon v-if="isSelected" class="size-4 shrink-0" aria-hidden="true" />
			</div>
		</template>
		<template #selected="{ label }">
			<span class="sr-only">{{ label }}</span>
			<span class="flex size-4 items-center justify-center">
				<LockOpenIcon v-if="lockStatus === 'unlocked'" class="size-4" aria-hidden="true" />
				<CircleSlashIcon
					v-else-if="lockStatus === 'cannot_disable'"
					class="size-4"
					aria-hidden="true"
				/>
				<LockIcon v-else class="size-4" aria-hidden="true" />
			</span>
		</template>
	</Combobox>
	<ButtonGroup v-else :label="formatMessage(messages.label)">
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
