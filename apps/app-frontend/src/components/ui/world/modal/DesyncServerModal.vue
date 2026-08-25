<script setup lang="ts">
import { LinkIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import type { DesyncServerMode, ServerWorld } from '@/helpers/worlds'

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const server = ref<ServerWorld | null>(null)

const emit = defineEmits<{
	confirm: [server: ServerWorld, mode: DesyncServerMode]
}>()

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
		id: 'instance.worlds.desync-server.keep',
		defaultMessage: 'Keep',
	},
	remove: {
		id: 'instance.worlds.desync-server.remove',
		defaultMessage: 'Remove',
	},
})

function show(value: ServerWorld) {
	server.value = value
	modal.value?.show()
}

function confirm(mode: DesyncServerMode) {
	if (server.value) emit('confirm', server.value, mode)
	modal.value?.hide()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" max-width="540px">
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" class="whitespace-nowrap" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button class="whitespace-nowrap" @click="confirm('keep_in_other_instances')">
					<LinkIcon />
					{{ formatMessage(messages.keep) }}
				</Button>
				<Button
					type="colored"
					color="red"
					class="whitespace-nowrap"
					@click="confirm('remove_from_other_instances')"
				>
					<TrashIcon />
					{{ formatMessage(messages.remove) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
