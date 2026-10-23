<template>
	<NewModal
		ref="modal"
		hide-header
		merge-header
		max-width="448px"
		:header="formatMessage(messages.title)"
		:on-hide="onHide"
	>
		<ServerInviteLinkContent
			:link="link"
			:loading="loading"
			:error="error"
			@retry="load"
			@copied="copied = true"
		/>
		<template #actions>
			<div class="flex justify-end">
				<Button v-if="copied" type="colored" color="brand" @click="hide">
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
import { createServerOnboardingInvite } from '#ui/components/flows/creation-flow-modal/server-onboarding-invite'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers'

import ServerInviteLinkContent from './ServerInviteLinkContent.vue'

const props = defineProps<{
	serverId: string
	worldId: string | null
	siteUrl: string
}>()

const client = injectModrinthClient()
const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const link = ref<string | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const copied = ref(false)
let run = 0

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

async function load() {
	const worldId = props.worldId
	if (!worldId) return
	const current = ++run
	loading.value = true
	error.value = null
	try {
		const next = await createServerOnboardingInvite(
			client,
			props.serverId,
			worldId,
			props.siteUrl,
			() => current === run,
		)
		if (current === run) link.value = next
	} catch (err) {
		if (current === run) error.value = err instanceof Error ? err.message : String(err)
	} finally {
		if (current === run) loading.value = false
	}
}

function show() {
	link.value = null
	copied.value = false
	modal.value?.show()
	void load()
}

function hide() {
	modal.value?.hide()
}

function onHide() {
	run++
	loading.value = false
}

defineExpose({ show, hide })
</script>
