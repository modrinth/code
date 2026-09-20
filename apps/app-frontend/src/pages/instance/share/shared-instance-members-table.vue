<template>
	<InvitedPlayersTableLayout
		:rows="rows"
		:can-manage="!actionsLocked"
		:push-update-disabled="pushUpdateDisabled"
		:push-update-pending="pushUpdatePending"
		invite-label="Invite friends"
		:invite-disabled="inviteDisabled"
		:invite-pending="invitePending"
		@push-update="management.pushUpdate($event)"
		@invite="management.invite($event)"
		@remove="management.remove"
	/>
</template>

<script setup lang="ts">
import { InvitedPlayersTableLayout } from '@modrinth/ui'

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
</script>
