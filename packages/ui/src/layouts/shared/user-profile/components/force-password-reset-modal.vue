<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title, { username: user.username })"
		:closable="!isResetting"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="critical" :header="formatMessage(messages.admonitionTitle)">
				{{ formatMessage(messages.admonitionBody, { username: user.username }) }}
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="force-password-reset-email">
					{{ formatMessage(messages.emailLabel) }}
				</label>
				<Input
					id="force-password-reset-email"
					v-model="email"
					type="email"
					:placeholder="formatMessage(messages.emailPlaceholder)"
					:disabled="isResetting"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isResetting" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					native-type="button"
					:disabled="isResetting"
					@click="reset"
				>
					<SpinnerIcon v-if="isResetting" class="animate-spin" />
					<KeyIcon v-else />
					{{ formatMessage(messages.button) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { KeyIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { Input } from '#ui/components/base/inputs'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

const props = defineProps<{
	user: Labrinth.Users.v3.User
	userId: string
}>()

const client = injectModrinthClient()
const notificationManager = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'profile.force-password-reset.title',
		defaultMessage: 'Force password reset for {username}',
	},
	admonitionTitle: {
		id: 'profile.force-password-reset.admonition-title',
		defaultMessage: 'The current password will stop working',
	},
	admonitionBody: {
		id: 'profile.force-password-reset.admonition-body',
		defaultMessage:
			"{username}'s password will be removed immediately, they will be signed out on every device, and a link to set a new one will be sent to the email below. If you change the email, it replaces the one on the account. The link expires after 24 hours and can be used even while the account is locked.",
	},
	emailLabel: {
		id: 'profile.force-password-reset.email-label',
		defaultMessage: 'Account email',
	},
	emailPlaceholder: {
		id: 'profile.force-password-reset.email-placeholder',
		defaultMessage: "Leave blank to use the account's email",
	},
	button: {
		id: 'profile.force-password-reset.button',
		defaultMessage: 'Reset password',
	},
	successTitle: {
		id: 'profile.force-password-reset.success-title',
		defaultMessage: 'Password reset',
	},
	successDescription: {
		id: 'profile.force-password-reset.success-description',
		defaultMessage: 'A password reset link has been sent to {username}.',
	},
	errorTitle: {
		id: 'profile.force-password-reset.error-title',
		defaultMessage: 'Failed to reset password',
	},
	errorDescription: {
		id: 'profile.force-password-reset.error-description',
		defaultMessage: 'An error occurred while resetting the password. Please try again.',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const email = ref('')
const isResetting = ref(false)

function show(): void {
	email.value = props.user.email ?? ''
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function reset(): Promise<void> {
	if (isResetting.value) return

	isResetting.value = true
	try {
		await client.labrinth.moderation_internal.forceUserPasswordReset(
			props.user.id,
			email.value.trim() || undefined,
		)
		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		hide()
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.successTitle),
			text: formatMessage(messages.successDescription, { username: props.user.username }),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: formatMessage(messages.errorDescription),
		})
	} finally {
		isResetting.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
