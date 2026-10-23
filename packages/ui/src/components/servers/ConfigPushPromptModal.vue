<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		max-width="500px"
		:on-hide="() => resolvePrompt('later')"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="info" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { path }) }}
			</Admonition>
			<p v-if="serverRunning || includesOtherChanges" class="m-0 text-primary">
				<template v-if="serverRunning">{{ formatMessage(messages.restartBody) }}</template>
				<template v-if="includesOtherChanges">
					{{ formatMessage(messages.otherChangesBody) }}
				</template>
			</p>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.later) }}
				</Button>
				<Button type="colored" color="blue" :disabled="actionDisabled" @click="confirm">
					<UploadIcon aria-hidden="true" />
					{{ formatMessage(messages.pushUpdate) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UploadIcon, XIcon } from '@modrinth/assets'
import { onBeforeUnmount, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

export type ConfigPushChoice = 'push' | 'later'

const props = defineProps<{
	actionDisabled?: boolean
	includesOtherChanges?: boolean
	serverRunning?: boolean
}>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	header: {
		id: 'servers.files.config-push.header',
		defaultMessage: 'Push config to players?',
	},
	admonitionHeader: {
		id: 'servers.files.config-push.admonition-header',
		defaultMessage: 'Share this config with players',
	},
	admonitionBody: {
		id: 'servers.files.config-push.admonition-body',
		defaultMessage:
			'You edited {path}. Push an update so players get this change next time they play.',
	},
	restartBody: {
		id: 'servers.files.config-push.restart-body',
		defaultMessage:
			'Pushing an update will restart your server, and players who are online will be disconnected.',
	},
	otherChangesBody: {
		id: 'servers.files.config-push.other-changes-body',
		defaultMessage: 'Your other unshared content changes will be included too.',
	},
	later: {
		id: 'servers.files.config-push.later',
		defaultMessage: 'Later',
	},
	pushUpdate: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const path = ref('')
let pendingPrompt: ((choice: ConfigPushChoice) => void) | undefined

function resolvePrompt(choice: ConfigPushChoice) {
	const resolve = pendingPrompt
	pendingPrompt = undefined
	resolve?.(choice)
}

function confirm() {
	if (props.actionDisabled) return
	resolvePrompt('push')
	modal.value?.hide()
}

/** Asks whether to push `configPath`. Resolves `undefined` if a prompt is already open. */
function show(configPath: string): Promise<ConfigPushChoice | undefined> {
	if (pendingPrompt) return Promise.resolve(undefined)
	path.value = configPath
	return new Promise((resolve) => {
		pendingPrompt = resolve
		modal.value?.show()
	})
}

function hide() {
	modal.value?.hide()
}

onBeforeUnmount(() => resolvePrompt('later'))

defineExpose({ show, hide })
</script>
