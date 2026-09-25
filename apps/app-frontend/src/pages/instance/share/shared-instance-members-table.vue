<template>
	<InvitedPlayersTableLayout
		:rows="rows"
		:can-manage="!actionsLocked"
		:push-update-disabled="pushUpdateDisabled"
		:push-update-pending="pushUpdatePending"
		:invite-label="formatMessage(messages.inviteFriends)"
		:invite-disabled="inviteDisabled"
		:invite-pending="invitePending"
		@push-update="management.pushUpdate($event)"
		@invite="management.invite($event)"
		@remove="management.remove"
	/>
</template>

<script setup lang="ts">
import { defineMessages, InvitedPlayersTableLayout, useVIntl } from '@modrinth/ui'

import { injectSharedInstanceManagement } from './shared-instance-management-context'

const management = injectSharedInstanceManagement()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	inviteFriends: { id: 'app.instance.share.invite-friends', defaultMessage: 'Invite friends' },
})
const {
	rows,
	actionsLocked,
	inviteDisabled,
	invitePending,
	pushUpdateDisabled,
	pushUpdatePending,
} = management
</script>
