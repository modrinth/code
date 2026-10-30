<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { name: serverName })"
		:closable="!isLifting"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="warning" :header="formatMessage(messages.temporaryHeader)">
				{{ formatMessage(messages.temporaryBody) }}
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.duration) }}
				</span>
				<Chips
					v-model="selectedDuration"
					:items="durationOptions"
					:format-label="(option) => durationLabel(option)"
					:capitalize="false"
					:aria-label="formatMessage(messages.duration)"
				/>
			</div>

			<div v-if="selectedDuration === 'custom'" class="flex flex-col gap-2.5">
				<label class="font-semibold text-contrast" for="lift-suspension-minutes">
					{{ formatMessage(messages.minutes) }}
				</label>
				<Input
					id="lift-suspension-minutes"
					v-model="customMinutes"
					type="number"
					inputmode="numeric"
					:min="1"
					:max="MAX_LIFT_MINUTES"
					:step="1"
					:error="customMinutes !== undefined && !isValidMinutes(customMinutes)"
					:disabled="isLifting"
					:placeholder="formatMessage(messages.minutesPlaceholder)"
				/>
				<span
					v-if="customMinutes !== undefined && !isValidMinutes(customMinutes)"
					class="text-sm text-red"
				>
					{{ formatMessage(messages.minutesInvalid, { max: MAX_LIFT_MINUTES }) }}
				</span>
				<span v-else-if="customMinutes !== undefined" class="text-sm text-secondary">
					{{ formatDuration(customMinutes) }}
				</span>
			</div>

			<Checkbox
				v-model="restrictPower"
				:label="formatMessage(messages.restrictPowerLabel)"
				:description="formatMessage(messages.restrictPowerDescription)"
				:disabled="isLifting"
			/>

			<span v-if="liftEndsAt !== null" class="text-secondary">
				{{ formatMessage(messages.endsAt, { date: formatDateTime(liftEndsAt) }) }}
			</span>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isLifting" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					native-type="button"
					:disabled="liftMinutes === null || isLifting"
					@click="lift"
				>
					<SpinnerIcon v-if="isLifting" class="animate-spin" />
					<LockOpenIcon v-else />
					{{ formatMessage(messages.liftButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LockOpenIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { useNow } from '@vueuse/core'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import Chips from '#ui/components/base/Chips.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

type DurationOption = '15m' | '1h' | '6h' | '1d' | '3d' | 'custom'

/** The longest a suspension can be temporarily lifted for, enforced by Archon. */
const MAX_LIFT_MINUTES = 3 * 24 * 60

const props = defineProps<{
	serverId: string
	serverName: string
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const messages = defineMessages({
	header: {
		id: 'servers.lift-suspension.header',
		defaultMessage: 'Lift suspension of {name}',
	},
	temporaryHeader: {
		id: 'servers.lift-suspension.temporary.header',
		defaultMessage: 'This lift is temporary',
	},
	temporaryBody: {
		id: 'servers.lift-suspension.temporary.body',
		defaultMessage:
			'The server will be suspended again automatically once the lift ends. Its subdomain stays unassigned while the suspension is lifted.',
	},
	duration: {
		id: 'servers.lift-suspension.duration',
		defaultMessage: 'Duration',
	},
	durationMinutes: {
		id: 'servers.lift-suspension.duration.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
	durationHours: {
		id: 'servers.lift-suspension.duration.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	durationDays: {
		id: 'servers.lift-suspension.duration.days',
		defaultMessage: '{count, plural, one {# day} other {# days}}',
	},
	durationCustom: {
		id: 'servers.lift-suspension.duration.custom',
		defaultMessage: 'Custom',
	},
	minutes: {
		id: 'servers.lift-suspension.minutes',
		defaultMessage: 'Minutes',
	},
	minutesPlaceholder: {
		id: 'servers.lift-suspension.minutes.placeholder',
		defaultMessage: 'Enter a number of minutes',
	},
	minutesInvalid: {
		id: 'servers.lift-suspension.minutes.invalid',
		defaultMessage: 'Enter a whole number of minutes between 1 and {max} (3 days).',
	},
	restrictPowerLabel: {
		id: 'servers.lift-suspension.restrict-power.label',
		defaultMessage: 'Block power actions while the suspension is lifted',
	},
	restrictPowerDescription: {
		id: 'servers.lift-suspension.restrict-power.description',
		defaultMessage:
			"The server can't be started, so its panel and files can be used without the server being played.",
	},
	endsAt: {
		id: 'servers.lift-suspension.ends-at',
		defaultMessage: 'The server will be suspended again around {date}.',
	},
	liftButton: {
		id: 'servers.lift-suspension.lift-button',
		defaultMessage: 'Lift suspension',
	},
	successTitle: {
		id: 'servers.lift-suspension.success.title',
		defaultMessage: 'Suspension lifted',
	},
	successText: {
		id: 'servers.lift-suspension.success.text',
		defaultMessage: '{name} is unsuspended until {date}.',
	},
	successTextRestricted: {
		id: 'servers.lift-suspension.success.text-restricted',
		defaultMessage: '{name} is unsuspended until {date}, with power actions blocked.',
	},
	errorTitle: {
		id: 'servers.lift-suspension.error.title',
		defaultMessage: 'Failed to lift suspension',
	},
})

const durationOptions: DurationOption[] = ['15m', '1h', '6h', '1d', '3d', 'custom']

const presetMinutes: Record<Exclude<DurationOption, 'custom'>, number> = {
	'15m': 15,
	'1h': 60,
	'6h': 6 * 60,
	'1d': 24 * 60,
	'3d': MAX_LIFT_MINUTES,
}

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const selectedDuration = ref<DurationOption>('15m')
const customMinutes = ref<number | undefined>()
const restrictPower = ref(true)
const isLifting = ref(false)
const now = useNow({ interval: 30_000 })

const liftMinutes = computed(() => {
	if (selectedDuration.value !== 'custom') return presetMinutes[selectedDuration.value]
	return customMinutes.value !== undefined && isValidMinutes(customMinutes.value)
		? customMinutes.value
		: null
})

const liftEndsAt = computed(() =>
	liftMinutes.value === null ? null : new Date(now.value.getTime() + liftMinutes.value * 60_000),
)

function isValidMinutes(minutes: number) {
	return Number.isInteger(minutes) && minutes >= 1 && minutes <= MAX_LIFT_MINUTES
}

function durationLabel(option: DurationOption) {
	return option === 'custom'
		? formatMessage(messages.durationCustom)
		: formatDuration(presetMinutes[option])
}

function formatDuration(totalMinutes: number) {
	const days = Math.floor(totalMinutes / (24 * 60))
	const hours = Math.floor((totalMinutes % (24 * 60)) / 60)
	const minutes = totalMinutes % 60

	const parts = []
	if (days > 0) parts.push(formatMessage(messages.durationDays, { count: days }))
	if (hours > 0) parts.push(formatMessage(messages.durationHours, { count: hours }))
	if (minutes > 0) parts.push(formatMessage(messages.durationMinutes, { count: minutes }))
	return parts.join(', ')
}

function show(): void {
	selectedDuration.value = '15m'
	customMinutes.value = undefined
	restrictPower.value = true
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function lift(): Promise<void> {
	const minutes = liftMinutes.value
	if (minutes === null || isLifting.value) return

	isLifting.value = true
	try {
		const response = await client.archon.servers_internal.liftSuspension(props.serverId, {
			minutes,
			restrict_power: restrictPower.value,
		})
		hide()
		addNotification({
			type: 'success',
			title: formatMessage(messages.successTitle),
			text: formatMessage(
				response.restrict_power ? messages.successTextRestricted : messages.successText,
				{ name: props.serverName, date: formatDateTime(response.lifted_until) },
			),
		})
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: error instanceof Error ? error.message : undefined,
		})
	} finally {
		isLifting.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
