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

import { injectSharedInstance } from '@/pages/instance/shared-instance-context'

import { injectInstanceSettings } from './instance-settings-context.ts'
import RelinkSharedInstanceModal from './relink-shared-instance/relink-shared-instance-modal.vue'

const { instance, offline } = injectInstanceSettings()
const { currentUserCanUseSharedInstances } = injectSharedInstance()
const auth = injectAuth()
const { formatMessage } = useVIntl()
const relinkModal = ref<InstanceType<typeof RelinkSharedInstanceModal>>()

const nonRelinkableLinkTypes = new Set([
	'shared_instance',
	'imported_modpack',
	'server_project',
	'server_project_modpack',
	'modrinth_hosting',
])

const canRelink = computed(
	() =>
		currentUserCanUseSharedInstances.value &&
		!instance.value.shared_instance &&
		!nonRelinkableLinkTypes.has(instance.value.link?.type ?? '') &&
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
			'Reconnect this instance to a shared instance it was unlinked from, using an invite link or ID.',
	},
})
</script>
