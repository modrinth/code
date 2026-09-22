<template>
	<div class="relative flex flex-col gap-6">
		<ServerPlayCard
			:address="serverAddress"
			:disabled="actionsLocked || !worldId || (!canSetup && !sharedInstanceId)"
			:pending-action="pendingAction"
			@play="perform('play')"
			@download="perform('download')"
		/>
		<section class="flex flex-col gap-3">
			<h1 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.invitedPlayersTitle) }}
			</h1>
			<Admonition
				v-if="players.members.isError.value"
				type="critical"
				:header="formatMessage(messages.playersError)"
			>
				<Button @click="players.members.refetch()">{{ formatMessage(messages.retry) }}</Button>
			</Admonition>
			<InvitedPlayersTableLayout
				:rows="players.rows.value"
				:loading="players.members.isLoading.value"
				:can-manage="canSetup"
				:disabled="players.membershipMutation.isPending.value || actionsLocked"
				:show-push-update="!!sharedInstanceId"
				:push-update-disabled="actionsLocked || !worldId"
				:push-update-pending="previewQuery.isFetching.value || pendingAction === 'push'"
				:invite-disabled="actionsLocked || !worldId"
				:invite-pending="pendingAction === 'invite'"
				@push-update="showPreview()"
				@invite="perform('invite')"
				@remove="confirmRemove"
			/>
		</section>
		<span
			ref="pageBottom"
			class="pointer-events-none absolute bottom-0 h-px w-px"
			aria-hidden="true"
		/>
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="formatMessage(messages.inviteHeader, { name: server.name })"
			:friends="players.candidates.value"
			:search-users="players.search"
			:link="
				players.link.value
					? `${siteUrl}/share/${encodeURIComponent(players.link.value.id)}`
					: undefined
			"
			:link-expires-at="players.link.value?.expiration"
			:link-max-uses="players.link.value?.max_uses"
			:link-max-uses-limit="players.remaining.value"
			:update-invite-link="updateInviteLink"
			:can-invite="
				canSetup &&
				!actionsLocked &&
				!players.membershipMutation.isPending.value &&
				players.remaining.value > 0
			"
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
			:confirm-label="
				formatMessage(previewAction === 'play' ? messages.pushAndPlay : messages.pushUpdate)
			"
			:confirm-icon="UploadIcon"
			:confirm-disabled="!canSetup || actionsLocked || previewQuery.isError.value || !previewQuery.data.value"
			:added-label="formatMessage(messages.added)"
			:removed-label="formatMessage(messages.removed)"
			@confirm="perform(previewAction, true)"
			@cancel="previewOpen = false"
		>
			<template #additional-content>
				<ServerConfigFilePicker
					v-if="previewOpen && worldId"
					:key="worldId"
					ref="configPicker"
					:server-id="serverId"
					:world-id="worldId"
					:disabled="actionsLocked"
				/>
				<p v-if="previewQuery.isFetching.value" class="m-0 flex items-center gap-2">
					<SpinnerIcon class="animate-spin" />{{ formatMessage(messages.refreshingPreview) }}
				</p>
				<Admonition
					v-else-if="previewQuery.isError.value"
					type="critical"
					:header="formatMessage(messages.previewError)"
				>
					<Button @click="previewQuery.refetch()">{{ formatMessage(messages.retry) }}</Button>
				</Admonition>
			</template>
		</ContentDiffModal>
		<ConfirmModal
			ref="removeModal"
			:title="formatMessage(messages.removePlayer)"
			:description="
				formatMessage(messages.removeDescription, { username: playerToRemove?.username ?? '' })
			"
			:proceed-label="formatMessage(messages.removePlayer)"
			@proceed="removePlayer"
		/>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { useIntersectionObserver } from '@vueuse/core'
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import ConfirmModal from '#ui/components/modal/ConfirmModal.vue'
import ServerConfigFilePicker from '#ui/components/servers/ServerConfigFilePicker.vue'
import {
	type InviteLinkSettings,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersUser,
} from '#ui/components/sharing'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { useServerPreferences } from '#ui/composables/server-preferences'
import ContentDiffModal from '#ui/layouts/shared/installation-settings/components/ContentDiffModal.vue'
import InvitedPlayersTableLayout from '#ui/layouts/shared/invited-players/layout.vue'
import {
	type ServerShareActionTarget,
	useServerShareReview,
} from '#ui/layouts/shared/server-sharing/use-server-share-review'
import {
	getHostingServerAddress,
	injectAuth,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	type ServerPlayTarget,
} from '#ui/providers'
import { injectPageContext } from '#ui/providers/page-context'

import ServerPlayCard from './ServerPlayCard.vue'
import type { ServerPlayerRow } from './types'
import { useServerPlayers } from './use-server-players'

type Action = 'play' | 'invite' | 'download' | 'push'
const props = defineProps<{
	onPlayServer: (target: ServerPlayTarget) => void | Promise<void>
	onDownloadMrpack: (blob: Blob, filename: string) => Promise<void>
	siteUrl: string
}>()
const { formatMessage } = useVIntl()
const pageContext = injectPageContext(null)
const pageBottom = ref<HTMLElement | null>(null)
const intercomHiddenRequestId = Symbol('server-play-bottom')
useIntersectionObserver(pageBottom, ([entry]) => {
	pageContext?.intercomBubble?.requestHidden?.(
		intercomHiddenRequestId,
		entry?.isIntersecting ?? false,
	)
})
onScopeDispose(() => {
	pageContext?.intercomBubble?.requestHidden?.(intercomHiddenRequestId, false)
})
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const auth = injectAuth()
const { serverId, worldId, server, serverFull, busyReasons, powerState, isConnected } =
	injectModrinthServerContext()
const { canSetup, canUsePowerActions, permissionDeniedMessage } = useServerPermissions()
const world = computed(() => serverFull.value?.worlds.find((world) => world.id === worldId.value))
const sharedInstanceId = computed(() => world.value?.content?.shared_instance_id ?? null)
const needsUpdate = computed(() => world.value?.content?.shared_instance_needs_update ?? false)
const players = useServerPlayers(sharedInstanceId, canSetup)
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal>>()
const shareReview = useServerShareReview<Action>({
	execute: performAction,
	disabled: computed(() => players.linkMutation.isPending.value),
})
const {
	diffModal,
	configPicker,
	previewOpen,
	previewQuery,
	actionMutation,
	pending: actionsLocked,
	runAction,
} = shareReview
const removeModal = ref<InstanceType<typeof ConfirmModal>>()
const playerToRemove = ref<ServerPlayerRow>()
const previewAction = ref<'play' | 'push'>('push')
const preferences = useServerPreferences(serverId)
const serverAddress = computed(() =>
	getHostingServerAddress(server.value.net, serverFull.value?.subdomain),
)
async function performAction(action: Action, target: ServerShareActionTarget) {
	const { worldId: targetWorldId, configPaths, isCurrent: sameContext } = target
	let id = serverFull.value?.worlds.find((world) => world.id === targetWorldId)?.content
		?.shared_instance_id
	const shouldShare =
		canSetup.value &&
		(!id ||
			action === 'push' ||
			((action === 'play' || action === 'download') &&
				(needsUpdate.value || configPaths.length > 0)))
	if (shouldShare) {
		const shared = await shareReview.publish(target)
		id = shared.shared_instance_id
	} else if (!canSetup.value && (action === 'invite' || action === 'push')) {
		throw new Error(formatMessage(messages.permission))
	}
	if (!id) throw new Error(formatMessage(messages.notShared))
	if (!sameContext()) return
	if (action === 'play') {
		await ensureServerRunning(sameContext)
		if (!sameContext()) return
		await props.onPlayServer({ serverId, worldId: targetWorldId })
	} else if (action === 'invite') {
		await nextTick()
		const members = await players.members.refetch({ throwOnError: true })
		if (!sameContext()) return
		await players.ensureLink(id, members.data?.remaining)
		if (sameContext()) invitePlayersModal.value?.show()
	} else if (action === 'download') {
		const latest = await client.sharedinstances.instances_v1.getLatestVersion(id)
		if (!latest.ready) throw new Error(formatMessage(messages.notReady))
		const blob = await client.sharedinstances.instances_v1.downloadMrpack(id, latest.version)
		if (sameContext())
			await props.onDownloadMrpack(
				blob,
				`${server.value.name.replace(/[\\/:*?"<>|]/g, '_')}.mrpack`,
			)
	}
}
const pendingAction = computed(() =>
	actionMutation.isPending.value ? actionMutation.variables.value?.action : undefined,
)
async function ensureServerRunning(sameContext: () => boolean) {
	const deadline = Date.now() + 180_000
	let startRequested = false
	let sawStarting = false
	while (sameContext()) {
		if (busyReasons.value.length) throw new Error(formatMessage(messages.busy))
		if (isConnected.value) {
			const state = powerState.value
			if (state === 'running') return
			if (state === 'starting') sawStarting = true
			if (sawStarting && (state === 'stopped' || state === 'crashed' || state === 'stopping')) {
				throw new Error(formatMessage(messages.startFailed))
			}
			if (!startRequested && (state === 'stopped' || state === 'crashed')) {
				if (!canUsePowerActions.value) throw new Error(permissionDeniedMessage.value)
				startRequested = true
				await client.archon.servers_v0.power(serverId, 'Start')
			}
		}
		if (Date.now() >= deadline) throw new Error(formatMessage(messages.startTimeout))
		await new Promise((resolve) => setTimeout(resolve, 500))
	}
}
async function perform(action: Action, reviewed = false) {
	if (!worldId.value || actionsLocked.value) return
	if (
		action === 'play' &&
		!reviewed &&
		canSetup.value &&
		needsUpdate.value &&
		preferences.value.reviewChangesBeforePlaying
	) {
		void showPreview(true)
		return
	}
	await runAction(action, reviewed)
}
async function showPreview(playAfter = false) {
	if (actionsLocked.value) return
	previewAction.value = playAfter ? 'play' : 'push'
	await shareReview.showPreview()
}
function changeMember(userId: string, remove: boolean, user?: InvitePlayersUser) {
	if (!sharedInstanceId.value || players.membershipMutation.isPending.value || !canSetup.value)
		return
	players.membershipMutation.mutate(
		{ id: sharedInstanceId.value, userId, remove, user },
		{ onError: (error) => handleError(error) },
	)
}
function invitePlayer(payload: InvitePlayersInvitePayload) {
	changeMember(payload.user.id, false, payload.user)
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
	await players.linkMutation.mutateAsync({
		id: sharedInstanceId.value,
		settings,
		replaceId: players.link.value?.id,
	})
}
watch([worldId, sharedInstanceId, () => auth.user.value?.id], () => {
	invitePlayersModal.value?.hide()
	removeModal.value?.hide()
	playerToRemove.value = undefined
})
const messages = defineMessages({
	startFailed: {
		id: 'servers.play.start-failed',
		defaultMessage:
			'The server stopped before it was ready. Check the console before trying again.',
	},
	startTimeout: {
		id: 'servers.play.start-timeout',
		defaultMessage:
			'The server is taking too long to start. Check the console before trying again.',
	},
	invitedPlayersTitle: { id: 'servers.play.players.title', defaultMessage: 'Invited players' },
	pushUpdate: { id: 'servers.play.push-update', defaultMessage: 'Push update' },
	pushAndPlay: { id: 'servers.play.push-and-play', defaultMessage: 'Push update and play' },
	shareChanges: { id: 'servers.play.share-changes', defaultMessage: 'Share your changes' },
	shareChangesBody: {
		id: 'servers.play.share-changes-body',
		defaultMessage: 'These changes will be available to players when they update their instance.',
	},
	inviteHeader: { id: 'servers.play.invite-header', defaultMessage: 'Invite players to {name}' },
	refreshingPreview: {
		id: 'servers.play.refreshing-preview',
		defaultMessage: 'Refreshing changes…',
	},
	previewError: {
		id: 'servers.play.preview-error',
		defaultMessage: 'Could not refresh the changes. Retry before publishing.',
	},
	playersError: {
		id: 'servers.play.players-error',
		defaultMessage: 'Could not load invited players',
	},
	retry: { id: 'servers.play.retry', defaultMessage: 'Retry' },
	invitesUnavailable: {
		id: 'servers.play.invites-unavailable',
		defaultMessage:
			'Invitations are unavailable while an action is in progress or the player limit has been reached.',
	},
	removePlayer: { id: 'servers.play.remove-player', defaultMessage: 'Remove player' },
	removeDescription: {
		id: 'servers.play.remove-description',
		defaultMessage:
			'Remove {username} from this shared instance? This does not ban them from the Minecraft server.',
	},
	added: { id: 'servers.play.diff-added', defaultMessage: 'Added' },
	removed: { id: 'servers.play.diff-removed', defaultMessage: 'Removed' },
	busy: {
		id: 'servers.play.busy',
		defaultMessage: 'Wait for the current server operation to finish.',
	},
	permission: {
		id: 'servers.play.permission',
		defaultMessage: 'You do not have permission to share this world.',
	},
	notShared: {
		id: 'servers.play.not-shared',
		defaultMessage: 'An owner or editor needs to share this world first.',
	},
	notReady: {
		id: 'servers.play.not-ready',
		defaultMessage: 'The shared content is not ready yet. Please try again shortly.',
	},
})
</script>
