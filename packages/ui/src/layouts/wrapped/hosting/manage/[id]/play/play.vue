<template>
	<div class="flex flex-col gap-6">
		<Admonition v-if="needsUpdate && canSetup" type="warning" :header="formatMessage(messages.unpublished)" inline-actions>
			{{ formatMessage(messages.unpublishedBody) }}
			<template #actions>
				<Button :disabled="actionsLocked" @click="showPreview()">
					<SpinnerIcon v-if="previewQuery.isFetching.value || pendingAction === 'push'" class="animate-spin" />
					<UploadIcon v-else />
					{{ formatMessage(messages.pushUpdate) }}
				</Button>
			</template>
		</Admonition>
		<ServerPlayCard
			:address="serverAddress"
			:disabled="actionsLocked || !worldId || (!canSetup && !sharedInstanceId)"
			:can-invite="canSetup"
			:pending-action="pendingAction"
			@play="perform('play')"
			@invite="perform('invite')"
			@download="perform('download')"
		/>
		<section class="flex flex-col gap-3">
			<h1 class="m-0 text-2xl font-semibold text-contrast">{{ formatMessage(messages.invitedPlayersTitle) }}</h1>
			<div v-if="players.members.isLoading.value" class="flex justify-center p-8" role="status" :aria-label="formatMessage(messages.loading)">
				<SpinnerIcon class="animate-spin" />
			</div>
			<Admonition v-else-if="players.members.isError.value" type="critical" :header="formatMessage(messages.playersError)">
				<Button @click="players.members.refetch()">{{ formatMessage(messages.retry) }}</Button>
			</Admonition>
			<ServerPlayersTable v-else :rows="players.rows.value" :can-manage="canSetup" :disabled="players.membershipMutation.isPending.value || actionsLocked" @remove="confirmRemove" @open-actions="confirmRemove" />
		</section>
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="formatMessage(messages.inviteHeader, { name: server.name })"
			:friends="players.candidates.value"
			:search-users="players.search"
			:link="players.link.value ? `${siteUrl}/share/${encodeURIComponent(players.link.value.id)}` : undefined"
			:link-expires-at="players.link.value?.expiration"
			:link-max-uses="players.link.value?.max_uses"
			:link-max-uses-limit="players.remaining.value"
			:update-invite-link="updateInviteLink"
			:can-invite="canSetup && !actionsLocked && !players.membershipMutation.isPending.value && players.remaining.value > 0"
			:invite-disabled-message="formatMessage(messages.invitesUnavailable)"
			@invite="invitePlayer"
			@cancel="(user) => changeMember(user.id, true)"
		/>
		<ContentDiffModal
			ref="diffModal"
			:header="formatMessage(messages.pushUpdate)"
			:admonition-header="formatMessage(messages.shareChanges)"
			:description="formatMessage(messages.shareChangesBody)"
			:diffs="previewQuery.data.value?.items ?? []"
			:confirm-label="formatMessage(previewAction === 'play' ? messages.pushAndPlay : messages.pushUpdate)"
			:confirm-icon="UploadIcon"
			:confirm-disabled="actionsLocked || previewQuery.isError.value || !previewQuery.data.value"
			:added-label="formatMessage(messages.added)"
			:removed-label="formatMessage(messages.removed)"
			@confirm="perform(previewAction, true)"
			@cancel="previewOpen = false"
		>
			<template #additional-content>
				<p v-if="previewQuery.isFetching.value" class="m-0 flex items-center gap-2"><SpinnerIcon class="animate-spin" />{{ formatMessage(messages.refreshingPreview) }}</p>
				<Admonition v-else-if="previewQuery.isError.value" type="critical" :header="formatMessage(messages.previewError)">
					<Button @click="previewQuery.refetch()">{{ formatMessage(messages.retry) }}</Button>
				</Admonition>
			</template>
		</ContentDiffModal>
		<ConfirmModal ref="removeModal" :title="formatMessage(messages.removePlayer)" :description="formatMessage(messages.removeDescription, { username: playerToRemove?.username ?? '' })" :proceed-label="formatMessage(messages.removePlayer)" @proceed="removePlayer" />
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import ConfirmModal from '#ui/components/modal/ConfirmModal.vue'
import { type InviteLinkSettings, type InvitePlayersInvitePayload, InvitePlayersModal } from '#ui/components/sharing'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import ContentDiffModal from '#ui/layouts/shared/installation-settings/components/ContentDiffModal.vue'
import { getHostingServerAddress, injectAuth, injectModrinthClient, injectModrinthServerContext, injectNotificationManager, type ServerPlayTarget } from '#ui/providers'

import ServerPlayCard from './ServerPlayCard.vue'
import ServerPlayersTable from './ServerPlayersTable.vue'
import { resolveServerShareDiff } from './share-diff'
import type { ServerPlayerRow } from './types'
import { useServerPlayers } from './use-server-players'

type Action = 'play' | 'invite' | 'download' | 'push'
const props = defineProps<{
	onPlayServer: (target: ServerPlayTarget) => void | Promise<void>
	onDownloadMrpack: (blob: Blob, filename: string) => Promise<void>
	siteUrl: string
}>()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const auth = injectAuth()
const queryClient = useQueryClient()
const { serverId, worldId, server, serverFull, busyReasons } = injectModrinthServerContext()
const { canSetup } = useServerPermissions()
const world = computed(() => serverFull.value?.worlds.find((world) => world.id === worldId.value))
const sharedInstanceId = computed(() => world.value?.content?.shared_instance_id ?? null)
const needsUpdate = computed(() => world.value?.content?.shared_instance_needs_update ?? false)
const players = useServerPlayers(sharedInstanceId, canSetup)
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal>>()
const diffModal = ref<InstanceType<typeof ContentDiffModal>>()
const removeModal = ref<InstanceType<typeof ConfirmModal>>()
const playerToRemove = ref<ServerPlayerRow>()
const previewOpen = ref(false)
const previewAction = ref<'play' | 'push'>('push')
const preferences = useStorage(`pyro-server-${serverId}-preferences`, {
	reviewChangesBeforePlaying: false,
})
const serverAddress = computed(() => getHostingServerAddress(server.value.net, serverFull.value?.subdomain))
const previewQuery = useQuery({
	queryKey: computed(() => ['servers', 'share-diff', serverId, worldId.value, auth.user.value?.id]),
	enabled: computed(() => previewOpen.value && !!worldId.value && !!sharedInstanceId.value),
	queryFn: async () => {
		const diff = await client.archon.content_v1.getShareDiff(serverId, worldId.value!)
		return { diff, items: await resolveServerShareDiff(client, diff) }
	},
	retry: false,
})
const actionMutation = useMutation({
	mutationFn: async ({ action, targetWorldId, userId }: { action: Action; targetWorldId: string; userId: string | undefined }) => {
		if (busyReasons.value.length) throw new Error(formatMessage(messages.busy))
		if (auth.user.value?.id !== userId) return
		const sameContext = () => worldId.value === targetWorldId && auth.user.value?.id === userId
		let id = serverFull.value?.worlds.find((world) => world.id === targetWorldId)?.content?.shared_instance_id
		if (canSetup.value) {
			const shared = await client.archon.content_v1.share(serverId, targetWorldId)
			id = shared.shared_instance_id
			queryClient.setQueryData<Archon.Servers.v1.ServerFull>(['servers', 'v1', 'detail', serverId], (current) => current ? {
				...current,
				worlds: current.worlds.map((world) => world.id === targetWorldId && world.content ? {
					...world, content: { ...world.content, shared_instance_id: shared.shared_instance_id },
				} : world),
			} : current)
			await queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] })
		} else if (action === 'invite' || action === 'push') {
			throw new Error(formatMessage(messages.permission))
		}
		if (!id) throw new Error(formatMessage(messages.notShared))
		if (!sameContext()) return
		if (action === 'play') {
			await props.onPlayServer({ serverId, worldId: targetWorldId })
		} else if (action === 'invite') {
			await nextTick()
			await players.members.refetch({ throwOnError: true })
			if (!sameContext()) return
			await players.ensureLink(id)
			if (sameContext()) invitePlayersModal.value?.show()
		} else if (action === 'download') {
			const latest = await client.sharedinstances.instances_v1.getLatestVersion(id)
			if (!latest.ready) throw new Error(formatMessage(messages.notReady))
			const blob = await client.sharedinstances.instances_v1.downloadMrpack(id, latest.version)
			if (sameContext()) await props.onDownloadMrpack(blob, `${server.value.name.replace(/[\\/:*?"<>|]/g, '_')}.mrpack`)
		} else {
			previewOpen.value = false
			await queryClient.invalidateQueries({ queryKey: ['servers', 'share-diff', serverId, targetWorldId] })
		}
	},
	onError: (error) => handleError(error),
})
const pendingAction = computed(() => actionMutation.isPending.value ? actionMutation.variables.value?.action : undefined)
const actionsLocked = computed(() => actionMutation.isPending.value || previewQuery.isFetching.value || players.linkMutation.isPending.value || busyReasons.value.length > 0)
function perform(action: Action, reviewed = false) {
	if (!worldId.value || actionsLocked.value) return
	if (action === 'play' && !reviewed && canSetup.value && needsUpdate.value && preferences.value.reviewChangesBeforePlaying) {
		void showPreview(true)
		return
	}
	previewOpen.value = false
	actionMutation.mutate({ action, targetWorldId: worldId.value, userId: auth.user.value?.id })
}
async function showPreview(playAfter = false) {
	if (actionsLocked.value) return
	const target = worldId.value
	previewAction.value = playAfter ? 'play' : 'push'
	previewOpen.value = true
	const result = await previewQuery.refetch()
	if (target !== worldId.value) return
	if (!previewOpen.value) return
	if (result.error) {
		previewOpen.value = false
		handleError(result.error)
	} else diffModal.value?.show()
}
function changeMember(userId: string, remove: boolean) {
	if (!sharedInstanceId.value || players.membershipMutation.isPending.value || !canSetup.value) return
	players.membershipMutation.mutate({ id: sharedInstanceId.value, userId, remove }, { onError: (error) => handleError(error) })
}
function invitePlayer(payload: InvitePlayersInvitePayload) {
	changeMember(payload.user.id, false)
}
function confirmRemove(player: ServerPlayerRow) {
	playerToRemove.value = player
	removeModal.value?.show()
}
function removePlayer() {
	if (playerToRemove.value) changeMember(playerToRemove.value.id, true)
}
async function updateInviteLink(settings: InviteLinkSettings) {
	if (!sharedInstanceId.value || players.linkMutation.isPending.value) return
	await players.linkMutation.mutateAsync({ id: sharedInstanceId.value, settings, replaceId: players.link.value?.id })
}
watch([worldId, () => auth.user.value?.id], () => {
	invitePlayersModal.value?.hide()
	diffModal.value?.hide()
	removeModal.value?.hide()
	previewOpen.value = false
	playerToRemove.value = undefined
})
const messages = defineMessages({
	invitedPlayersTitle: { id: 'servers.play.players.title', defaultMessage: 'Invited players' },
	unpublished: { id: 'servers.play.unpublished', defaultMessage: 'Your changes haven’t been shared yet' },
	unpublishedBody: { id: 'servers.play.unpublished-body', defaultMessage: 'Push an update to share your content changes with players.' },
	pushUpdate: { id: 'servers.play.push-update', defaultMessage: 'Push update' },
	pushAndPlay: { id: 'servers.play.push-and-play', defaultMessage: 'Push update and play' },
	shareChanges: { id: 'servers.play.share-changes', defaultMessage: 'Share your changes' },
	shareChangesBody: { id: 'servers.play.share-changes-body', defaultMessage: 'These changes will be available to players when they update their instance.' },
	inviteHeader: { id: 'servers.play.invite-header', defaultMessage: 'Invite players to {name}' },
	refreshingPreview: { id: 'servers.play.refreshing-preview', defaultMessage: 'Refreshing changes…' },
	previewError: { id: 'servers.play.preview-error', defaultMessage: 'Could not refresh the changes. Retry before publishing.' },
	loading: { id: 'servers.play.loading', defaultMessage: 'Loading players' },
	playersError: { id: 'servers.play.players-error', defaultMessage: 'Could not load invited players' },
	retry: { id: 'servers.play.retry', defaultMessage: 'Retry' },
	invitesUnavailable: { id: 'servers.play.invites-unavailable', defaultMessage: 'Invitations are unavailable while an action is in progress or the player limit has been reached.' },
	removePlayer: { id: 'servers.play.remove-player', defaultMessage: 'Remove player' },
	removeDescription: { id: 'servers.play.remove-description', defaultMessage: 'Remove {username} from this shared instance? This does not ban them from the Minecraft server.' },
	added: { id: 'servers.play.diff-added', defaultMessage: 'Added' },
	removed: { id: 'servers.play.diff-removed', defaultMessage: 'Removed' },
	busy: { id: 'servers.play.busy', defaultMessage: 'Wait for the current server operation to finish.' },
	permission: { id: 'servers.play.permission', defaultMessage: 'You do not have permission to share this world.' },
	notShared: { id: 'servers.play.not-shared', defaultMessage: 'An owner or editor needs to share this world first.' },
	notReady: { id: 'servers.play.not-ready', defaultMessage: 'The shared content is not ready yet. Please try again shortly.' },
})
</script>
