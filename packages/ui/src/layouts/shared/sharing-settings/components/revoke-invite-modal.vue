<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="500px">
		<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
			<IntlFormatted :message-id="messages.admonitionBody" :values="{ code: `/${inviteId}` }">
				<template #monospace="{ children }">
					<code class="font-mono"><component :is="() => children" /></code>
				</template>
			</IntlFormatted>
		</Admonition>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" :disabled="busy" @click="$emit('confirm')">
					<XIcon />
					{{ formatMessage(messages.revokeButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Admonition, Button } from '#ui/components/base'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

defineProps<{ inviteId: string; busy: boolean }>()
defineEmits<{ confirm: [] }>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()

const messages = defineMessages({
	header: {
		id: 'instance.settings.sharing.revoke-invite.header',
		defaultMessage: 'Revoke invite',
	},
	admonitionHeader: {
		id: 'instance.settings.sharing.revoke-invite.admonition-header',
		defaultMessage: 'This action cannot be undone',
	},
	admonitionBody: {
		id: 'instance.settings.sharing.revoke-invite.admonition-body',
		defaultMessage:
			'The invite link <monospace>{code}</monospace> will stop working immediately. People who already joined will keep access.',
	},
	revokeButton: {
		id: 'instance.settings.sharing.revoke-invite.confirm',
		defaultMessage: 'Revoke invite',
	},
})

defineExpose({
	show: () => modal.value?.show(),
	hide: () => modal.value?.hide(),
})
</script>
