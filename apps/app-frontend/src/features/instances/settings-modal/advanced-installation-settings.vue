<template>
	<Accordion v-if="canRelink" divider>
		<template #title>
			<span class="text-lg font-semibold">{{ formatMessage(messages.advancedTitle) }}</span>
		</template>
		<div class="flex flex-col gap-2.5">
			<span class="text-lg font-semibold text-contrast">{{
				formatMessage(messages.relinkTitle)
			}}</span>
			<div>
				<Button :disabled="!!offline || !auth.user.value" @click="relinkModal?.show($event)">
					<LinkIcon class="size-5" />
					{{ formatMessage(messages.relinkButton) }}
				</Button>
			</div>
			<span class="text-primary">{{ formatMessage(messages.relinkDescription) }}</span>
		</div>
	</Accordion>
	<RelinkSharedInstanceModal ref="relinkModal" />
</template>

<script setup lang="ts">
import { LinkIcon } from '@modrinth/assets'
import { Accordion, Button, defineMessages, injectAuth, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectInstanceSettings } from './instance-settings-context.ts'
import RelinkSharedInstanceModal from './relink-shared-instance/relink-shared-instance-modal.vue'

const { instance, offline } = injectInstanceSettings()
const auth = injectAuth()
const { formatMessage } = useVIntl()
const relinkModal = ref<InstanceType<typeof RelinkSharedInstanceModal>>()

const canRelink = computed(
	() =>
		!instance.value.shared_instance &&
		instance.value.link?.type !== 'shared_instance' &&
		instance.value.link?.type !== 'imported_modpack' &&
		!instance.value.quarantined &&
		instance.value.install_stage === 'installed',
)

const messages = defineMessages({
	advancedTitle: {
		id: 'installation-settings.advanced.title',
		defaultMessage: 'Advanced',
	},
	relinkTitle: {
		id: 'installation-settings.relink-shared-instance.title',
		defaultMessage: 'Relink shared instance',
	},
	relinkButton: {
		id: 'installation-settings.relink-shared-instance.button',
		defaultMessage: 'Relink shared instance',
	},
	relinkDescription: {
		id: 'installation-settings.relink-shared-instance.description',
		defaultMessage:
			'Link this instance back to a shared instance you own using its ID or an invite link.',
	},
})
</script>
