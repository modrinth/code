<template>
	<div>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(messages.title) }}</h2>
			<p>{{ formatMessage(messages.description) }}</p>

			<div v-if="loading" class="mt-4">
				<p>{{ formatMessage(messages.loading) }}</p>
			</div>
			<div v-else-if="!auth.user.email" class="mt-4">
				<p>{{ formatMessage(messages.noEmail) }}</p>
				<NuxtLink to="/settings/account" class="iconified-button mt-2">
					<MailIcon />
					{{ formatMessage(messages.addEmailButton) }}
				</NuxtLink>
			</div>
			<template v-else>
				<div class="flex flex-col gap-4 mt-4">
					<div
						v-for="notifType in notificationTypes"
						:key="notifType"
						class="flex flex-row flex-wrap items-center justify-between gap-2"
					>
						<label :for="`notif-${notifType}`" class="flex-1">
							<span class="block font-semibold text-contrast">
								{{
									getNotificationTypeLabel(notifType)
										? formatMessage(getNotificationTypeLabel(notifType))
										: notifType
								}}
							</span>
							<span v-if="getNotificationTypeDescription(notifType)" class="text-secondary">
								{{ formatMessage(getNotificationTypeDescription(notifType)) }}
							</span>
						</label>
						<Toggle
							:id="`notif-${notifType}`"
							:model-value="getPreferenceEnabled(notifType)"
							class="shrink-0"
							@update:model-value="(val) => updatePreference(notifType, val)"
						/>
					</div>
				</div>
			</template>
		</section>
		<UnsavedChangesPopup
			:original="originalState"
			:modified="modifiedState"
			:saving="saving"
			@reset="resetPreferences"
			@save="savePreferences"
		/>
	</div>
</template>

<script setup>
import { MailIcon } from '@modrinth/assets'
import {
	defineMessages,
	injectNotificationManager,
	Toggle,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'

useHead({
	title: 'Email notifications - Modrinth',
})

definePageMeta({
	middleware: 'auth',
})

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const messages = defineMessages({
	title: {
		id: 'settings.notifications.title',
		defaultMessage: 'Email notifications',
	},
	description: {
		id: 'settings.notifications.description',
		defaultMessage:
			'Choose which email notifications you would like to receive.',
	},
	loading: {
		id: 'settings.notifications.loading',
		defaultMessage: 'Loading preferences...',
	},
	noEmail: {
		id: 'settings.notifications.no-email',
		defaultMessage:
			'You need to add an email address to your account before you can configure email notifications.',
	},
	addEmailButton: {
		id: 'settings.notifications.action.add-email',
		defaultMessage: 'Add email address',
	},
	errorTitle: {
		id: 'settings.notifications.toast.error.title',
		defaultMessage: 'An error occurred',
	},
})

const notificationTypeMessages = {
	project_update: defineMessages({
		label: {
			id: 'settings.notifications.type.project-update.label',
			defaultMessage: 'Project updates',
		},
		description: {
			id: 'settings.notifications.type.project-update.description',
			defaultMessage: 'When a project you follow publishes a new version.',
		},
	}),
	status_change: defineMessages({
		label: {
			id: 'settings.notifications.type.status-change.label',
			defaultMessage: 'Project status changes',
		},
		description: {
			id: 'settings.notifications.type.status-change.description',
			defaultMessage: 'When the status of your project changes (e.g., approved, rejected).',
		},
	}),
	team_invite: defineMessages({
		label: {
			id: 'settings.notifications.type.team-invite.label',
			defaultMessage: 'Team invitations',
		},
		description: {
			id: 'settings.notifications.type.team-invite.description',
			defaultMessage: 'When you are invited to join a project team.',
		},
	}),
	organization_invite: defineMessages({
		label: {
			id: 'settings.notifications.type.organization-invite.label',
			defaultMessage: 'Organization invitations',
		},
		description: {
			id: 'settings.notifications.type.organization-invite.description',
			defaultMessage: 'When you are invited to join an organization.',
		},
	}),
	moderator_message: defineMessages({
		label: {
			id: 'settings.notifications.type.moderator-message.label',
			defaultMessage: 'Moderator messages',
		},
		description: {
			id: 'settings.notifications.type.moderator-message.description',
			defaultMessage: 'When a moderator sends a message in your project or report thread.',
		},
	}),
	payout_available: defineMessages({
		label: {
			id: 'settings.notifications.type.payout-available.label',
			defaultMessage: 'Payout available',
		},
		description: {
			id: 'settings.notifications.type.payout-available.description',
			defaultMessage: 'When a payout is available for withdrawal.',
		},
	}),
	moderation_message_received: defineMessages({
		label: {
			id: 'settings.notifications.type.moderation-message-received.label',
			defaultMessage: 'Moderation thread messages',
		},
		description: {
			id: 'settings.notifications.type.moderation-message-received.description',
			defaultMessage: "When a moderator sends a message in your project's moderation thread.",
		},
	}),
	report_status_updated: defineMessages({
		label: {
			id: 'settings.notifications.type.report-status-updated.label',
			defaultMessage: 'Report status updated',
		},
		description: {
			id: 'settings.notifications.type.report-status-updated.description',
			defaultMessage: 'When the status of a report you submitted is updated.',
		},
	}),
	report_submitted: defineMessages({
		label: {
			id: 'settings.notifications.type.report-submitted.label',
			defaultMessage: 'Report submitted',
		},
		description: {
			id: 'settings.notifications.type.report-submitted.description',
			defaultMessage: 'When your report has been successfully submitted.',
		},
	}),
	project_status_approved: defineMessages({
		label: {
			id: 'settings.notifications.type.project-status-approved.label',
			defaultMessage: 'Project approved',
		},
		description: {
			id: 'settings.notifications.type.project-status-approved.description',
			defaultMessage: 'When your project is approved by the moderation team.',
		},
	}),
	project_status_neutral: defineMessages({
		label: {
			id: 'settings.notifications.type.project-status-neutral.label',
			defaultMessage: 'Project status updated',
		},
		description: {
			id: 'settings.notifications.type.project-status-neutral.description',
			defaultMessage: "When your project's status is updated to a non-approved state.",
		},
	}),
}

function getNotificationTypeLabel(type) {
	return notificationTypeMessages[type]?.label ?? null
}

function getNotificationTypeDescription(type) {
	return notificationTypeMessages[type]?.description ?? null
}

const loading = ref(true)
const saving = ref(false)
const notificationTypes = ref([])
const preferences = ref([])
const originalPreferences = ref([])

const originalState = computed(() => {
	const state = {}
	for (const notifType of notificationTypes.value) {
		const pref = originalPreferences.value.find(
			(p) => p.notification_type === notifType && p.channel === 'email',
		)
		state[notifType] = pref?.enabled ?? false
	}
	return state
})

const modifiedState = computed(() => {
	const state = {}
	for (const notifType of notificationTypes.value) {
		const pref = preferences.value.find(
			(p) => p.notification_type === notifType && p.channel === 'email',
		)
		state[notifType] = pref?.enabled ?? false
	}
	return state
})

function getPreferenceEnabled(notifType) {
	const pref = preferences.value.find(
		(p) => p.notification_type === notifType && p.channel === 'email',
	)
	return pref?.enabled ?? false
}

function updatePreference(notifType, enabled) {
	const existing = preferences.value.find(
		(p) => p.notification_type === notifType && p.channel === 'email',
	)
	if (existing) {
		existing.enabled = enabled
	} else {
		preferences.value.push({
			notification_type: notifType,
			channel: 'email',
			enabled,
		})
	}
}

async function fetchPreferences() {
	loading.value = true
	try {
		const res = await useBaseFetch('email_preferences', {
			internal: true,
		})
		notificationTypes.value = res.notification_types
		preferences.value = res.preferences.map((p) => ({ ...p }))
		originalPreferences.value = res.preferences.map((p) => ({ ...p }))
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	loading.value = false
}

function resetPreferences() {
	preferences.value = originalPreferences.value.map((p) => ({ ...p }))
}

async function savePreferences() {
	saving.value = true
	try {
		await useBaseFetch('email_preferences', {
			method: 'PATCH',
			internal: true,
			body: {
				preferences: preferences.value,
			},
		})
		originalPreferences.value = preferences.value.map((p) => ({ ...p }))
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	saving.value = false
}

await fetchPreferences()
</script>





