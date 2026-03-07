<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<span class="text-primary">
				<IntlFormatted
					:message-id="backupLink ? messages.warningBodyBackup : messages.warningBody"
				>
					<template #backup="{ children }">
						<RouterLink :to="backupLink!" class="text-link hover:underline" @click="modal?.hide()">
							<component :is="() => children" />
						</RouterLink>
					</template>
				</IntlFormatted>
			</span>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<DownloadIcon />
						{{ formatMessage(messages.reinstallButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'instance.confirm-reinstall.header',
		defaultMessage: 'Reinstall modpack',
	},
	admonitionHeader: {
		id: 'instance.confirm-reinstall.admonition-header',
		defaultMessage: 'Reinstallation warning',
	},
	admonitionBody: {
		id: 'instance.confirm-reinstall.admonition-body',
		defaultMessage:
			'Reinstalling will reset all installed or modified content to what is provided by the modpack, removing any mods or content you have added on top of the original installation.',
	},
	warningBody: {
		id: 'instance.confirm-reinstall.warning-body',
		defaultMessage:
			'We recommend creating a backup before proceeding. If your worlds depend on additional installed content, reinstalling will likely break them.',
	},
	warningBodyBackup: {
		id: 'instance.confirm-reinstall.warning-body-backup',
		defaultMessage:
			'We recommend creating a <backup>backup</backup> before proceeding. If your worlds depend on additional installed content, reinstalling will likely break them.',
	},
	reinstallButton: {
		id: 'instance.confirm-reinstall.reinstall-button',
		defaultMessage: 'Reinstall modpack',
	},
})

defineProps<{
	backupLink?: string
}>()

const emit = defineEmits<{
	(e: 'reinstall'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('reinstall')
}

defineExpose({
	show,
})
</script>
