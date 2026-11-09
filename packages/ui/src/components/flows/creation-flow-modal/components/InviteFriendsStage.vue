<template>
	<ServerInviteLinkContent
		:link="ctx.invite.link.value"
		:loading="ctx.invite.loading.value"
		:error="ctx.invite.error.value"
		@retry="ctx.invite.retry()"
		@copied="ctx.invite.copied.value = true"
	/>
</template>

<script setup lang="ts">
import { injectCreationFlowContext } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import ServerInviteLinkContent from '#ui/components/sharing/server-invite-link/ServerInviteLinkContent.vue'

const ctx = injectCreationFlowContext()
</script>
