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
			<div class="flex justify-end gap-2">
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
					<button @click="handleInstallAnyway">
						<DownloadIcon />
						{{ formatMessage(messages.installAnyway) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
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
		id: 'app.instance.shared-instance-already-installed.header',
		defaultMessage: 'Shared instance already installed',
	},
	body: {
		id: 'app.instance.shared-instance-already-installed.body',
		defaultMessage:
			'This shared instance is already installed as <bold>{instanceName}</bold>. Are you sure you want to install another copy?',
	},
	instance: {
		id: 'app.instance.shared-instance-already-installed.instance',
		defaultMessage: 'Instance',
	},
	installAnyway: {
		id: 'app.instance.shared-instance-already-installed.install-anyway',
		defaultMessage: 'Install anyway',
	},
})

const emit = defineEmits<{
	(e: 'cancel'): void
	(e: 'go-to-instance'): void
	(e: 'install-anyway'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const instanceName = ref('')

function show(name: string) {
	instanceName.value = name
	modal.value?.show()
}

function handleCancel() {
	modal.value?.hide()
	emit('cancel')
}

function handleGoToInstance() {
	modal.value?.hide()
	emit('go-to-instance')
}

function handleInstallAnyway() {
	modal.value?.hide()
	emit('install-anyway')
}

defineExpose({
	show,
})
</script>
