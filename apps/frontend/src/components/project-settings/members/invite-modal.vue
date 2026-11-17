<script setup lang="ts">
import { UserPlusIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectModrinthClient,
	injectProjectPageContext,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { ref, useTemplateRef } from 'vue'

import { useMemberErrorNotification } from './use-member-error-notification'

const props = defineProps<{ canInvite: boolean; title: string }>()
const { formatMessage } = useVIntl()
const { projectV2: project, invalidate } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const notifyError = useMemberErrorNotification()
const loading = useLoading()
const modal = useTemplateRef('modal')
const username = ref('')

const messages = defineMessages({
	description: {
		id: 'project.settings.members.invite.description',
		defaultMessage:
			"Enter the Modrinth username of the person you'd like to invite to this project.",
	},
	username: { id: 'project.settings.members.invite.username', defaultMessage: 'Username' },
	invite: { id: 'project.settings.members.invite.button', defaultMessage: 'Invite' },
})

const { mutate: invite, isPending: inviting } = useMutation({
	mutationFn: async ({ team, username }: { team: string; username: string }) => {
		const user = await labrinth.users_v2.get(username)
		await labrinth.teams_v2.addMember(team, { user_id: user.id.trim() })
	},
	onSuccess: async () => {
		username.value = ''
		modal.value?.hide()
		await invalidate()
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

function submit() {
	if (!props.canInvite || inviting.value || !username.value.trim()) return
	loading.value = true
	invite({ team: project.value.team, username: username.value })
}

function show(event: MouseEvent) {
	modal.value?.show(event)
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="title" width="450px">
		<p>{{ formatMessage(messages.description) }}</p>
		<form class="flex gap-2" @submit.prevent="submit">
			<Input
				id="username"
				v-model="username"
				:placeholder="formatMessage(messages.username)"
				:aria-label="formatMessage(messages.username)"
				:disabled="!canInvite || inviting"
				wrapper-class="min-w-0 flex-1"
			/>
			<Button
				type="colored"
				color="brand"
				native-type="submit"
				:disabled="!canInvite || inviting || !username.trim()"
			>
				<UserPlusIcon /> {{ formatMessage(messages.invite) }}
			</Button>
		</form>
	</NewModal>
</template>
