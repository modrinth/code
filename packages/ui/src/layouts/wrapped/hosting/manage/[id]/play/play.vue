<template>
	<div class="relative flex flex-col gap-6">
		<ServerPlayCard
			:address="serverAddress"
			:disabled="actionsLocked || migrating || !worldId || (!canSetup && !sharedInstanceId)"
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
			<InvitedPlayersTable
				:rows="players.rows.value"
				:loading="players.members.isLoading.value"
				:can-manage="canSetup"
				:disabled="players.membershipMutation.isPending.value || actionsLocked || migrating"
				:show-push-update="!!sharedInstanceId"
				:push-update-disabled="actionsLocked || migrating || !worldId"
				:push-update-pending="previewQuery.isFetching.value || pendingAction === 'push'"
				:invite-disabled="actionsLocked || migrating || !worldId"
				:invite-pending="pendingAction === 'invite'"
				@push-update="showPreview()"
				@push-update-hover="shareReview.prefetchFiles()"
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
			v-bind="inviteProps"
			:header="inviteHeader"
			@invite="invitePlayer"
			@cancel="(user) => changeMember(user.id, true)"
		/>
		<ServerSharePushModal
			ref="diffModal"
			:server-id="serverId"
			:world-id="worldId ?? undefined"
			:diffs="previewQuery.data.value?.items ?? []"
			:changes-description="
				previewAction === 'play' ? formatMessage(messages.shareChangesPlayBody) : undefined
			"
			:confirm-label="previewAction === 'play' ? formatMessage(messages.pushAndPlay) : undefined"
			:restart-confirm-label="
				previewAction === 'play' ? formatMessage(messages.pushRestartAndPlay) : undefined
			"
			:requires-restart="previewQuery.data.value?.diff.requires_restart"
			:loading="previewQuery.isFetching.value"
			:error="previewQuery.isError.value"
			:disabled="
				!canSetup || actionsLocked || previewQuery.isError.value || !previewQuery.data.value
			"
			@confirm="(paths, restart) => perform(previewAction, true, paths, restart)"
			@cancel="previewOpen = false"
			@retry="previewQuery.refetch()"
		/>
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
import { useIntersectionObserver } from '@vueuse/core'
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import ConfirmModal from '#ui/components/modal/ConfirmModal.vue'
import ServerSharePushModal from '#ui/components/servers/sharing/ServerSharePushModal.vue'
import {
	type InvitedPlayerRow,
	InvitedPlayersTable,
	type InviteLinkSettings,
	type InvitePlayersContentProps,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersUser,
} from '#ui/components/sharing'
import {
	injectServerInviteHandoff,
	type ServerInviteBinding,
} from '#ui/components/sharing/server-sharing-update/handoff'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { sharedInstanceMrpackFilename } from '#ui/composables/server-sharing/shared-instance'
import {
	type ServerShareActionTarget,
	useServerShareReview,
} from '#ui/composables/server-sharing/use-server-share-review'
import { useEnsureServerRunning } from '#ui/composables/use-ensure-server-running'
import {
	injectAuth,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	type ServerPlayTarget,
} from '#ui/providers'
import { injectPageContext } from '#ui/providers/page-context'
import { getHostingServerAddress } from '#ui/utils/servers'

import {
	injectServerPlayMigration,
	type ServerPlayMigrationNext,
} from '../../providers/server-play-migration'
import ServerPlayCard from './ServerPlayCard.vue'
import { useServerPlayers } from './use-server-players'

type Action = 'play' | 'invite' | 'download' | 'push'
const props = defineProps<{
	onPlayServer: (target: ServerPlayTarget) => void | Promise<void>
	onDownloadMrpack?: (instanceId: string, version: number, filename: string) => Promise<void>
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
const { serverId, worldId, server, serverFull } = injectModrinthServerContext()
const { canSetup } = useServerPermissions()
const ensureServerRunning = useEnsureServerRunning()
const world = computed(() => serverFull.value?.worlds.find((world) => world.id === worldId.value))
const sharedInstanceId = computed(() => world.value?.content?.shared_instance_id ?? null)
const needsUpdate = computed(() => world.value?.content?.shared_instance_needs_update ?? false)
const players = useServerPlayers(sharedInstanceId, canSetup)
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal>>()
const inviteHandoff = injectServerInviteHandoff(null)
const migration = injectServerPlayMigration(null)
const migrating = computed(() => !!migration && (migration.active.value || !migration.ready.value))
let inviteHandoffStarted = false
let disposed = false
onScopeDispose(() => {
	disposed = true
})
const shareReview = useServerShareReview<Action>({
	execute: performAction,
	disabled: computed(() => players.linkMutation.isPending.value),
})
const {
	diffModal,
	previewOpen,
	previewQuery,
	actionMutation,
	pending: actionsLocked,
	runAction,
} = shareReview
const removeModal = ref<InstanceType<typeof ConfirmModal>>()
const playerToRemove = ref<InvitedPlayerRow>()
const previewAction = ref<'play' | 'push'>('push')
const inviteHeader = computed(() =>
	formatMessage(messages.inviteHeader, { name: server.value.name }),
)
const inviteProps = computed<InvitePlayersContentProps>(() => ({
	friends: players.candidates.value,
	searchUsers: players.search,
	link: players.link.value
		? `${props.siteUrl}/share/${encodeURIComponent(players.link.value.id)}`
		: undefined,
	linkExpiresAt: players.link.value?.expiration,
	linkMaxUses: players.link.value?.max_uses,
	linkMaxUsesLimit: players.remaining.value,
	updateInviteLink,
	canInvite:
		canSetup.value &&
		!actionsLocked.value &&
		!players.membershipMutation.isPending.value &&
		players.remaining.value > 0,
	inviteDisabledMessage: formatMessage(messages.invitesUnavailable),
}))
const inviteBinding = computed<ServerInviteBinding>(() => ({
	header: inviteHeader.value,
	props: inviteProps.value,
	onInvite: invitePlayer,
	onCancel: (user) => changeMember(user.id, true),
}))
const serverAddress = computed(() =>
	getHostingServerAddress(server.value?.net, serverFull.value?.subdomain),
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
		if (!sameContext()) return
		if (inviteHandoff?.requested.value) inviteHandoff.provide(inviteBinding)
		else invitePlayersModal.value?.show()
	} else if (action === 'download' && props.onDownloadMrpack) {
		const latest = await client.sharedinstances.instances_v1.getLatestVersion(id)
		if (!latest.ready) throw new Error(formatMessage(messages.notReady))
		if (!sameContext()) return
		await props.onDownloadMrpack(
			id,
			latest.version,
			sharedInstanceMrpackFilename(server.value.name),
		)
	}
}
const pendingAction = computed(() =>
	actionMutation.isPending.value ? actionMutation.variables.value?.action : undefined,
)
async function perform(
	action: Action,
	reviewed = false,
	configPaths: string[] = [],
	restart?: boolean,
) {
	if (!worldId.value || actionsLocked.value || migrating.value) return
	if (requireMigration(action, () => perform(action, reviewed, configPaths, restart))) return
	if (action === 'play' && !reviewed && canSetup.value && needsUpdate.value) {
		void showPreview(true)
		return
	}
	await runAction(action, reviewed, configPaths, restart)
}
function requireMigration(action: Action, resume: () => Promise<void>) {
	if (!migration?.required.value) return false
	const next: ServerPlayMigrationNext =
		action === 'invite'
			? 'invite'
			: () => {
					if (!disposed) void resume()
				}
	if (inviteHandoff?.requested.value) {
		inviteHandoff.provideMigration({
			modpackName: migration.modpackName.value,
			start: () => migration.start(next),
		})
	} else {
		migration.prompt(next)
	}
	return true
}
async function showPreview(playAfter = false) {
	if (actionsLocked.value || migrating.value) return
	if (requireMigration('push', () => showPreview(playAfter))) return
	previewAction.value = playAfter ? 'play' : 'push'
	await shareReview.showPreview(previewAction.value)
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
function confirmRemove(player: InvitedPlayerRow) {
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
watch(
	[worldId, canSetup, () => inviteHandoff?.requested.value, () => migration?.ready.value],
	async () => {
		if (!inviteHandoff?.requested.value) {
			inviteHandoffStarted = false
			return
		}
		if (!worldId.value || inviteHandoffStarted || (migration && !migration.ready.value)) return
		inviteHandoffStarted = true
		if (canSetup.value) await perform('invite')
		if (!inviteHandoff.binding.value && !inviteHandoff.migration.value) inviteHandoff.cancel()
	},
	{ immediate: true },
)
onScopeDispose(() => {
	if (inviteHandoff?.binding.value === inviteBinding.value) inviteHandoff.cancel()
})
watch([worldId, sharedInstanceId, () => auth.user.value?.id], () => {
	invitePlayersModal.value?.hide()
	removeModal.value?.hide()
	playerToRemove.value = undefined
})
const messages = defineMessages({
	invitedPlayersTitle: { id: 'servers.play.players.title', defaultMessage: 'Invited players' },
	pushAndPlay: { id: 'servers.play.push-and-play', defaultMessage: 'Push update and play' },
	pushRestartAndPlay: {
		id: 'servers.play.push-restart-and-play',
		defaultMessage: 'Push, restart and play',
	},
	shareChangesPlayBody: {
		id: 'servers.play.share-changes-play-body',
		defaultMessage:
			'Starting the server will push these changes to all invited players. They will be asked to update their instance the next time they play.',
	},
	inviteHeader: { id: 'servers.play.invite-header', defaultMessage: 'Invite players to {name}' },
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
