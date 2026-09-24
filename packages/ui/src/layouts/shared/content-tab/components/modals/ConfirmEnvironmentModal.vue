<template>
	<NewModal
		ref="modal"
		:header="formatMessage(titleMessage)"
		fade="warning"
		max-width="500px"
		:on-hide="() => resolveConfirmation(false)"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(admonitionMessage)">
				{{ formatMessage(bodyMessage, { project }) }}
			</Admonition>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="orange" :disabled="actionDisabled" @click="confirm">
					<CheckIcon />
					{{ formatMessage(enabled ? commonMessages.enableButton : commonMessages.disableButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentSide } from '../../types'

const props = defineProps<{ actionDisabled?: boolean }>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	serverTitle: {
		id: 'content.confirm-environment.server-title',
		defaultMessage: 'Enable for your server?',
	},
	playerTitle: {
		id: 'content.confirm-environment.player-title',
		defaultMessage: 'Enable for players?',
	},
	disableServerTitle: {
		id: 'content.confirm-environment.disable-server-title',
		defaultMessage: 'Disable for your server?',
	},
	disablePlayerTitle: {
		id: 'content.confirm-environment.disable-player-title',
		defaultMessage: 'Disable for players?',
	},
	admonitionTitle: {
		id: 'content.confirm-environment.admonition-title',
		defaultMessage: 'This content may be incompatible',
	},
	requiredAdmonitionTitle: {
		id: 'content.confirm-environment.required-admonition-title',
		defaultMessage: 'This content is required here',
	},
	serverBody: {
		id: 'content.confirm-environment.server-body',
		defaultMessage:
			'{project} isn’t intended to run on a server. Enabling it could cause errors or prevent your server from starting.',
	},
	playerBody: {
		id: 'content.confirm-environment.player-body',
		defaultMessage:
			'{project} isn’t intended to run on players’ clients. Enabling it could cause errors or prevent their game from starting.',
	},
	singleplayerBody: {
		id: 'content.confirm-environment.singleplayer-body',
		defaultMessage:
			'{project} is intended for singleplayer. Enabling it on your server could cause errors or prevent your server from starting.',
	},
	disableServerBody: {
		id: 'content.confirm-environment.disable-server-body',
		defaultMessage:
			'{project} is required on the server. Disabling it may prevent your server from working correctly.',
	},
	disablePlayerBody: {
		id: 'content.confirm-environment.disable-player-body',
		defaultMessage:
			'{project} is required on players’ clients. Disabling it may prevent players from using this content correctly.',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const project = ref('')
const side = ref<ContentSide>('server')
const enabled = ref(true)
const singleplayer = ref(false)
let pendingConfirmation: ((confirmed: boolean) => void) | undefined
const titleMessage = computed(() => {
	if (enabled.value) return side.value === 'server' ? messages.serverTitle : messages.playerTitle
	return side.value === 'server' ? messages.disableServerTitle : messages.disablePlayerTitle
})
const admonitionMessage = computed(() =>
	enabled.value ? messages.admonitionTitle : messages.requiredAdmonitionTitle,
)
const bodyMessage = computed(() => {
	if (!enabled.value)
		return side.value === 'server' ? messages.disableServerBody : messages.disablePlayerBody
	if (side.value === 'player') return messages.playerBody
	return singleplayer.value ? messages.singleplayerBody : messages.serverBody
})

function resolveConfirmation(confirmed: boolean) {
	const resolve = pendingConfirmation
	pendingConfirmation = undefined
	resolve?.(confirmed)
}

function confirm() {
	if (props.actionDisabled) return
	resolveConfirmation(true)
	modal.value?.hide()
}

function show(
	name: string,
	targetSide: ContentSide,
	targetEnabled: boolean,
	isSingleplayer: boolean,
) {
	if (pendingConfirmation) return Promise.resolve(false)
	project.value = name
	side.value = targetSide
	enabled.value = targetEnabled
	singleplayer.value = isSingleplayer
	return new Promise<boolean>((resolve) => {
		pendingConfirmation = resolve
		modal.value?.show()
	})
}

onBeforeUnmount(() => resolveConfirmation(false))

defineExpose({ show })
</script>
