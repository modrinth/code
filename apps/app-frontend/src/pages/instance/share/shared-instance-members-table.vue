<template>
	<InvitedPlayersTableLayout
		:rows="rows"
		:can-manage="!actionsLocked"
		@remove="management.remove"
	>
		<template v-if="!actionsLocked" #toolbar-actions>
			<Button
				type="outlined"
				size="lg"
				class="shrink-0 !border"
				:disabled="pushUpdateDisabled"
				@click="management.pushUpdate($event)"
			>
				<SpinnerIcon v-if="pushUpdatePending" class="animate-spin" aria-hidden="true" />
				<UploadIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.pushUpdate) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				size="lg"
				class="shrink-0"
				:disabled="invitePending || inviteDisabled"
				@click="management.invite($event)"
			>
				<SpinnerIcon v-if="invitePending" class="animate-spin" aria-hidden="true" />
				<UserPlusIcon v-else aria-hidden="true" />
				Invite friends
			</Button>
		</template>
	</InvitedPlayersTableLayout>
</template>

<script setup lang="ts">
import { SpinnerIcon, UploadIcon, UserPlusIcon } from '@modrinth/assets'
import { Button, defineMessages, InvitedPlayersTableLayout, useVIntl } from '@modrinth/ui'

import { injectSharedInstanceManagement } from './shared-instance-management-context'

const management = injectSharedInstanceManagement()
const {
	rows,
	actionsLocked,
	inviteDisabled,
	invitePending,
	pushUpdateDisabled,
	pushUpdatePending,
} = management
const { formatMessage } = useVIntl()
const messages = defineMessages({
	pushUpdate: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
})
</script>
