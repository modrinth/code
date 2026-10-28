<template>
	<NewModal
		ref="modal"
		:header="`Lift suspension of ${serverName}`"
		:closable="!isLifting"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="warning" header="This lift is temporary">
				The server will be suspended again automatically once the lift ends. Its subdomain stays
				unassigned while the suspension is lifted.
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">Duration</span>
				<Chips
					v-model="selectedDuration"
					:items="durationOptions"
					:format-label="(option) => durationLabels[option]"
					:capitalize="false"
					aria-label="Duration"
				/>
			</div>

			<div v-if="selectedDuration === 'custom'" class="flex flex-col gap-2.5">
				<label class="font-semibold text-contrast" for="lift-suspension-minutes">Minutes</label>
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
					placeholder="Enter a number of minutes"
				/>
				<span
					v-if="customMinutes !== undefined && !isValidMinutes(customMinutes)"
					class="text-sm text-red"
				>
					Enter a whole number of minutes between 1 and {{ MAX_LIFT_MINUTES }} (3 days).
				</span>
				<span v-else-if="customMinutes !== undefined" class="text-sm text-secondary">
					{{ formatDuration(customMinutes) }}
				</span>
			</div>

			<Checkbox
				v-model="restrictPower"
				label="Block power actions while the suspension is lifted"
				description="The server can't be started, so its panel and files can be used without the server being played."
				:disabled="isLifting"
			/>

			<span v-if="liftMinutes !== null" class="text-secondary">
				The server will be suspended again {{ formatEndTime(liftMinutes) }}.
			</span>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isLifting" @click="hide">
					<XIcon />
					Cancel
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
					Lift suspension
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LockOpenIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import Chips from '#ui/components/base/Chips.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'

type DurationOption = '15m' | '1h' | '6h' | '1d' | '3d' | 'custom'

/** The longest a suspension can be temporarily lifted for, enforced by Archon. */
const MAX_LIFT_MINUTES = 3 * 24 * 60

const props = defineProps<{
	serverId: string
	serverName: string
}>()

const emit = defineEmits<{
	lifted: []
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

const durationOptions: DurationOption[] = ['15m', '1h', '6h', '1d', '3d', 'custom']

const durationLabels: Record<DurationOption, string> = {
	'15m': '15 minutes',
	'1h': '1 hour',
	'6h': '6 hours',
	'1d': '1 day',
	'3d': '3 days',
	custom: 'Custom',
}

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

const liftMinutes = computed(() => {
	if (selectedDuration.value !== 'custom') return presetMinutes[selectedDuration.value]
	return customMinutes.value !== undefined && isValidMinutes(customMinutes.value)
		? customMinutes.value
		: null
})

function isValidMinutes(minutes: number) {
	return Number.isInteger(minutes) && minutes >= 1 && minutes <= MAX_LIFT_MINUTES
}

function pluralize(count: number, unit: string) {
	return `${count} ${unit}${count === 1 ? '' : 's'}`
}

function formatDuration(totalMinutes: number) {
	const days = Math.floor(totalMinutes / (24 * 60))
	const hours = Math.floor((totalMinutes % (24 * 60)) / 60)
	const minutes = totalMinutes % 60

	const parts = []
	if (days > 0) parts.push(pluralize(days, 'day'))
	if (hours > 0) parts.push(pluralize(hours, 'hour'))
	if (minutes > 0) parts.push(pluralize(minutes, 'minute'))
	return parts.join(', ')
}

function formatDate(date: Date) {
	return date.toLocaleString('en-US', {
		dateStyle: 'medium',
		timeStyle: 'short',
	})
}

function formatEndTime(minutes: number) {
	return `around ${formatDate(new Date(Date.now() + minutes * 60_000))}`
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
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
		hide()
		addNotification({
			type: 'success',
			title: 'Suspension lifted',
			text: `${props.serverName} is unsuspended until ${formatDate(new Date(response.lifted_until))}${response.restrict_power ? ', with power actions blocked' : ''}.`,
		})
		emit('lifted')
	} catch (error) {
		addNotification({
			type: 'error',
			title: 'Failed to lift suspension',
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
