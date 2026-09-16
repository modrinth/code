<script setup lang="ts">
import { MailIcon, SearchIcon, SendIcon, UserIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	IntlFormatted,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
	acceptOwyxFriend,
	listOwyxFriends,
	type OwyxFriend,
	removeOwyxFriend,
	requestOwyxFriend,
} from '@/helpers/owyx-friends'
import type { ModrinthCredentials } from '@/helpers/mr_auth'
import { resolveOwyxAvatarUrl } from '@/helpers/owyx-avatar'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const props = defineProps<{
	/** Legacy Modrinth credentials — ignored; friends are Owyx-backed. */
	credentials: ModrinthCredentials | null
	owyxSignedIn?: boolean
	signIn: () => void
}>()

const friends = ref<OwyxFriend[]>([])
const loading = ref(false)
const search = ref('')
const username = ref('')
const friendInvitesModal = ref<{ show: () => void; hide: () => void } | null>(null)
const addFriendModal = ref<{ show: () => void; hide: () => void } | null>(null)

async function refresh() {
	if (!props.owyxSignedIn) {
		friends.value = []
		return
	}
	loading.value = true
	try {
		friends.value = await listOwyxFriends()
	} catch (e) {
		handleError(e)
		friends.value = []
	} finally {
		loading.value = false
	}
}

onMounted(() => {
	void refresh()
})
watch(
	() => props.owyxSignedIn,
	() => {
		void refresh()
	},
)

const filtered = computed(() =>
	friends.value.filter((f) =>
		f.nickname.toLowerCase().includes(search.value.trim().toLowerCase()),
	),
)
const accepted = computed(() => filtered.value.filter((f) => f.status === 'accepted'))
const pendingOutgoing = computed(() =>
	filtered.value.filter((f) => f.status === 'pending' && !f.incoming),
)
const incomingRequests = computed(() =>
	friends.value.filter((f) => f.status === 'pending' && f.incoming),
)

function showAddFriendModal() {
	username.value = ''
	addFriendModal.value?.show()
}

async function addFriendFromModal() {
	const nick = username.value.trim()
	if (!nick) return
	addFriendModal.value?.hide()
	try {
		await requestOwyxFriend(nick)
		username.value = ''
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function acceptIncoming(friend: OwyxFriend) {
	try {
		await acceptOwyxFriend(friend.id)
		await refresh()
	} catch (e) {
		handleError(e)
	}
}

async function removeFriend(friend: OwyxFriend) {
	try {
		await removeOwyxFriend(friend.id)
		await refresh()
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
		defaultMessage: 'Use the nickname from their owyx.site account.',
	},
	usernamePlaceholder: {
		id: 'friends.add-friend.username.placeholder',
		defaultMessage: 'Enter Owyx nickname...',
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
	pending: { id: 'friends.heading.pending', defaultMessage: 'Pending' },
	offline: { id: 'friends.heading.offline', defaultMessage: 'Friends' },
	presenceSoon: {
		id: 'friends.presence-coming-soon',
		defaultMessage: 'Live presence (“what they’re playing”) is coming soon.',
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
		defaultMessage: '<link>Add friends</link> by Owyx nickname.',
	},
})
</script>

<template>
	<ModalWrapper ref="friendInvitesModal" header="Friend requests">
		<p v-if="incomingRequests.length === 0">No pending requests.</p>
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
						<span class="text-contrast font-medium">{{ friend.nickname }}</span> sent a request
					</p>
				</div>
				<div class="flex gap-2">
					<Button type="colored" color="brand" @click="acceptIncoming(friend)">
						<UserPlusIcon />
						Accept
					</Button>
					<Button @click="removeFriend(friend)">
						<XIcon />
						Decline
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
					@keyup.enter="addFriendFromModal"
				/>
				<Button
					type="colored"
					color="brand"
					:disabled="username.length === 0"
					@click="addFriendFromModal"
				>
					<SendIcon />
					{{ formatMessage(messages.sendFriendRequest) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>

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
			v-if="incomingRequests.length > 0"
			v-tooltip="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
			type="quiet"
			:label="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
			class="relative"
			@click="friendInvitesModal?.show()"
		>
			<span
				class="absolute -top-0.5 -right-0.5 bg-brand text-inverted rounded-full size-4 text-[10px] flex items-center justify-center pointer-events-none"
			>
				{{ incomingRequests.length }}
			</span>
			<MailIcon />
		</IconButton>
	</div>

	<div v-if="owyxSignedIn && !loading" class="friends-list">
		<p class="m-0 mb-3 text-xs text-secondary">{{ formatMessage(messages.presenceSoon) }}</p>
		<template v-if="friends.length > 0">
			<p v-if="filtered.length === 0" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.noFriendsMatch, { query: search }) }}
			</p>
			<template v-else>
				<div v-if="accepted.length > 0" class="mb-3">
					<h4 class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.offline) }} ({{ accepted.length }})
					</h4>
					<div
						v-for="friend in accepted"
						:key="friend.id"
						class="flex items-center gap-2 py-1.5"
					>
						<Avatar :src="resolveOwyxAvatarUrl(friend.avatarUrl)" size="1.75rem" circle />
						<span class="flex-1 truncate text-sm text-primary">{{ friend.nickname }}</span>
						<Button class="!px-2 !py-1" @click="removeFriend(friend)">
							<XIcon class="h-3.5 w-3.5" />
						</Button>
					</div>
				</div>
				<div v-if="pendingOutgoing.length > 0">
					<h4 class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.pending) }} ({{ pendingOutgoing.length }})
					</h4>
					<div
						v-for="friend in pendingOutgoing"
						:key="friend.id"
						class="flex items-center gap-2 py-1.5"
					>
						<Avatar :src="resolveOwyxAvatarUrl(friend.avatarUrl)" size="1.75rem" circle />
						<span class="flex-1 truncate text-sm text-secondary">{{ friend.nickname }}</span>
						<Button class="!px-2 !py-1" @click="removeFriend(friend)">
							<XIcon class="h-3.5 w-3.5" />
						</Button>
					</div>
				</div>
			</template>
		</template>
		<div v-else class="text-secondary text-sm">
			<IntlFormatted :message-id="messages.addFriendsToShare">
				<template #link="{ children }">
					<button class="text-link cursor-pointer bg-transparent border-none p-0" @click="showAddFriendModal">
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
