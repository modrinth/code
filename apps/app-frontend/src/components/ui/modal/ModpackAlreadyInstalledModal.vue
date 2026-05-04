<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="500px">
		<p class="m-0 text-secondary">
			<IntlFormatted :message-id="messages.body" :values="{ instanceName }">
				<template #bold="{ children }">
					<span class="font-medium text-contrast"><component :is="() => children" /></span>
				</template>
			</IntlFormatted>
		</p>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="handleGoToInstance">
						{{ formatMessage(messages.instance) }}
						<RightArrowIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="handleCreateAnyway">
						<PlusIcon />
						{{ formatMessage(messages.create) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
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
	body: {
		id: 'app.instance.modpack-already-installed.body',
		defaultMessage:
			'This modpack is already installed in the <bold>{instanceName}</bold> instance. Are you sure you want to duplicate it?',
	},
	instance: {
		id: 'app.instance.modpack-already-installed.instance',
		defaultMessage: 'Instance',
	},
	create: {
		id: 'app.instance.modpack-already-installed.create',
		defaultMessage: 'Create',
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

function handleCancel() {
	modal.value?.hide()
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
