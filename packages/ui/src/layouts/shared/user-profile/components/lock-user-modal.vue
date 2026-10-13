<template>
	<NewModal
		ref="modal"
		:header="`Lock ${user.username}`"
		:closable="!isLocking"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="critical" header="This account will become locked">
				{{ user.username }} will be signed out on every device and will not be able to login until
				their account is unlocked.
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="lock-user-reason">Reason</label>
				<Textarea
					id="lock-user-reason"
					v-model="reason"
					placeholder="Shown to the user and other moderators"
					:disabled="isLocking"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isLocking" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					native-type="button"
					:disabled="!reason.trim() || isLocking"
					@click="lock"
				>
					<SpinnerIcon v-if="isLocking" class="animate-spin" />
					<LockIcon v-else />
					Lock account
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LockIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Textarea from '#ui/components/base/inputs/Textarea.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables'
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

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const reason = ref('')
const isLocking = ref(false)

function show(): void {
	reason.value = ''
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function lock(): Promise<void> {
	if (!reason.value.trim() || isLocking.value) return

	isLocking.value = true
	try {
		await client.labrinth.moderation_internal.lockUser(props.user.id, reason.value.trim())
		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		hide()
		notificationManager.addNotification({
			type: 'success',
			title: 'Account locked',
			text: `${props.user.username}'s account has been locked.`,
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: 'Failed to lock account',
			text: 'An error occurred while locking this account. Please try again.',
		})
	} finally {
		isLocking.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
