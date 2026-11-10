<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title, { username: user.username })"
		:closable="!isRevoking"
		fade="danger"
		max-width="500px"
	>
		<Admonition type="critical" :header="formatMessage(messages.admonitionTitle)">
			{{ formatMessage(messages.admonitionBody, { username: user.username }) }}
		</Admonition>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" native-type="button" :disabled="isRevoking" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					native-type="button"
					:disabled="isRevoking"
					@click="revoke"
				>
					<SpinnerIcon v-if="isRevoking" class="animate-spin" />
					<LogOutIcon v-else />
					{{ formatMessage(messages.button) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LogOutIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

const props = defineProps<{
	user: Labrinth.Users.v3.User
}>()

const client = injectModrinthClient()
const notificationManager = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'profile.revoke-sessions.title',
		defaultMessage: 'Revoke sessions for {username}',
	},
	admonitionTitle: {
		id: 'profile.revoke-sessions.admonition-title',
		defaultMessage: 'Are you sure you want to revoke all sessions?',
	},
	admonitionBody: {
		id: 'profile.revoke-sessions.admonition-body',
		defaultMessage:
			'{username} will be signed out on every device and will need to sign in again. Personal access tokens are not affected.',
	},
	button: {
		id: 'profile.revoke-sessions.button',
		defaultMessage: 'Revoke sessions',
	},
	successTitle: {
		id: 'profile.revoke-sessions.success-title',
		defaultMessage: 'Sessions revoked',
	},
	successDescription: {
		id: 'profile.revoke-sessions.success-description',
		defaultMessage: '{username} has been signed out everywhere.',
	},
	errorTitle: {
		id: 'profile.revoke-sessions.error-title',
		defaultMessage: 'Failed to revoke sessions',
	},
	errorDescription: {
		id: 'profile.revoke-sessions.error-description',
		defaultMessage: 'An error occurred while revoking sessions. Please try again.',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const isRevoking = ref(false)

function show(): void {
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function revoke(): Promise<void> {
	if (isRevoking.value) return

	isRevoking.value = true
	try {
		await client.labrinth.moderation_internal.revokeUserSessions(props.user.id)
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
		isRevoking.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
