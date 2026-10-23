<template>
	<ServerInviteLinkContent
		:link="ctx.inviteLink.value"
		:loading="ctx.inviteLoading.value"
		:error="ctx.inviteError.value"
		@retry="ctx.retryInvite()"
		@copied="ctx.inviteCopied.value = true"
	/>
</template>

<script setup lang="ts">
import { injectCreationFlowContext } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import ServerInviteLinkContent from '#ui/components/sharing/server-invite-link/ServerInviteLinkContent.vue'

const ctx = injectCreationFlowContext()
</script>
