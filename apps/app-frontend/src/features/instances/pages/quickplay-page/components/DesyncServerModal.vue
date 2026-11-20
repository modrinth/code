<script setup lang="ts">
import { LinkIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref, shallowRef } from 'vue'

import type { DesyncServerMode, ServerWorld } from '@/platform/app-lib/worlds/types'

const emit = defineEmits<{
	confirm: [server: ServerWorld, mode: DesyncServerMode]
}>()

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const server = shallowRef<ServerWorld | null>(null)

const messages = defineMessages({
	title: {
		id: 'instance.worlds.desync-server.title',
		defaultMessage: 'Desync server',
	},
	description: {
		id: 'instance.worlds.desync-server.description',
		defaultMessage:
			'Do you want to keep this server in other synced instances, or remove it from them?',
	},
	keep: {
		id: 'instance.worlds.desync-server.keep-elsewhere',
		defaultMessage: 'Keep in other instances',
	},
	remove: {
		id: 'instance.worlds.desync-server.remove-elsewhere',
		defaultMessage: 'Remove from other instances',
	},
})

function show(value: ServerWorld, event?: MouseEvent) {
	server.value = value
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function confirm(mode: DesyncServerMode) {
	if (server.value) emit('confirm', server.value, mode)
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		width="min(540px, calc(95vw - 10rem))"
	>
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button @click="confirm('keep_in_other_instances')">
					<LinkIcon aria-hidden="true" />
					{{ formatMessage(messages.keep) }}
				</Button>
				<Button type="colored" color="red" @click="confirm('remove_from_other_instances')">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.remove) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
