<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" width="420px" max-width="420px">
		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.expiryLabel) }}</span>
				<Combobox
					:model-value="selectedExpiryPreset"
					:options="expiryDropdownOptions"
					:display-value="expiryPickerLabel"
					:disabled="saving"
					:dropdown-min-width="customExpiryOpen ? '20rem' : undefined"
					:dropdown-class="customExpiryOpen ? 'bg-transparent border-0 -mt-1 pb-2 shadow-none' : ''"
					@open="handleExpiryPickerOpen"
					@close="handleExpiryPickerClose"
					@select="selectExpiryPreset"
				>
					<template #dropdown-footer>
						<div
							v-if="customExpiryOpen"
							class="flex flex-col rounded-2xl border border-solid border-surface-5 bg-surface-3 p-1"
						>
							<DatePicker
								v-model="customExpiry"
								:min-date="minimumExpiry"
								:max-date="maximumExpiry"
								:default-view-date="customExpiry || minimumExpiry"
								date-format="Y-m-d H:i"
								enable-time
								calendar-only
								wrapper-class="w-full"
								calendar-class="!border-none"
							/>
							<div class="flex justify-end gap-2 p-3 pt-1">
								<ButtonStyled type="outlined">
									<button type="button" @click="cancelCustomExpiry">
										{{ formatMessage(messages.cancel) }}
									</button>
								</ButtonStyled>
								<ButtonStyled color="brand">
									<button
										type="button"
										:disabled="!canApplyCustomExpiry"
										@click="applyCustomExpiry"
									>
										{{ formatMessage(messages.apply) }}
									</button>
								</ButtonStyled>
							</div>
						</div>
						<button
							v-else
							type="button"
							class="flex w-full cursor-pointer items-center border-0 border-t border-solid border-surface-5 bg-transparent px-4 py-3 text-left text-base font-semibold leading-tight text-primary transition-colors hover:bg-surface-5"
							@click.stop="openCustomExpiry"
						>
							{{ formatMessage(messages.customExpiry) }}
						</button>
					</template>
				</Combobox>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.maxUsesLabel) }}</span>
				<StyledInput
					v-model="maxUses"
					type="number"
					:min="1"
					:max="maximumUses"
					:step="1"
					:disabled="saving || maximumUses === 0"
					clamp
				/>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button :disabled="saving" @click="modal?.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canSave" @click="save">
						<SpinnerIcon v-if="saving" class="animate-spin" aria-hidden="true" />
						<SaveIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.save) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { useFormatDateTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectNotificationManager } from '../../../providers'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import DatePicker from '../../base/DatePicker.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'
import type { InviteLinkSettings } from './types'

const EXPIRY_PRESET_DURATIONS = {
	one_hour: 3_600_000,
	six_hours: 6 * 3_600_000,
	twelve_hours: 12 * 3_600_000,
	one_day: 86_400_000,
	three_days: 3 * 86_400_000,
	seven_days: 7 * 86_400_000,
} as const
const MINIMUM_EXPIRY_DURATION = EXPIRY_PRESET_DURATIONS.one_hour
const MAXIMUM_EXPIRY_DURATION = EXPIRY_PRESET_DURATIONS.seven_days
const EXPIRY_PRESET_MATCH_TOLERANCE = 2 * 60_000

type ExpiryPreset = keyof typeof EXPIRY_PRESET_DURATIONS

const props = withDefaults(
	defineProps<{
		linkExpiresAt?: string | Date | null
		linkMaxUses: number
		linkMaxUsesLimit?: number
		updateInviteLink?: (settings: InviteLinkSettings) => Promise<void>
	}>(),
	{
		linkMaxUsesLimit: 2147483647,
	},
)
const { formatMessage } = useVIntl()
const notificationManager = injectNotificationManager(null)
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const expiry = ref('')
const expiryMode = ref<'preset' | 'custom'>('preset')
const expiryPreset = ref<ExpiryPreset>('seven_days')
const expiryReferenceTime = ref(Date.now())
const customExpiry = ref('')
const customExpiryOpen = ref(false)
const maxUses = ref<number>()
const minimumExpiry = ref(new Date())
const maximumExpiry = ref(new Date())
const saving = ref(false)
const maximumUses = computed(() => Math.max(0, Math.floor(props.linkMaxUsesLimit)))
const formatExpiryDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const messages = defineMessages({
	title: {
		id: 'sharing.invite-players-modal.edit-invite-link-title',
		defaultMessage: 'Edit invite link',
	},
	expiryLabel: {
		id: 'sharing.invite-players-modal.expiry-label',
		defaultMessage: 'Expiry date',
	},
	maxUsesLabel: {
		id: 'sharing.invite-players-modal.max-uses-label',
		defaultMessage: 'Maximum uses',
	},
	inOneHour: {
		id: 'sharing.invite-players-modal.expiry-in-one-hour',
		defaultMessage: 'In 1 hour',
	},
	inSixHours: {
		id: 'sharing.invite-players-modal.expiry-in-six-hours',
		defaultMessage: 'In 6 hours',
	},
	inTwelveHours: {
		id: 'sharing.invite-players-modal.expiry-in-twelve-hours',
		defaultMessage: 'In 12 hours',
	},
	inOneDay: {
		id: 'sharing.invite-players-modal.expiry-in-one-day',
		defaultMessage: 'In 1 day',
	},
	inThreeDays: {
		id: 'sharing.invite-players-modal.expiry-in-three-days',
		defaultMessage: 'In 3 days',
	},
	inSevenDays: {
		id: 'sharing.invite-players-modal.expiry-in-seven-days',
		defaultMessage: 'In 7 days',
	},
	customExpiry: {
		id: 'sharing.invite-players-modal.custom-expiry',
		defaultMessage: 'Custom...',
	},
	customExpiryValue: {
		id: 'sharing.invite-players-modal.custom-expiry-value',
		defaultMessage: 'Custom: {date}',
	},
	cancel: {
		id: 'sharing.invite-players-modal.cancel-button',
		defaultMessage: 'Cancel',
	},
	apply: {
		id: 'sharing.invite-players-modal.apply-button',
		defaultMessage: 'Apply',
	},
	save: {
		id: 'sharing.invite-players-modal.save-button',
		defaultMessage: 'Save',
	},
	failed: {
		id: 'sharing.invite-players-modal.update-invite-link-failed-title',
		defaultMessage: 'Failed to update invite link',
	},
})

const expiryOptions = computed<ComboboxOption<ExpiryPreset>[]>(() => [
	{ value: 'one_hour', label: formatMessage(messages.inOneHour) },
	{ value: 'six_hours', label: formatMessage(messages.inSixHours) },
	{ value: 'twelve_hours', label: formatMessage(messages.inTwelveHours) },
	{ value: 'one_day', label: formatMessage(messages.inOneDay) },
	{ value: 'three_days', label: formatMessage(messages.inThreeDays) },
	{ value: 'seven_days', label: formatMessage(messages.inSevenDays) },
])
const expiryDropdownOptions = computed(() => (customExpiryOpen.value ? [] : expiryOptions.value))
const selectedExpiryPreset = computed(() =>
	expiryMode.value === 'preset' ? expiryPreset.value : undefined,
)
const expiryPickerLabel = computed(() => {
	if (expiryMode.value === 'preset') {
		return (
			expiryOptions.value.find((option) => option.value === expiryPreset.value)?.label ??
			formatMessage(messages.inSevenDays)
		)
	}

	const date = parseLocalDate(expiry.value)
	return date
		? formatMessage(messages.customExpiryValue, { date: formatExpiryDate(date) })
		: formatMessage(messages.customExpiry)
})
const canApplyCustomExpiry = computed(() => {
	const date = parseLocalDate(customExpiry.value)
	return !!date && date >= minimumExpiry.value && date <= maximumExpiry.value
})
const canSave = computed(() => {
	const date = parseLocalDate(expiry.value)
	return (
		!saving.value &&
		!!date &&
		date >= minimumExpiry.value &&
		date <= maximumExpiry.value &&
		Number.isInteger(maxUses.value ?? 0) &&
		(maxUses.value ?? 0) > 0 &&
		(maxUses.value ?? 0) <= maximumUses.value
	)
})

function formatLocalDate(date: Date) {
	const pad = (value: number) => value.toString().padStart(2, '0')
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

function parseLocalDate(value: string) {
	const match = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})$/.exec(value)
	if (!match) return null
	const [, year, month, day, hour, minute] = match
	const date = new Date(Number(year), Number(month) - 1, Number(day), Number(hour), Number(minute))
	return Number.isNaN(date.getTime()) ? null : date
}

function roundDownToMinute(timestamp: number) {
	const date = new Date(timestamp)
	date.setSeconds(0, 0)
	return date
}

function roundUpToMinute(timestamp: number) {
	const date = roundDownToMinute(timestamp)
	if (date.getTime() < timestamp) date.setMinutes(date.getMinutes() + 1)
	return date
}

function expiryForPreset(preset: ExpiryPreset) {
	const expiryTimestamp = expiryReferenceTime.value + EXPIRY_PRESET_DURATIONS[preset]
	const date = roundDownToMinute(expiryTimestamp)
	if (date < minimumExpiry.value) return minimumExpiry.value
	if (date > maximumExpiry.value) return maximumExpiry.value
	return date
}

function matchingExpiryPreset(date: Date) {
	const duration = date.getTime() - expiryReferenceTime.value
	let closestPreset: ExpiryPreset | null = null
	let closestDifference = Number.POSITIVE_INFINITY

	for (const [preset, presetDuration] of Object.entries(EXPIRY_PRESET_DURATIONS) as Array<
		[ExpiryPreset, number]
	>) {
		const difference = Math.abs(duration - presetDuration)
		if (difference < closestDifference) {
			closestPreset = preset
			closestDifference = difference
		}
	}

	return closestDifference <= EXPIRY_PRESET_MATCH_TOLERANCE ? closestPreset : null
}

function show() {
	expiryReferenceTime.value = Date.now()
	minimumExpiry.value = roundUpToMinute(expiryReferenceTime.value + MINIMUM_EXPIRY_DURATION)
	maximumExpiry.value = roundDownToMinute(expiryReferenceTime.value + MAXIMUM_EXPIRY_DURATION)
	const currentExpiry = props.linkExpiresAt ? new Date(props.linkExpiresAt) : maximumExpiry.value
	const date =
		Number.isNaN(currentExpiry.getTime()) || currentExpiry < minimumExpiry.value
			? minimumExpiry.value
			: currentExpiry > maximumExpiry.value
				? maximumExpiry.value
				: currentExpiry
	expiry.value = formatLocalDate(date)
	const matchingPreset = matchingExpiryPreset(date)
	expiryMode.value = matchingPreset ? 'preset' : 'custom'
	if (matchingPreset) expiryPreset.value = matchingPreset
	customExpiry.value = expiry.value
	customExpiryOpen.value = false
	maxUses.value = Math.min(props.linkMaxUses, maximumUses.value)
	modal.value?.show()
}

function selectExpiryPreset(option: ComboboxOption<ExpiryPreset>) {
	expiryMode.value = 'preset'
	expiryPreset.value = option.value
	expiry.value = formatLocalDate(expiryForPreset(option.value))
}

function handleExpiryPickerOpen() {
	customExpiryOpen.value = false
}

function handleExpiryPickerClose() {
	customExpiryOpen.value = false
	customExpiry.value = expiry.value
}

function openCustomExpiry() {
	customExpiry.value = expiry.value
	customExpiryOpen.value = true
}

function cancelCustomExpiry() {
	customExpiry.value = expiry.value
	customExpiryOpen.value = false
}

function closeExpiryPicker(event: Event) {
	const target = event.target
	if (!(target instanceof HTMLElement)) return
	target
		.closest('[role="listbox"], [role="menu"]')
		?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
}

function applyCustomExpiry(event: MouseEvent) {
	const date = parseLocalDate(customExpiry.value)
	if (!canApplyCustomExpiry.value || !date) return
	expiryMode.value = 'custom'
	expiry.value = formatLocalDate(date)
	closeExpiryPicker(event)
}

async function save() {
	const date = parseLocalDate(expiry.value)
	if (!canSave.value || !date || !props.updateInviteLink) return
	const clampedMaxUses = Math.min(maxUses.value ?? 1, maximumUses.value)
	saving.value = true
	try {
		await props.updateInviteLink({ expiresAt: date, maxUses: clampedMaxUses })
		modal.value?.hide()
	} catch (error) {
		notificationManager?.addNotification({
			type: 'error',
			title: formatMessage(messages.failed),
			text: error instanceof Error ? error.message : String(error),
		})
	} finally {
		saving.value = false
	}
}

watch([maxUses, maximumUses], ([uses, limit]) => {
	if (uses !== undefined && uses > limit) maxUses.value = limit
})

defineExpose({ show })
</script>
