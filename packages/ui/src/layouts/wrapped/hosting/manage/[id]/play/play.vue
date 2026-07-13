<template>
	<div class="flex flex-col gap-6">
		<ServerPlayCard
			:address="serverAddress"
			:modpack-download-url="modpackDownload?.url"
			:modpack-filename="modpackDownload?.filename"
			@play="props.onPlayServer?.(serverAddress)"
			@invite="showInvitePlayers"
		/>

		<section class="flex flex-col gap-3">
			<h1 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.invitedPlayersTitle) }}
			</h1>
			<SharedPlayersTable
				:rows="playersToDisplay"
				variant="server"
				@remove="removePlayer"
				@open-actions="(player) => props.onOpenPlayerActions?.(player)"
			/>
		</section>

		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="invitePlayersHeader"
			:friends="dummyFriends"
			:search-users="searchInviteUsers"
			:link="dummyInviteLink"
			:link-expires-at="dummyInviteLinkExpiresAt"
			:link-max-uses="10"
			@invite="invitePlayer"
			@cancel="cancelPlayerInvite"
		/>
	</div>
</template>

<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersSearchUser,
	type InvitePlayersUser,
	SharedPlayersTable,
	type SharedPlayersTableRow,
} from '#ui/components/sharing'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

import ServerPlayCard from './ServerPlayCard.vue'

const props = defineProps<{
	players?: SharedPlayersTableRow[]
	onPlayServer?: (address: string) => void | Promise<void>
	onInvitePlayers?: () => void | Promise<void>
	onRemovePlayer?: (player: SharedPlayersTableRow) => void | Promise<void>
	onOpenPlayerActions?: (player: SharedPlayersTableRow) => void | Promise<void>
}>()

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { server, serverFull } = injectModrinthServerContext()
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal> | null>(null)
const dummyPlayers = ref<SharedPlayersTableRow[]>(createDummyPlayers())
const dummyFriends = ref<InvitePlayersUser[]>(createDummyFriends())
const dummySearchUsers = createDummySearchUsers()
const dummyInviteLinkExpiresAt = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
const dummyInviteLink = 'https://modrinth.com/servers/invite/demo'

const messages = defineMessages({
	invitedPlayersTitle: {
		id: 'servers.play.players.title',
		defaultMessage: 'Invited players',
	},
})

const playersToDisplay = computed(() => props.players ?? dummyPlayers.value)
const invitePlayersHeader = computed(() => `Invite players to ${server.value.name}`)

const serverAddress = computed(() => {
	const subdomain = server.value.net.domain || serverFull.value?.subdomain
	if (subdomain) {
		return subdomain.endsWith('.modrinth.gg') ? subdomain : `${subdomain}.modrinth.gg`
	}

	return server.value.net.ip || ''
})

const modpackVersionId = computed(() =>
	server.value.upstream?.kind === 'modpack' ? server.value.upstream.version_id : null,
)

const { data: modpackVersion } = useQuery({
	queryKey: computed(() => ['servers', 'play', 'modpack-version', modpackVersionId.value]),
	queryFn: () => client.labrinth.versions_v3.getVersion(modpackVersionId.value!),
	enabled: computed(() => Boolean(modpackVersionId.value)),
	staleTime: 5 * 60 * 1000,
})

const modpackDownload = computed(() => {
	const version = modpackVersion.value
	if (!version) return undefined
	return version.files.find((file) => file.primary) ?? version.files[0]
})

function showInvitePlayers() {
	invitePlayersModal.value?.show()
	void props.onInvitePlayers?.()
}

async function searchInviteUsers(query: string): Promise<InvitePlayersSearchUser[]> {
	const normalizedQuery = query.trim().toLowerCase()
	const friendKeys = new Set(
		dummyFriends.value.flatMap((friend) => [
			friend.id.toLowerCase(),
			friend.username.toLowerCase(),
		]),
	)

	return dummySearchUsers.filter(
		(user) =>
			user.username.toLowerCase().startsWith(normalizedQuery) &&
			!friendKeys.has(user.id.toLowerCase()) &&
			!friendKeys.has(user.username.toLowerCase()),
	)
}

function invitePlayer(payload: InvitePlayersInvitePayload) {
	const existingFriend = dummyFriends.value.find((friend) => friend.id === payload.user.id)

	if (existingFriend) {
		existingFriend.status = 'pending'
	} else {
		dummyFriends.value.push({
			...payload.user,
			status: 'pending',
		})
	}

	const existingPlayer = dummyPlayers.value.find((player) => player.id === payload.user.id)
	if (existingPlayer) {
		existingPlayer.pending = true
		existingPlayer.method = 'direct'
		return
	}

	dummyPlayers.value.push({
		id: payload.user.id,
		username: payload.user.username,
		avatarUrl: payload.user.avatarUrl ?? undefined,
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
		pending: true,
	})
}

function cancelPlayerInvite(user: InvitePlayersUser) {
	const friend = dummyFriends.value.find((candidate) => candidate.id === user.id)
	if (friend) friend.status = 'available'

	dummyPlayers.value = dummyPlayers.value.filter(
		(player) => player.id !== user.id || !player.pending,
	)
}

function removePlayer(player: SharedPlayersTableRow) {
	if (player.pending) {
		cancelPlayerInvite({ id: player.id, username: player.username })
	}

	void props.onRemovePlayer?.(player)
}

function createDummyFriends(): InvitePlayersUser[] {
	return [
		{ id: 'coolbot', username: 'Coolbot', status: 'pending', online: true },
		{ id: 'geometrically', username: 'Geometrically', status: 'added', online: true },
		{ id: 'josh', username: 'Josh', status: 'added' },
		{ id: 'prospector', username: 'Prospector', status: 'added' },
		{ id: 'fetch', username: 'Fetch', status: 'available', online: true },
		{ id: 'emma', username: 'Emma', status: 'available' },
	]
}

function createDummySearchUsers(): InvitePlayersSearchUser[] {
	return [
		{ id: 'paperclip', username: 'Paperclip' },
		{ id: 'steve', username: 'Steve' },
		{ id: 'alex', username: 'Alex' },
	]
}

function createDummyPlayers(): SharedPlayersTableRow[] {
	const now = Date.now()
	const daysAgo = (days: number) => new Date(now - days * 24 * 60 * 60 * 1000)

	return [
		{
			id: 'coolbot',
			username: 'Coolbot',
			lastPlayedAt: null,
			joinedAt: null,
			method: 'direct',
			pending: true,
		},
		{
			id: 'geometrically',
			username: 'Geometrically',
			lastPlayedAt: daysAgo(0),
			joinedAt: daysAgo(0),
			method: 'link',
		},
		{
			id: 'josh',
			username: 'Josh',
			lastPlayedAt: daysAgo(0),
			joinedAt: daysAgo(0),
			method: 'link',
		},
		{
			id: 'boris',
			username: 'Boris',
			lastPlayedAt: daysAgo(4),
			joinedAt: daysAgo(7),
			method: 'direct',
		},
		{
			id: 'prospector',
			username: 'Prospector',
			lastPlayedAt: daysAgo(4),
			joinedAt: daysAgo(30),
			method: 'direct',
		},
		{
			id: 'imb',
			username: 'IMB',
			lastPlayedAt: daysAgo(14),
			joinedAt: daysAgo(90),
			method: 'direct',
		},
	]
}
</script>
