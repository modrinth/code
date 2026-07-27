<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" max-width="30rem">
		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.expiryLabel) }}</span>
				<DatePicker
					v-model="expiry"
					:disabled="saving"
					:min-date="minimumExpiry"
					:max-date="maximumExpiry"
					date-format="Y-m-d H:i"
					alt-format="F j, Y at h:i K"
					enable-time
					wrapper-class="w-full"
					input-class="w-full"
				/>
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

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectNotificationManager } from '../../../providers'
import ButtonStyled from '../../base/ButtonStyled.vue'
import DatePicker from '../../base/DatePicker.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'
import type { InviteLinkSettings } from './types'

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
const maxUses = ref<number>()
const minimumExpiry = ref(new Date())
const maximumExpiry = ref(new Date())
const saving = ref(false)
const maximumUses = computed(() => Math.max(0, Math.floor(props.linkMaxUsesLimit)))

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
	cancel: {
		id: 'sharing.invite-players-modal.cancel-button',
		defaultMessage: 'Cancel',
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

function show() {
	const now = new Date()
	minimumExpiry.value = new Date(now.getTime() + 3_600_000)
	minimumExpiry.value.setSeconds(0, 0)
	minimumExpiry.value.setMinutes(minimumExpiry.value.getMinutes() + 1)
	maximumExpiry.value = new Date(now.getTime() + 7 * 86_400_000)
	maximumExpiry.value.setSeconds(0, 0)
	const currentExpiry = props.linkExpiresAt ? new Date(props.linkExpiresAt) : maximumExpiry.value
	const date =
		Number.isNaN(currentExpiry.getTime()) || currentExpiry < minimumExpiry.value
			? minimumExpiry.value
			: currentExpiry > maximumExpiry.value
				? maximumExpiry.value
				: currentExpiry
	expiry.value = formatLocalDate(date)
	maxUses.value = Math.min(props.linkMaxUses, maximumUses.value)
	modal.value?.show()
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
