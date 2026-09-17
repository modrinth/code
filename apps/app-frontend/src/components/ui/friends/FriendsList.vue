<script setup lang="ts">
import {
	MailIcon,
	MoreVerticalIcon,
	SearchIcon,
	SendIcon,
	TrashIcon,
	UserIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	IntlFormatted,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { ModrinthCredentials } from '@/helpers/mr_auth'
import {
	fetchOwyxCatalog,
	getOwyxClientKey,
	getOwyxDemoFlag,
	getOwyxLocalApiFallback,
	getStoredOwyxApiBase,
	type OwyxServerEntry,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { resolveOwyxAvatarUrl } from '@/helpers/owyx-avatar'
import {
	acceptOwyxFriend,
	declineOwyxFriend,
	listOwyxFriends,
	type OwyxFriend,
	owyxFriendLabel,
	removeOwyxFriend,
	requestOwyxFriend,
	searchOwyxUsers,
} from '@/helpers/owyx-friends'
import { playOwyxUiSound } from '@/helpers/owyx-ui-sound'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const owyx = injectOwyxSiteSession()

const props = defineProps<{
	credentials: ModrinthCredentials | null
	owyxSignedIn?: boolean
	signIn: () => void
}>()

const friends = ref<OwyxFriend[]>([])
const catalogServers = ref<OwyxServerEntry[]>([])
const loading = ref(false)
const listError = ref('')
const offline = ref(typeof navigator !== 'undefined' ? !navigator.onLine : false)
const search = ref('')
const username = ref('')
const searchHits = ref<
	{ id: string; nickname: string; displayNickname?: string; avatarUrl?: string | null }[]
>([])
const searchBusy = ref(false)
const friendInvitesModal = ref<{ show: () => void; hide: () => void } | null>(null)
const addFriendModal = ref<{ show: () => void; hide: () => void } | null>(null)

let pollTimer: ReturnType<typeof setInterval> | null = null
let searchDebounce: ReturnType<typeof setTimeout> | null = null

function onOffline() {
	offline.value = true
}
function onOnline() {
	offline.value = false
	void quietRefresh()
}

function onVisibility() {
	if (document.visibilityState === 'visible') void quietRefresh()
}

function onOpenFriendsEvent(ev: Event) {
	const detail = (ev as CustomEvent<{ focus?: string }>).detail
	void quietRefresh()
	if (detail?.focus === 'incoming') {
		friendInvitesModal.value?.show()
	}
}

async function refresh() {
	if (!props.owyxSignedIn) {
		friends.value = []
		listError.value = ''
		return
	}
	loading.value = true
	listError.value = ''
	try {
		friends.value = await listOwyxFriends()
	} catch (e) {
		listError.value = e instanceof Error ? e.message : String(e)
		handleError(e)
		friends.value = []
	} finally {
		loading.value = false
	}
}

async function quietRefresh() {
	if (!props.owyxSignedIn) return
	try {
		friends.value = await listOwyxFriends()
		listError.value = ''
	} catch {
		/* keep previous list */
	}
}

async function loadCatalogQuiet() {
	try {
		const result = await fetchOwyxCatalog({
			baseUrl: sanitizeOwyxApiBase(getStoredOwyxApiBase()),
			clientKey: getOwyxClientKey(),
			authToken: owyx.session.value?.token,
			demoFallback: getOwyxDemoFlag(),
			allowLocalFallback: getOwyxLocalApiFallback(),
		})
		catalogServers.value = result.servers
	} catch {
		/* optional */
	}
}

function matchCatalogServer(friend: OwyxFriend): OwyxServerEntry | null {
	const name = friend.instanceName?.trim().toLowerCase()
	if (!name) return null
	return (
		catalogServers.value.find(
			(s) =>
				s.name.toLowerCase() === name ||
				s.address.toLowerCase() === name ||
				s.id.toLowerCase() === name,
		) || null
	)
}

onMounted(() => {
	window.addEventListener('offline', onOffline)
	window.addEventListener('online', onOnline)
	document.addEventListener('visibilitychange', onVisibility)
	window.addEventListener('owyx:open-friends', onOpenFriendsEvent as EventListener)
	void refresh()
	void loadCatalogQuiet()
	pollTimer = setInterval(() => {
		void quietRefresh()
	}, 20_000)
})
onUnmounted(() => {
	window.removeEventListener('offline', onOffline)
	window.removeEventListener('online', onOnline)
	document.removeEventListener('visibilitychange', onVisibility)
	window.removeEventListener('owyx:open-friends', onOpenFriendsEvent as EventListener)
	if (pollTimer) clearInterval(pollTimer)
	if (searchDebounce) clearTimeout(searchDebounce)
})
watch(
	() => props.owyxSignedIn,
	() => {
		void refresh()
	},
)

watch(username, (q) => {
	if (searchDebounce) clearTimeout(searchDebounce)
	const trimmed = q.trim()
	if (trimmed.length < 2) {
		searchHits.value = []
		return
	}
	searchDebounce = setTimeout(async () => {
		searchBusy.value = true
		try {
			searchHits.value = await searchOwyxUsers(trimmed)
		} catch {
			searchHits.value = []
		} finally {
			searchBusy.value = false
		}
	}, 280)
})

const isSearching = computed(() => search.value.trim().length > 0)

const filtered = computed(() =>
	friends.value.filter((f) => {
		const q = search.value.trim().toLowerCase()
		if (!q) return true
		return (
			f.nickname.toLowerCase().includes(q) ||
			(f.displayNickname || '').toLowerCase().includes(q)
		)
	}),
)
const accepted = computed(() => filtered.value.filter((f) => f.status === 'accepted'))
const onlineFriends = computed(() =>
	accepted.value.filter((f) => f.presence === 'online' || f.presence === 'playing'),
)
const offlineFriends = computed(() =>
	accepted.value.filter((f) => f.presence !== 'online' && f.presence !== 'playing'),
)
const pendingOutgoing = computed(() =>
	filtered.value.filter((f) => f.status === 'pending' && !f.incoming),
)
const incomingRequests = computed(() =>
	friends.value.filter((f) => f.status === 'pending' && f.incoming),
)

function friendStatusLabel(friend: OwyxFriend) {
	if (friend.presence === 'playing') {
		return formatMessage(messages.playingStatus, {
			name: friend.instanceName || formatMessage(messages.unknownInstance),
		})
	}
	if (friend.presence === 'online') return formatMessage(messages.onlineStatus)
	return formatMessage(messages.offlineStatus)
}

const friendNickSet = computed(
	() => new Set(friends.value.map((f) => f.nickname.toLowerCase())),
)

function friendDisplay(friend: OwyxFriend | { nickname: string; displayNickname?: string }) {
	return owyxFriendLabel(friend)
}

defineExpose({
	refresh,
	quietRefresh,
	showIncoming: () => friendInvitesModal.value?.show(),
})

function showAddFriendModal() {
	username.value = ''
	searchHits.value = []
	addFriendModal.value?.show()
}

async function addFriendFromModal(nickOverride?: string) {
	const nick = (nickOverride ?? username.value).trim()
	if (!nick) return
	addFriendModal.value?.hide()
	try {
		await requestOwyxFriend(nick)
		username.value = ''
		searchHits.value = []
		playOwyxUiSound('success')
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function acceptIncoming(friend: OwyxFriend) {
	try {
		await acceptOwyxFriend(friend.id)
		playOwyxUiSound('success')
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function declineIncoming(friend: OwyxFriend) {
	try {
		await declineOwyxFriend(friend.id)
		playOwyxUiSound('soft')
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function removeFriend(friend: OwyxFriend) {
	try {
		await removeOwyxFriend(friend.id)
		playOwyxUiSound('soft')
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function copyPlayingInstance(friend: OwyxFriend) {
	const name = friend.instanceName?.trim()
	if (!name) return
	try {
		await navigator.clipboard.writeText(name)
		playOwyxUiSound('click')
		addNotification({
			type: 'success',
			title: formatMessage(messages.copiedInstanceName),
		})
	} catch (e) {
		handleError(e)
	}
}

async function copyFriendServerAddress(friend: OwyxFriend) {
	const server = matchCatalogServer(friend)
	if (!server?.address) return
	try {
		await navigator.clipboard.writeText(server.address)
		playOwyxUiSound('success')
		addNotification({
			type: 'success',
			title: formatMessage(messages.copiedServerAddress),
		})
	} catch (e) {
		handleError(e)
	}
}

defineExpose({ showAddFriendModal })

const messages = defineMessages({
	addFriend: { id: 'friends.action.add-friend', defaultMessage: 'Add a friend' },
	addingAFriend: { id: 'friends.add-friend.title', defaultMessage: 'Adding a friend' },
	usernameTitle: {
		id: 'friends.add-friend.username.title',
		defaultMessage: "What's your friend's Owyx nickname?",
	},
	usernameDescription: {
		id: 'friends.add-friend.username.description',
		defaultMessage: 'Search by nickname or type it exactly (3–16 characters).',
	},
	usernamePlaceholder: {
		id: 'friends.add-friend.username.placeholder',
		defaultMessage: 'Search Owyx nickname…',
	},
	sendFriendRequest: {
		id: 'friends.add-friend.submit',
		defaultMessage: 'Send friend request',
	},
	viewFriendRequests: {
		id: 'friends.action.view-friend-requests',
		defaultMessage: '{count} friend {count, plural, one {request} other {requests}}',
	},
	searchFriends: {
		id: 'friends.search-friends-placeholder',
		defaultMessage: 'Search friends...',
	},
	friends: { id: 'friends.heading', defaultMessage: 'Friends' },
	online: { id: 'friends.heading.online', defaultMessage: 'Online' },
	pending: { id: 'friends.heading.pending', defaultMessage: 'Pending' },
	sectionHeading: {
		id: 'friends.section.heading',
		defaultMessage: '{title} - {count}',
	},
	offlineStatus: {
		id: 'friends.status.offline',
		defaultMessage: 'Offline',
	},
	onlineStatus: {
		id: 'friends.status.online',
		defaultMessage: 'Online',
	},
	playingStatus: {
		id: 'friends.status.playing',
		defaultMessage: 'Playing {name}',
	},
	unknownInstance: {
		id: 'friends.status.unknown-instance',
		defaultMessage: 'Minecraft',
	},
	alreadyFriends: {
		id: 'friends.search.already',
		defaultMessage: 'Already friends',
	},
	friendRequestSent: {
		id: 'friends.friend.request-sent',
		defaultMessage: 'Friend request sent',
	},
	removeFriend: {
		id: 'friends.friend.remove-friend',
		defaultMessage: 'Remove friend',
	},
	cancelRequest: {
		id: 'friends.friend.cancel-request',
		defaultMessage: 'Cancel request',
	},
	noFriendsMatch: {
		id: 'friends.no-friends-match',
		defaultMessage: `No friends matching ''{query}''`,
	},
	signInToAddFriends: {
		id: 'friends.sign-in-to-add-friends',
		defaultMessage: '<link>Sign in to an Owyx account</link> to add friends.',
	},
	addFriendsToShare: {
		id: 'friends.add-friends-to-share',
		defaultMessage: '<link>Add friends</link> to see what they’re playing!',
	},
	friendRequestsHeader: {
		id: 'friends.requests.header',
		defaultMessage: 'Friend requests',
	},
	noPendingRequests: {
		id: 'friends.requests.empty',
		defaultMessage: 'No pending requests.',
	},
	sentARequest: {
		id: 'friends.requests.sent-a-request',
		defaultMessage: 'sent a request',
	},
	accept: { id: 'friends.requests.accept', defaultMessage: 'Accept' },
	decline: { id: 'friends.requests.decline', defaultMessage: 'Decline' },
	loading: { id: 'friends.loading', defaultMessage: 'Loading friends…' },
	offlineBanner: {
		id: 'friends.offline-banner',
		defaultMessage: 'You are offline. Friend list may be out of date.',
	},
	listError: {
		id: 'friends.list-error',
		defaultMessage:
			'Could not reach Owyx friends API. Check your connection and client key, then retry.',
	},
	retry: { id: 'friends.retry', defaultMessage: 'Retry' },
	copyInstance: {
		id: 'friends.copy-instance',
		defaultMessage: 'Copy what they’re playing',
	},
	copyServerAddress: {
		id: 'friends.copy-server-address',
		defaultMessage: 'Copy matching server address',
	},
	copiedInstanceName: {
		id: 'friends.copied-instance-name',
		defaultMessage: 'Instance name copied',
	},
	copiedServerAddress: {
		id: 'friends.copied-server-address',
		defaultMessage: 'Server address copied',
	},
})
</script>

<template>
	<ModalWrapper ref="friendInvitesModal" :header="formatMessage(messages.friendRequestsHeader)">
		<p v-if="incomingRequests.length === 0">{{ formatMessage(messages.noPendingRequests) }}</p>
		<div v-else class="flex flex-col gap-4 min-w-[28rem]">
			<div v-for="friend in incomingRequests" :key="friend.id" class="flex gap-2 items-center">
				<Avatar
					:src="resolveOwyxAvatarUrl(friend.avatarUrl)"
					class="w-12 h-12 rounded-full"
					size="2.25rem"
					circle
				/>
				<div class="flex-1 min-w-0">
					<p class="m-0">
						<span class="text-contrast font-medium">{{ friendDisplay(friend) }}</span>
						{{ ' ' }}{{ formatMessage(messages.sentARequest) }}
					</p>
				</div>
				<div class="flex gap-2">
					<Button type="colored" color="brand" @click="acceptIncoming(friend)">
						<UserPlusIcon />
						{{ formatMessage(messages.accept) }}
					</Button>
					<Button @click="declineIncoming(friend)">
						<XIcon />
						{{ formatMessage(messages.decline) }}
					</Button>
				</div>
			</div>
		</div>
	</ModalWrapper>

	<ModalWrapper ref="addFriendModal" :header="formatMessage(messages.addingAFriend)">
		<div class="min-w-[28rem]">
			<h2 class="m-0 text-base font-medium text-primary">
				{{ formatMessage(messages.usernameTitle) }}
			</h2>
			<p class="m-0 mt-1 text-sm text-secondary leading-tight">
				{{ formatMessage(messages.usernameDescription) }}
			</p>
			<div class="flex items-center gap-2 mt-4">
				<Input
					v-model="username"
					:icon="UserIcon"
					type="text"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					wrapper-class="flex-1"
					@keyup.enter="addFriendFromModal()"
				/>
				<Button
					type="colored"
					color="brand"
					:disabled="username.trim().length < 3"
					@click="addFriendFromModal()"
				>
					<SendIcon />
					{{ formatMessage(messages.sendFriendRequest) }}
				</Button>
			</div>
			<div v-if="searchHits.length > 0 || searchBusy" class="mt-3 flex flex-col gap-1">
				<p v-if="searchBusy" class="m-0 text-xs text-secondary">…</p>
				<button
					v-for="hit in searchHits"
					:key="hit.id"
					type="button"
					class="flex w-full cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent px-2 py-1.5 text-left hover:bg-button-bg"
					:disabled="friendNickSet.has(hit.nickname.toLowerCase())"
					@click="addFriendFromModal(hit.nickname)"
				>
					<Avatar :src="resolveOwyxAvatarUrl(hit.avatarUrl)" size="1.75rem" circle />
					<span class="flex-1 truncate text-sm text-contrast">{{
						friendDisplay(hit)
					}}</span>
					<span
						v-if="friendNickSet.has(hit.nickname.toLowerCase())"
						class="text-xs text-secondary"
						>{{ formatMessage(messages.alreadyFriends) }}</span
					>
					<UserPlusIcon v-else class="h-4 w-4 text-brand" />
				</button>
			</div>
		</div>
	</ModalWrapper>

	<p
		v-if="owyxSignedIn && offline"
		class="m-0 mb-2 rounded-lg bg-button-bg px-2 py-1.5 text-xs text-secondary"
		role="status"
	>
		{{ formatMessage(messages.offlineBanner) }}
	</p>

	<div
		v-if="owyxSignedIn && loading"
		class="friends-skeleton flex flex-col gap-2 mb-3"
		aria-busy="true"
	>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.loading) }}</p>
		<div class="h-8 rounded-full bg-button-bg animate-pulse" />
		<div class="h-8 rounded-full bg-button-bg animate-pulse opacity-80" />
		<div class="h-8 rounded-full bg-button-bg animate-pulse opacity-60" />
	</div>

	<div
		v-else-if="owyxSignedIn && listError && friends.length === 0"
		class="mb-3 flex flex-col gap-2 text-sm text-secondary"
	>
		<p class="m-0">{{ formatMessage(messages.listError) }}</p>
		<p v-if="listError" class="m-0 text-xs opacity-80">{{ listError }}</p>
		<Button size="sm" class="self-start" @click="refresh">{{
			formatMessage(messages.retry)
		}}</Button>
	</div>

	<div v-if="owyxSignedIn && !loading" class="flex gap-1 items-center mb-3 -ml-1">
		<template v-if="friends.length > 0">
			<IconButton
				v-tooltip="formatMessage(messages.addFriend)"
				type="quiet"
				:label="formatMessage(messages.addFriend)"
				@click="showAddFriendModal"
			>
				<UserPlusIcon />
			</IconButton>
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="text"
				appearance="transparent"
				:placeholder="formatMessage(messages.searchFriends)"
				clearable
				input-class="!text-primary !placeholder:text-primary"
				wrapper-class="flex-1 !border-button-bg [&>span:first-child]:!text-primary [&>span:first-child]:!opacity-100"
				@keyup.esc="search = ''"
			/>
		</template>
		<h3 v-else class="w-full text-base text-primary font-medium m-0">
			{{ formatMessage(messages.friends) }}
		</h3>
		<IconButton
			v-if="incomingRequests.length > 0 || friends.length > 0"
			v-tooltip="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
			type="quiet"
			:label="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
			class="relative"
			@click="friendInvitesModal?.show()"
		>
			<span
				v-if="incomingRequests.length > 0"
				class="absolute -top-0.5 -right-0.5 bg-brand text-inverted rounded-full size-4 text-[10px] flex items-center justify-center pointer-events-none"
			>
				{{ incomingRequests.length }}
			</span>
			<MailIcon />
		</IconButton>
		<IconButton
			v-if="friends.length === 0"
			v-tooltip="formatMessage(messages.addFriend)"
			type="quiet"
			:label="formatMessage(messages.addFriend)"
			@click="showAddFriendModal"
		>
			<UserPlusIcon />
		</IconButton>
	</div>

	<div v-if="owyxSignedIn && !loading" class="friends-list flex flex-col gap-2">
		<template v-if="friends.length > 0">
			<p v-if="filtered.length === 0" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.noFriendsMatch, { query: search }) }}
			</p>
			<template v-else>
				<Accordion
					v-if="onlineFriends.length > 0"
					:open-by-default="true"
					:force-open="isSearching"
					:button-class="
						'flex w-full items-center bg-transparent border-0 p-0' +
						(isSearching
							? ''
							: ' cursor-pointer hover:brightness-[--hover-brightness] active:scale-[0.98] transition-all')
					"
				>
					<template #title>
						<h3 class="text-base text-primary font-medium m-0">
							{{
								formatMessage(messages.sectionHeading, {
									title: formatMessage(messages.online),
									count: onlineFriends.length,
								})
							}}
						</h3>
					</template>
					<template #default>
						<div class="pt-3 flex flex-col gap-1">
							<div
								v-for="friend in onlineFriends"
								:key="friend.id"
								class="group grid items-center grid-cols-[1fr_auto] gap-2 hover:bg-button-bg transition-colors rounded-full mr-1 select-none"
							>
								<div class="grid min-w-0 grid-cols-[auto_1fr] items-center gap-2">
									<div class="relative shrink-0">
										<Avatar :src="resolveOwyxAvatarUrl(friend.avatarUrl)" size="2rem" circle />
										<span
											class="presence-dot"
											:class="
												friend.presence === 'playing'
													? 'presence-dot--playing'
													: 'presence-dot--online'
											"
											aria-hidden="true"
										/>
									</div>
									<div class="flex min-w-0 flex-col">
										<span class="truncate text-sm text-contrast m-0">{{
											friendDisplay(friend)
										}}</span>
										<span class="m-0 text-xs text-secondary">{{ friendStatusLabel(friend) }}</span>
									</div>
								</div>
								<TeleportOverflowMenu
									type="quiet"
									label="More options"
									class="opacity-0 group-hover:opacity-100 transition-opacity"
									:options="[
										...(friend.presence === 'playing' && friend.instanceName
											? [
													{
														id: 'copy-instance',
														label: formatMessage(messages.copyInstance),
														action: () => copyPlayingInstance(friend),
													},
												]
											: []),
										...(friend.presence === 'playing' && matchCatalogServer(friend)
											? [
													{
														id: 'copy-server-address',
														label: formatMessage(messages.copyServerAddress),
														action: () => copyFriendServerAddress(friend),
													},
												]
											: []),
										{
											id: 'remove-friend',
											label: formatMessage(messages.removeFriend),
											action: () => removeFriend(friend),
											tone: 'red',
										},
									]"
								>
									<MoreVerticalIcon />
									<template
										v-if="friend.presence === 'playing' && friend.instanceName"
										#copy-instance
									>
										{{ formatMessage(messages.copyInstance) }}
									</template>
									<template
										v-if="friend.presence === 'playing' && matchCatalogServer(friend)"
										#copy-server-address
									>
										{{ formatMessage(messages.copyServerAddress) }}
									</template>
									<template #remove-friend>
										<TrashIcon />
										{{ formatMessage(messages.removeFriend) }}
									</template>
								</TeleportOverflowMenu>
							</div>
						</div>
					</template>
				</Accordion>

				<Accordion
					v-if="offlineFriends.length > 0"
					:open-by-default="onlineFriends.length === 0"
					:force-open="isSearching"
					:button-class="
						'flex w-full items-center bg-transparent border-0 p-0' +
						(isSearching
							? ''
							: ' cursor-pointer hover:brightness-[--hover-brightness] active:scale-[0.98] transition-all')
					"
				>
					<template #title>
						<h3 class="text-base text-primary font-medium m-0">
							{{
								formatMessage(messages.sectionHeading, {
									title: formatMessage(messages.friends),
									count: offlineFriends.length,
								})
							}}
						</h3>
					</template>
					<template #default>
						<div class="pt-3 flex flex-col gap-1">
							<div
								v-for="friend in offlineFriends"
								:key="friend.id"
								class="group grid items-center grid-cols-[1fr_auto] gap-2 hover:bg-button-bg transition-colors rounded-full mr-1 select-none"
							>
								<div class="grid min-w-0 grid-cols-[auto_1fr] items-center gap-2">
									<Avatar
										:src="resolveOwyxAvatarUrl(friend.avatarUrl)"
										size="2rem"
										circle
										class="grayscale opacity-80"
									/>
									<div class="flex min-w-0 flex-col">
										<span class="truncate text-sm text-primary m-0">{{
											friendDisplay(friend)
										}}</span>
										<span class="m-0 text-xs text-secondary">{{ friendStatusLabel(friend) }}</span>
									</div>
								</div>
								<TeleportOverflowMenu
									type="quiet"
									label="More options"
									class="opacity-0 group-hover:opacity-100 transition-opacity"
									:options="[
										{
											id: 'remove-friend',
											label: formatMessage(messages.removeFriend),
											action: () => removeFriend(friend),
											tone: 'red',
										},
									]"
								>
									<MoreVerticalIcon />
									<template #remove-friend>
										<TrashIcon />
										{{ formatMessage(messages.removeFriend) }}
									</template>
								</TeleportOverflowMenu>
							</div>
						</div>
					</template>
				</Accordion>

				<Accordion
					v-if="pendingOutgoing.length > 0"
					:open-by-default="true"
					:force-open="isSearching"
					:button-class="
						'flex w-full items-center bg-transparent border-0 p-0' +
						(isSearching
							? ''
							: ' cursor-pointer hover:brightness-[--hover-brightness] active:scale-[0.98] transition-all')
					"
				>
					<template #title>
						<h3 class="text-base text-primary font-medium m-0">
							{{
								formatMessage(messages.sectionHeading, {
									title: formatMessage(messages.pending),
									count: pendingOutgoing.length,
								})
							}}
						</h3>
					</template>
					<template #default>
						<div class="pt-3 flex flex-col gap-1">
							<div
								v-for="friend in pendingOutgoing"
								:key="friend.id"
								class="group grid items-center grid-cols-[1fr_auto] gap-2 hover:bg-button-bg transition-colors rounded-full mr-1 select-none"
							>
								<div class="grid min-w-0 grid-cols-[auto_1fr] items-center gap-2">
									<Avatar :src="resolveOwyxAvatarUrl(friend.avatarUrl)" size="2rem" circle />
									<div class="flex min-w-0 flex-col">
										<span class="truncate text-sm text-contrast m-0">{{
											friendDisplay(friend)
										}}</span>
										<span class="m-0 text-xs text-secondary">{{
											formatMessage(messages.friendRequestSent)
										}}</span>
									</div>
								</div>
								<IconButton
									v-tooltip="formatMessage(messages.cancelRequest)"
									type="quiet"
									:label="formatMessage(messages.cancelRequest)"
									@click="removeFriend(friend)"
								>
									<XIcon />
								</IconButton>
							</div>
						</div>
					</template>
				</Accordion>
			</template>
		</template>
		<div v-else class="text-secondary text-sm">
			<IntlFormatted :message-id="messages.addFriendsToShare">
				<template #link="{ children }">
					<button
						class="text-link cursor-pointer bg-transparent border-none p-0"
						@click="showAddFriendModal"
					>
						<component :is="() => children" />
					</button>
				</template>
			</IntlFormatted>
		</div>
	</div>

	<div v-else-if="!owyxSignedIn" class="text-secondary text-sm">
		<IntlFormatted :message-id="messages.signInToAddFriends">
			<template #link="{ children }">
				<button class="text-link cursor-pointer bg-transparent border-none p-0" @click="signIn()">
					<component :is="() => children" />
				</button>
			</template>
		</IntlFormatted>
	</div>
</template>

<style scoped>
.presence-dot {
	position: absolute;
	right: -1px;
	bottom: -1px;
	width: 0.55rem;
	height: 0.55rem;
	border-radius: 9999px;
	border: 2px solid var(--color-bg, #050508);
	background: #22c55e;
}
.presence-dot--playing {
	background: #00e5ff;
	animation: presence-pulse 1.6s ease-in-out infinite;
}
.presence-dot--online {
	background: #22c55e;
}
@keyframes presence-pulse {
	0%,
	100% {
		box-shadow: 0 0 0 0 color-mix(in srgb, #00e5ff 55%, transparent);
	}
	50% {
		box-shadow: 0 0 0 4px color-mix(in srgb, #00e5ff 0%, transparent);
	}
}
@media (prefers-reduced-motion: reduce) {
	.presence-dot--playing {
		animation: none;
	}
	.friends-skeleton .animate-pulse {
		animation: none;
	}
}
</style>
