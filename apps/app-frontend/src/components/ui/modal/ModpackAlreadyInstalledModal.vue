<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="warning"
		max-width="500px"
	>
		<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
			{{ formatMessage(messages.admonitionBody, { instanceName }) }}
		</Admonition>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="handleGoToInstance">
						<ExternalIcon />
						{{ formatMessage(messages.goToInstance) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="handleCreateAnyway">
						<PlusIcon />
						{{ formatMessage(messages.createAnyway) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { ExternalIcon, PlusIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'app.instance.modpack-already-installed.header',
		defaultMessage: 'Modpack already installed',
	},
	admonitionHeader: {
		id: 'app.instance.modpack-already-installed.admonition-header',
		defaultMessage: 'Duplicate modpack',
	},
	admonitionBody: {
		id: 'app.instance.modpack-already-installed.admonition-body',
		defaultMessage:
			'This modpack is already installed in the "{instanceName}" instance.',
	},
	goToInstance: {
		id: 'app.instance.modpack-already-installed.go-to-instance',
		defaultMessage: 'Go to instance',
	},
	createAnyway: {
		id: 'app.instance.modpack-already-installed.create-anyway',
		defaultMessage: 'Create anyway',
	},
})

const emit = defineEmits<{
	(e: 'go-to-instance', instancePath: string): void
	(e: 'create-anyway'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const instanceName = ref('')
const instancePath = ref('')

function show(name: string, path: string) {
	instanceName.value = name
	instancePath.value = path
	modal.value?.show()
}

function handleGoToInstance() {
	modal.value?.hide()
	emit('go-to-instance', instancePath.value)
}

function handleCreateAnyway() {
	modal.value?.hide()
	emit('create-anyway')
}

defineExpose({
	show,
})
</script>
