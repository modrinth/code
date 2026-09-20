<template>
	<NewModal
		ref="modal"
		:header="formatMessage(side === 'server' ? messages.serverTitle : messages.playerTitle)"
		fade="warning"
		max-width="500px"
		:on-hide="() => resolveConfirmation(false)"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionTitle)">
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
					{{ formatMessage(commonMessages.enableButton) }}
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
	admonitionTitle: {
		id: 'content.confirm-environment.admonition-title',
		defaultMessage: 'This content may be incompatible',
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
})

const modal = ref<InstanceType<typeof NewModal>>()
const project = ref('')
const side = ref<ContentSide>('server')
const singleplayer = ref(false)
let pendingConfirmation: ((confirmed: boolean) => void) | undefined
const bodyMessage = computed(() =>
	side.value === 'player'
		? messages.playerBody
		: singleplayer.value
			? messages.singleplayerBody
			: messages.serverBody,
)

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

function show(name: string, targetSide: ContentSide, isSingleplayer: boolean) {
	if (pendingConfirmation) return Promise.resolve(false)
	project.value = name
	side.value = targetSide
	singleplayer.value = isSingleplayer
	return new Promise<boolean>((resolve) => {
		pendingConfirmation = resolve
		modal.value?.show()
	})
}

onBeforeUnmount(() => resolveConfirmation(false))

defineExpose({ show })
</script>
