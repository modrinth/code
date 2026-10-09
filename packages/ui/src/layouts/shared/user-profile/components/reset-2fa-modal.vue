<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title, { username: user.username })"
		:closable="!isResetting"
		fade="danger"
		max-width="500px"
	>
		<Admonition type="critical" :header="formatMessage(messages.admonitionTitle)">
			{{ formatMessage(messages.admonitionBody, { username: user.username }) }}
		</Admonition>

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
					<ShieldAlertIcon v-else />
					{{ formatMessage(messages.button) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ShieldAlertIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
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
		id: 'profile.reset-2fa.title',
		defaultMessage: 'Reset two-factor authentication for {username}',
	},
	admonitionTitle: {
		id: 'profile.reset-2fa.admonition-title',
		defaultMessage: 'Are you sure you want to reset two-factor authentication?',
	},
	admonitionBody: {
		id: 'profile.reset-2fa.admonition-body',
		defaultMessage:
			"{username}'s authenticator and backup codes will be removed and they will be notified. Only do this after verifying their identity.",
	},
	button: {
		id: 'profile.reset-2fa.button',
		defaultMessage: 'Reset 2FA',
	},
	successTitle: {
		id: 'profile.reset-2fa.success-title',
		defaultMessage: 'Two-factor authentication reset',
	},
	successDescription: {
		id: 'profile.reset-2fa.success-description',
		defaultMessage: "{username}'s two-factor authentication has been removed.",
	},
	errorTitle: {
		id: 'profile.reset-2fa.error-title',
		defaultMessage: 'Failed to reset two-factor authentication',
	},
	errorDescription: {
		id: 'profile.reset-2fa.error-description',
		defaultMessage:
			'An error occurred while resetting two-factor authentication. Please try again.',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const isResetting = ref(false)

function show(): void {
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function reset(): Promise<void> {
	if (isResetting.value) return

	isResetting.value = true
	try {
		await client.labrinth.moderation_internal.resetUser2fa(props.user.id)
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
