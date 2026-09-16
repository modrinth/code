<script setup lang="ts">
import { SpinnerIcon, UnlinkIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { Button, Toggle } from '#ui/components/base'
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'

import ActiveInvites from './components/active-invites.vue'
import RevokeInviteModal from './components/revoke-invite-modal.vue'
import UnpublishModal from './components/unpublish-modal.vue'
import { injectSharingSettings } from './providers/sharing-settings'

defineProps<{
	siteUrl: string
	unpublishDescription: MessageDescriptor
	unpublishConfirmation: MessageDescriptor
}>()

const ctx = injectSharingSettings()
const { formatMessage } = useVIntl()
const { reviewChangesBeforePlaying } = ctx
const revokeModal = ref<InstanceType<typeof RevokeInviteModal>>()
const unpublishModal = ref<InstanceType<typeof UnpublishModal>>()
const revokeTarget = ref<{ key: string; inviteId: string }>()
const unpublishTarget = ref<string>()
const revokingId = ref<string>()
const unpublishing = ref(false)
const busy = computed(
	() => ctx.busy.value || !ctx.targetKey.value || !!revokingId.value || unpublishing.value,
)

function showRevoke(inviteId: string) {
	if (busy.value || !ctx.targetKey.value) return
	revokeTarget.value = { key: ctx.targetKey.value, inviteId }
	revokeModal.value?.show()
}

async function revokeInvite() {
	const target = revokeTarget.value
	if (!target || busy.value || target.key !== ctx.targetKey.value) return
	revokeModal.value?.hide()
	revokingId.value = target.inviteId
	try {
		await ctx.revokeInvite(target.inviteId, target.key)
	} catch (error) {
		ctx.onError(error)
	} finally {
		revokingId.value = undefined
	}
}

function showUnpublish() {
	if (busy.value || !ctx.targetKey.value) return
	unpublishTarget.value = ctx.targetKey.value
	unpublishModal.value?.show()
}

async function unpublish() {
	const target = unpublishTarget.value
	if (!target || busy.value || target !== ctx.targetKey.value) return
	unpublishing.value = true
	try {
		await ctx.unpublish(target)
		if (unpublishTarget.value === target) unpublishModal.value?.hide()
	} catch (error) {
		ctx.onError(error)
	} finally {
		unpublishing.value = false
	}
}

watch(
	ctx.targetKey,
	() => {
		revokeModal.value?.hide()
		unpublishModal.value?.hide()
		revokeTarget.value = undefined
		unpublishTarget.value = undefined
	},
	{ flush: 'sync' },
)

const messages = defineMessages({
	reviewChangesBeforePlaying: {
		id: 'server.settings.sharing.review-changes-before-playing',
		defaultMessage: 'Review changes before playing',
	},
	reviewChangesBeforePlayingDescription: {
		id: 'server.settings.sharing.review-changes-before-playing-description',
		defaultMessage: 'Review content updates before sharing them with players. Saved on this device.',
	},
	unpublishTitle: {
		id: 'installation-settings.shared-instance.title',
		defaultMessage: 'Unpublish instance',
	},
	unpublish: {
		id: 'installation-settings.shared-instance.unpublish-button',
		defaultMessage: 'Unpublish shared instance',
	},
	unpublishing: {
		id: 'installation-settings.shared-instance.unpublishing-button',
		defaultMessage: 'Unpublishing...',
	},
})
</script>

<template>
	<div class="flex flex-col gap-8">
		<div
			v-if="reviewChangesBeforePlaying !== undefined"
			class="flex items-center justify-between gap-2"
		>
			<label for="review-changes-before-playing" class="flex flex-col gap-1">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.reviewChangesBeforePlaying) }}
				</span>
				<span>{{ formatMessage(messages.reviewChangesBeforePlayingDescription) }}</span>
			</label>
			<Toggle
				id="review-changes-before-playing"
				v-model="reviewChangesBeforePlaying"
				class="flex-none"
			/>
		</div>
		<ActiveInvites :site-url="siteUrl" :busy="busy" :revoking-id="revokingId" @revoke="showRevoke" />
		<section class="flex flex-col gap-2.5">
			<h3 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.unpublishTitle) }}
			</h3>
			<div>
				<Button type="colored" color="orange" :disabled="busy" @click="showUnpublish">
					<SpinnerIcon v-if="unpublishing" class="animate-spin" />
					<UnlinkIcon v-else />
					{{ formatMessage(unpublishing ? messages.unpublishing : messages.unpublish) }}
				</Button>
			</div>
			<p class="m-0 text-primary">{{ formatMessage(unpublishDescription) }}</p>
		</section>
	</div>
	<RevokeInviteModal
		ref="revokeModal"
		:invite-id="revokeTarget?.inviteId ?? ''"
		:busy="busy"
		@confirm="revokeInvite"
	/>
	<UnpublishModal
		ref="unpublishModal"
		:busy="busy"
		:unpublishing="unpublishing"
		:description="unpublishConfirmation"
		@confirm="unpublish"
	/>
</template>
