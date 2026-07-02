<script setup lang="ts">
import { MailIcon, SearchIcon, SendIcon, UserIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	StyledInput,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, ref } from 'vue'

import FriendsSection from '@/components/ui/friends/FriendsSection.vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { friend_listener } from '@/helpers/events'
import {
	acceptCachedFriend,
	add_friend,
	createPendingFriend,
	friendsQueryKey,
	type FriendWithUserData,
	getFriendsWithUserData,
	remove_friend,
	removeCachedFriend,
	upsertCachedFriend,
} from '@/helpers/friends.ts'
import type { ModrinthCredentials } from '@/helpers/mr_auth'

const { formatMessage } = useVIntl()

const { handleError } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()
const queryClient = useQueryClient()

const props = defineProps<{
	credentials: ModrinthCredentials | null
	signIn: () => void
}>()

const userCredentials = computed(() => props.credentials)
const friendsKey = computed(() => friendsQueryKey(userCredentials.value?.user_id))

const search = ref('')
const friendInvitesModal = ref()

const username = ref('')
const addFriendModal = ref()

const friendsQuery = useQuery({
	queryKey: friendsKey,
	queryFn: () => getFriendsWithUserData(userCredentials.value),
	enabled: () => !!userCredentials.value,
	staleTime: 30_000,
})

const userFriends = computed(() => friendsQuery.data.value ?? [])
const sortedFriends = computed<FriendWithUserData[]>(() =>
	userFriends.value.slice().sort((a, b) => {
		if (a.last_updated === null && b.last_updated === null) {
			return 0 // Both are null, equal in sorting
		}
		if (a.last_updated === null) {
			return 1 // `a` is null, move it after `b`
		}
		if (b.last_updated === null) {
			return -1 // `b` is null, move it after `a`
		}
		// Both are non-null, sort by date
		return b.last_updated.diff(a.last_updated)
	}),
)
const filteredFriends = computed<FriendWithUserData[]>(() =>
	sortedFriends.value.filter((x) =>
		x.username.trim().toLowerCase().includes(search.value.trim().toLowerCase()),
	),
)

const activeFriends = computed<FriendWithUserData[]>(() =>
	filteredFriends.value.filter((x) => !!x.status && x.online && x.accepted),
)
const onlineFriends = computed<FriendWithUserData[]>(() =>
	filteredFriends.value.filter((x) => x.online && !x.status && x.accepted),
)
const offlineFriends = computed<FriendWithUserData[]>(() =>
	filteredFriends.value.filter((x) => !x.online && x.accepted),
)
const pendingFriends = computed(() =>
	filteredFriends.value
		.filter((x) => !x.accepted && x.id !== userCredentials.value?.user_id)
		.slice()
		.sort((a, b) => b.created.diff(a.created)),
)
const incomingRequests = computed(() =>
	userFriends.value
		.filter((x) => !x.accepted && x.id === userCredentials.value?.user_id)
		.slice()
		.sort((a, b) => b.created.diff(a.created)),
)

type FriendsMutationContext = {
	queryKey: ReturnType<typeof friendsQueryKey>
	previousFriends?: FriendWithUserData[]
}

type AddFriendMutationVariables = {
	userId: string
	user: {
		id: string
		username: string
		avatarUrl?: string | null
	}
	acceptExisting?: boolean
}

type RemoveFriendMutationVariables = {
	userId: string
	user: FriendWithUserData
}

const loading = computed(() => !!userCredentials.value && friendsQuery.isLoading.value)

const addFriendMutation = useMutation({
	mutationFn: ({ userId }: AddFriendMutationVariables) => add_friend(userId),
	onMutate: async ({ user, acceptExisting }): Promise<FriendsMutationContext> => {
		const queryKey = friendsKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousFriends = queryClient.getQueryData<FriendWithUserData[]>(queryKey)

		queryClient.setQueryData<FriendWithUserData[]>(queryKey, (friends = []) =>
			acceptExisting
				? acceptCachedFriend(friends, user.id, user.username, userCredentials.value?.user_id)
				: upsertCachedFriend(
						friends,
						createPendingFriend(user, userCredentials.value?.user_id),
						userCredentials.value?.user_id,
					),
		)

		return { queryKey, previousFriends }
	},
	onError: (error, _variables, context) => {
		restoreFriendsQuery(context)
		handleError(toError(error))
	},
	onSettled: (_data, _error, _variables, context) => {
		void queryClient.invalidateQueries({ queryKey: context?.queryKey ?? friendsKey.value })
	},
})

const removeFriendMutation = useMutation({
	mutationFn: ({ userId }: RemoveFriendMutationVariables) => remove_friend(userId),
	onMutate: async ({ user, userId }): Promise<FriendsMutationContext> => {
		const queryKey = friendsKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousFriends = queryClient.getQueryData<FriendWithUserData[]>(queryKey)

		queryClient.setQueryData<FriendWithUserData[]>(queryKey, (friends = []) =>
			removeCachedFriend(friends, userId, user.username, userCredentials.value?.user_id),
		)

		return { queryKey, previousFriends }
	},
	onError: (error, _variables, context) => {
		restoreFriendsQuery(context)
		handleError(toError(error))
	},
	onSettled: (_data, _error, _variables, context) => {
		void queryClient.invalidateQueries({ queryKey: context?.queryKey ?? friendsKey.value })
	},
})

function restoreFriendsQuery(context?: FriendsMutationContext) {
	if (!context) return

	if (context.previousFriends === undefined) {
		queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
		return
	}

	queryClient.setQueryData(context.queryKey, context.previousFriends)
}

function toError(error: unknown) {
	if (error instanceof Error) return error
	if (typeof error === 'string') return new Error(error)
	if (error && typeof error === 'object') {
		const record = error as Record<string, unknown>
		const message = record.message ?? record.error
		if (typeof message === 'string') return new Error(message)
		return new Error(JSON.stringify(error))
	}
	return new Error(String(error))
}

function addFriendFromModal() {
	const target = username.value.trim()
	if (!target) return

	addFriendModal.value.hide()
	addFriendMutation.mutate({
		userId: target,
		user: {
			id: target,
			username: target,
		},
	})
	username.value = ''
}

function addFriend(friend: FriendWithUserData) {
	const id = friend.id === userCredentials.value?.user_id ? friend.friend_id : friend.id
	if (id) {
		addFriendMutation.mutate({
			userId: id,
			user: {
				id,
				username: friend.username,
				avatarUrl: friend.avatar,
			},
			acceptExisting: true,
		})
	}
}

function removeFriend(friend: FriendWithUserData) {
	const id = friend.id === userCredentials.value?.user_id ? friend.friend_id : friend.id
	if (id) {
		removeFriendMutation.mutate({
			userId: id,
			user: friend,
		})
	}
}

const unlisten = await friend_listener(() => {
	void queryClient.invalidateQueries({ queryKey: friendsKey.value })
})
onUnmounted(() => {
	unlisten()
})

const messages = defineMessages({
	addFriend: {
		id: 'friends.action.add-friend',
		defaultMessage: 'Add a friend',
	},
	addingAFriend: {
		id: 'friends.add-friend.title',
		defaultMessage: 'Adding a friend',
	},
	usernameTitle: {
		id: 'friends.add-friend.username.title',
		defaultMessage: "What's your friend's Modrinth username?",
	},
	usernameDescription: {
		id: 'friends.add-friend.username.description',
		defaultMessage: 'It may be different from their Minecraft username!',
	},
	usernamePlaceholder: {
		id: 'friends.add-friend.username.placeholder',
		defaultMessage: 'Enter Modrinth username...',
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
	friends: {
		id: 'friends.heading',
		defaultMessage: 'Friends',
	},
	pending: {
		id: 'friends.heading.pending',
		defaultMessage: 'Pending',
	},
	active: {
		id: 'friends.heading.active',
		defaultMessage: 'Active',
	},
	online: {
		id: 'friends.heading.online',
		defaultMessage: 'Online',
	},
	offline: {
		id: 'friends.heading.offline',
		defaultMessage: 'Offline',
	},
	noFriendsMatch: {
		id: 'friends.no-friends-match',
		defaultMessage: `No friends matching ''{query}''`,
	},
	signInToAddFriends: {
		id: 'friends.sign-in-to-add-friends',
		defaultMessage:
			"<link>Sign in to a Modrinth account</link> to add friends and see what they're playing!",
	},
	addFriendsToShare: {
		id: 'friends.add-friends-to-share',
		defaultMessage: "<link>Add friends</link> to see what they're playing!",
	},
})
</script>

<template>
	<ModalWrapper ref="friendInvitesModal" header="View friend requests">
		<p v-if="incomingRequests.length === 0">You have no pending friend requests :C</p>
		<div v-else class="flex flex-col gap-4 min-w-[40rem]">
			<div v-for="friend in incomingRequests" :key="friend.username" class="flex gap-2">
				<Avatar :src="friend.avatar" class="w-12 h-12 rounded-full" size="2.25rem" circle />
				<div class="grid grid-cols-[1fr_auto] w-full gap-4">
					<div>
						<p class="m-0">
							<template v-if="friend.id === userCredentials?.user_id">
								<span class="text-contrast">{{ friend.username }}</span> sent you a friend request
							</template>
							<template v-else>
								You sent <span class="font-bold">{{ friend.username }}</span> a friend request
							</template>
						</p>
						<p class="m-0 text-sm text-secondary">
							{{ formatRelativeTime(friend.created.toISOString()) }}
						</p>
					</div>
					<div class="flex gap-2">
						<template v-if="friend.id === userCredentials?.user_id">
							<ButtonStyled color="brand">
								<button @click="addFriend(friend)">
									<UserPlusIcon />
									Accept
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<button @click="removeFriend(friend)">
									<XIcon />
									Ignore
								</button>
							</ButtonStyled>
						</template>
						<template v-else>
							<ButtonStyled>
								<button @click="removeFriend(friend)">
									<XIcon />
									Cancel
								</button>
							</ButtonStyled>
						</template>
					</div>
				</div>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper ref="addFriendModal" :header="formatMessage(messages.addingAFriend)">
		<div class="min-w-[30rem]">
			<h2 class="m-0 text-base font-medium text-primary">
				{{ formatMessage(messages.usernameTitle) }}
			</h2>
			<p class="m-0 mt-1 text-sm text-secondary leading-tight">
				{{ formatMessage(messages.usernameDescription) }}
			</p>
			<div class="flex items-center gap-2 mt-4">
				<StyledInput
					v-model="username"
					:icon="UserIcon"
					type="text"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					wrapper-class="flex-1"
					@keyup.enter="addFriendFromModal"
				/>
				<ButtonStyled color="brand">
					<button :disabled="username.length === 0" @click="addFriendFromModal">
						<SendIcon />
						{{ formatMessage(messages.sendFriendRequest) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</ModalWrapper>
	<div v-if="userCredentials && !loading" class="flex gap-1 items-center mb-3 -ml-1">
		<template v-if="sortedFriends.length > 0">
			<ButtonStyled circular type="transparent">
				<button
					v-tooltip="formatMessage(messages.addFriend)"
					:aria-label="formatMessage(messages.addFriend)"
					@click="addFriendModal.show"
				>
					<UserPlusIcon />
				</button>
			</ButtonStyled>
			<StyledInput
				v-model="search"
				:icon="SearchIcon"
				type="text"
				:placeholder="formatMessage(messages.searchFriends)"
				clearable
				input-class="!bg-transparent !border !border-solid !border-button-bg !text-primary !placeholder:text-primary"
				wrapper-class="flex-1 [&>svg]:!text-primary [&>svg]:!opacity-100"
				@keyup.esc="search = ''"
			/>
		</template>
		<h3 v-else class="w-full text-base text-primary font-medium m-0">
			{{ formatMessage(messages.friends) }}
		</h3>
		<ButtonStyled v-if="incomingRequests.length > 0" circular type="transparent">
			<button
				v-tooltip="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
				class="relative"
				:aria-label="formatMessage(messages.viewFriendRequests, { count: incomingRequests.length })"
				@click="friendInvitesModal.show"
			>
				<MailIcon />
				<span
					v-if="incomingRequests.length > 0"
					aria-hidden="true"
					class="absolute bg-brand text-brand-inverted text-[8px] top-0.5 px-1 right-0.5 min-w-3 h-3 rounded-full flex items-center justify-center font-bold"
				>
					{{ incomingRequests.length }}
				</span>
			</button>
		</ButtonStyled>
	</div>
	<div class="flex flex-col gap-3">
		<h3 v-if="loading" class="text-base text-primary font-medium m-0">
			{{ formatMessage(messages.friends) }}
		</h3>
		<template v-if="loading">
			<div v-for="n in 5" :key="n" class="flex gap-2 items-center animate-pulse">
				<div class="min-w-9 min-h-9 bg-button-bg rounded-full"></div>
				<div class="flex flex-col w-full">
					<div class="h-3 bg-button-bg rounded-full w-1/2 mb-1"></div>
					<div class="h-2.5 bg-button-bg rounded-full w-3/4"></div>
				</div>
			</div>
		</template>
		<template v-else-if="sortedFriends.length === 0">
			<div class="text-sm">
				<div v-if="!userCredentials">
					<IntlFormatted :message-id="messages.signInToAddFriends">
						<template #link="{ children }">
							<span class="font-semibold text-brand cursor-pointer" @click="signIn">
								<component :is="() => children" />
							</span>
						</template>
					</IntlFormatted>
				</div>
				<div v-else>
					<IntlFormatted :message-id="messages.addFriendsToShare">
						<template #link="{ children }">
							<span class="font-semibold text-brand cursor-pointer" @click="addFriendModal.show">
								<component :is="() => children" />
							</span>
						</template>
					</IntlFormatted>
				</div>
			</div>
		</template>
		<template v-else>
			<FriendsSection
				v-if="activeFriends.length > 0"
				:is-searching="!!search"
				open-by-default
				:friends="activeFriends"
				:heading="formatMessage(messages.active)"
				:remove-friend="removeFriend"
			/>
			<FriendsSection
				v-if="onlineFriends.length > 0"
				:is-searching="!!search"
				open-by-default
				:friends="onlineFriends"
				:heading="formatMessage(messages.online)"
				:remove-friend="removeFriend"
			/>
			<FriendsSection
				v-if="offlineFriends.length > 0"
				:is-searching="!!search"
				:open-by-default="activeFriends.length + onlineFriends.length < 3"
				:friends="offlineFriends"
				:heading="formatMessage(messages.offline)"
				:remove-friend="removeFriend"
			/>
			<FriendsSection
				v-if="pendingFriends.length > 0"
				:is-searching="!!search"
				:friends="pendingFriends"
				:heading="formatMessage(messages.pending)"
				:remove-friend="removeFriend"
			/>
			<p v-if="filteredFriends.length === 0 && search" class="text-sm text-secondary my-1 mx-4">
				{{ formatMessage(messages.noFriendsMatch, { query: search }) }}
			</p>
		</template>
	</div>
</template>
