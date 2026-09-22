<template>
	<CreationFlowModal
		ref="modalRef"
		type="server-onboarding"
		@after-hide="onAfterHide"
		@after-show="onAfterShow"
	/>
</template>

<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'

import { useDismissServerIntro } from '#ui/composables/server-onboarding'
import { injectServerOnboardingInviteFlow } from '#ui/providers/server-onboarding-invite'

import CreationFlowModal from '../flows/creation-flow-modal/index.vue'

const inviteFlow = injectServerOnboardingInviteFlow()
const dismissServerIntro = useDismissServerIntro()
const modalRef = useTemplateRef<InstanceType<typeof CreationFlowModal>>('modalRef')
let resolveShown: (() => void) | null = null
let modalHidden = false
let introFinished = false

function clearFinishedInvite() {
	if (modalHidden && introFinished) inviteFlow.clear()
}

function onAfterHide() {
	modalHidden = true
	clearFinishedInvite()
}

function onAfterShow() {
	resolveShown?.()
	resolveShown = null
}

watch(
	() => inviteFlow.request.value,
	async (request) => {
		if (!request) return
		modalHidden = false
		introFinished = false
		try {
			await nextTick()
			if (!modalRef.value) throw new Error('Invite modal is unavailable')
			const shown = new Promise<void>((resolve) => {
				resolveShown = resolve
			})
			await modalRef.value.show()
			modalRef.value.ctx.showInvite(
				request.serverId,
				request.worldId,
				request.siteUrl,
				async () => {
					try {
						await dismissServerIntro.mutateAsync(request.serverId)
					} finally {
						introFinished = true
						clearFinishedInvite()
					}
				},
			)
			await shown
			inviteFlow.markShown()
		} catch (error) {
			resolveShown = null
			inviteFlow.fail(error)
		}
	},
	{ immediate: true },
)
</script>
