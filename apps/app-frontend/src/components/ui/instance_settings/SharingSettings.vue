<template>
	<div class="flex flex-col gap-6">
		<SharedInstancePublishModal
			ref="publishModal"
			:instance="instance"
			@state-change="publishState = $event"
		/>

		<div class="flex flex-col gap-3">
			<span class="text-lg font-semibold text-contrast">
				{{ formatMessage(messages.manualPushTitle) }}
			</span>
			<span class="text-primary">{{ formatMessage(messages.manualPushDescription) }}</span>
			<div>
				<ButtonStyled>
					<button class="!shadow-none" :disabled="isBusy" @click="reviewUpdate">
						<SpinnerIcon v-if="publishState !== 'idle'" class="animate-spin" />
						<UploadIcon v-else class="size-5" />
						{{ formatMessage(manualPushButtonMessage) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<SharedInstanceInstallationSettingsControls
			can-unpublish
			:busy="isBusy"
			:unpublishing="unpublishing"
			:unpublish="unpublishSharedInstance"
		/>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import SharedInstanceInstallationSettingsControls from '@/components/ui/shared-instances/SharedInstanceInstallationSettingsControls.vue'
import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import { unpublish_shared_instance } from '@/helpers/instance'
import { injectInstanceSettings } from '@/providers/instance-settings'

const { instance, offline, onUnlinked } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const publishState = ref<'idle' | 'reviewing' | 'publishing'>('idle')
const unpublishing = ref(false)
const isBusy = computed(
	() =>
		instance.value.install_stage !== 'installed' ||
		unpublishing.value ||
		publishState.value !== 'idle' ||
		!!offline,
)
const manualPushButtonMessage = computed(() => {
	if (publishState.value === 'publishing') return messages.pushingButton
	if (publishState.value === 'reviewing') return messages.reviewingButton
	return messages.manualPushButton
})

function reviewUpdate(event: MouseEvent) {
	publishModal.value?.show(event)
}

async function unpublishSharedInstance() {
	unpublishing.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		await queryClient.invalidateQueries({ queryKey: ['sharedInstanceUsers', instance.value.id] })
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		handleError(error)
	} finally {
		unpublishing.value = false
	}
}

const messages = defineMessages({
	manualPushTitle: {
		id: 'instance.settings.tabs.sharing.manual-push.title',
		defaultMessage: 'Manual push update',
	},
	manualPushDescription: {
		id: 'instance.settings.tabs.sharing.manual-push.description',
		defaultMessage:
			'Review the current instance changes and choose any config files you want to send to everyone using it.',
	},
	manualPushButton: {
		id: 'instance.settings.tabs.sharing.manual-push.button',
		defaultMessage: 'Manually push update',
	},
	reviewingButton: {
		id: 'instance.settings.tabs.sharing.manual-push.reviewing-button',
		defaultMessage: 'Reviewing',
	},
	pushingButton: {
		id: 'instance.settings.tabs.sharing.manual-push.pushing-button',
		defaultMessage: 'Pushing',
	},
})
</script>
