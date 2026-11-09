<template>
	<ModrinthAccountRequiredModal ref="accountModal" :request-auth="requestAuth" />
	<SharedInstanceInstallModal ref="installModal" />
	<ContentDiffModal
		ref="updateModal"
		:header="formatMessage(messages.update)"
		:admonition-header="formatMessage(messages.update)"
		:description="formatMessage(messages.updateDescription)"
		:diffs="updateDiffs"
		:confirm-label="formatMessage(messages.update)"
		:confirm-icon="DownloadIcon"
		show-external-warnings
		@confirm="confirmUpdate"
	/>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import { ContentDiffModal, defineMessages, injectAuth, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import { get as getCredentials, type ModrinthAuthFlow } from '@/helpers/mr_auth'

import { type ServerPlayModals, useServerPlay } from './use-server-play'

const auth = injectAuth()
const { formatMessage } = useVIntl()
const accountModal = ref<InstanceType<typeof ModrinthAccountRequiredModal>>()
const installModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const updateModal = ref<InstanceType<typeof ContentDiffModal>>()

const modals = computed<ServerPlayModals>(() => ({
	showAccountRequired: () => accountModal.value?.show(),
	showInstallPreview: (preview, confirm) => installModal.value?.show(preview, confirm),
	showUpdatePreview: () => updateModal.value?.show(),
	hideAll: () => {
		installModal.value?.hide()
		updateModal.value?.hide()
	},
}))

const { play, confirmUpdate, updateDiffs } = useServerPlay(modals)

async function requestAuth(flow: ModrinthAuthFlow) {
	await auth.requestSignIn('', flow, { showModal: false })
	return !!(await getCredentials())
}

const messages = defineMessages({
	update: { id: 'hosting.play.update-to-play', defaultMessage: 'Update to play' },
	updateDescription: {
		id: 'hosting.play.update-description',
		defaultMessage: 'Update this instance to the server’s latest shared content before joining.',
	},
})

defineExpose({ play })
</script>
