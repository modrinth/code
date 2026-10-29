<template>
	<NewModal
		ref="modal"
		hide-header
		merge-header
		max-width="448px"
		:header="formatMessage(messages.title)"
		:on-hide="invite.reset"
	>
		<ServerInviteLinkContent
			:link="invite.link.value"
			:loading="invite.loading.value"
			:error="invite.error.value"
			@retry="invite.retry()"
			@copied="invite.copied.value = true"
		/>
		<template #actions>
			<div class="flex justify-end">
				<Button v-if="invite.copied.value" type="colored" color="brand" @click="hide">
					<CheckIcon aria-hidden="true" />
					{{ formatMessage(messages.done) }}
				</Button>
				<Button v-else @click="hide">
					<CheckIcon aria-hidden="true" />
					{{ formatMessage(messages.done) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerInviteLink } from '#ui/composables/server-sharing'

import ServerInviteLinkContent from './ServerInviteLinkContent.vue'

const props = defineProps<{
	serverId: string
	worldId: string | null
	siteUrl: string
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const invite = useServerInviteLink()

const messages = defineMessages({
	title: {
		id: 'servers.setup.onboarding.invite.title',
		defaultMessage: 'Invite friends to join',
	},
	done: {
		id: 'servers.setup.onboarding.invite.done',
		defaultMessage: 'Done',
	},
})

function show() {
	const worldId = props.worldId
	modal.value?.show()
	if (worldId) invite.start({ serverId: props.serverId, worldId, siteUrl: props.siteUrl })
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
