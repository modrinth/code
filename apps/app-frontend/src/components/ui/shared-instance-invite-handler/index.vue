<template>
	<SharedInstanceInviteLinkModal ref="inviteLinkModal" :process-invite="installFromInviteId" />
	<ModrinthAccountRequiredModal ref="accountRequiredModal" :request-auth="requestAuth" />
	<InstallToPlayModal ref="installModal" />
</template>

<script setup lang="ts">
import { injectAuth } from '@modrinth/ui'
import { nextTick, ref } from 'vue'

import InstallToPlayModal from '@/components/ui/modal/InstallToPlayModal.vue'
import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import type { ModrinthAuthFlow } from '@/helpers/mr_auth'

import SharedInstanceInviteLinkModal from './shared-instance-invite-link-modal.vue'
import type { SharedInstanceInviteHandler } from './shared-instance-invite-types'
import { useSharedInstanceInviteHandler } from './use-shared-instance-invite-handler'

const auth = injectAuth()
const inviteLinkModal = ref<InstanceType<typeof SharedInstanceInviteLinkModal>>()
const installModal = ref<InstanceType<typeof InstallToPlayModal>>()
const accountRequiredModal = ref<InstanceType<typeof ModrinthAccountRequiredModal>>()
const { handleNotification, installFromInviteId } = useSharedInstanceInviteHandler(
	installModal,
	accountRequiredModal,
)

async function requestAuth(flow: ModrinthAuthFlow) {
	await auth.requestSignIn('', flow)
	await nextTick()
	return !!auth.session_token.value
}

function showManualInviteLinkModal(event?: MouseEvent) {
	inviteLinkModal.value?.show(event)
}

defineExpose<SharedInstanceInviteHandler>({
	handleNotification,
	installFromInviteId,
	showManualInviteLinkModal,
})
</script>
