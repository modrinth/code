<template>
	<div class="flex flex-col gap-4">
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="inviteModalHeader"
			:friends="inviteFriends"
			:search-users="searchInviteUsers"
			:user-profile-link="userProfileLink"
			@invite="invitePlayer"
			@cancel="cancelInvite"
		/>
		<ConfirmUnlinkModal
			ref="shareUnlinkModal"
			mode="share-instance"
			:backup-tip="importedModpackBackupTip"
			@unlink="unlinkImportedModpackForShare"
		/>

		<template v-if="rows.length > 0">
			<div class="flex flex-col gap-4">
				<div class="flex items-center gap-2">
					<StyledInput
						v-model="memberSearch"
						:icon="SearchIcon"
						:placeholder="`Search ${rows.length} users...`"
						wrapper-class="min-w-0 flex-1"
						input-class="!h-10"
						clearable
					/>
					<ButtonStyled color="brand">
						<button
							class="flex !h-10 shrink-0 items-center gap-2"
							@click="showInvitePlayers($event)"
						>
							<UserPlusIcon aria-hidden="true" />
							{{ formatMessage(messages.inviteFriendsButton) }}
						</button>
					</ButtonStyled>
				</div>

				<div class="flex flex-wrap items-center gap-1.5">
					<FilterIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
					<button
						class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
						:class="
							methodFilter === 'all'
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
						:aria-pressed="methodFilter === 'all'"
						@click="methodFilter = 'all'"
					>
						All
					</button>
					<button
						v-for="option in methodFilterOptions"
						:key="option.id"
						class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
						:class="
							methodFilter === option.id
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
						:aria-pressed="methodFilter === option.id"
						@click="toggleMethodFilter(option.id)"
					>
						{{ option.label }}
					</button>
				</div>
			</div>

			<Table
				v-model:sort-column="sortColumn"
				v-model:sort-direction="sortDirection"
				:columns="columns"
				:data="sortedRows"
				row-key="id"
				table-min-width="42rem"
				@sort="handleTableSort"
			>
				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						No users match your filters.
					</div>
				</template>

				<template #cell-username="{ row }">
					<div class="flex min-w-0 max-w-full items-center gap-2">
						<AutoLink
							v-tooltip="truncatedTooltip(usernameRefs[row.id], row.username)"
							:to="userProfileLink(row.username)"
							class="inline-flex max-w-full min-w-0 items-center gap-2 text-primary hover:underline"
						>
							<Avatar
								:src="row.avatarUrl"
								:alt="`${row.username}'s avatar`"
								:tint-by="row.username"
								size="24px"
								circle
								no-shadow
							/>
							<span
								:ref="(element) => setUsernameRef(row.id, element)"
								class="min-w-0 truncate font-medium"
							>
								{{ row.username }}
							</span>
						</AutoLink>
					</div>
				</template>

				<template #cell-lastPlayed="{ row }">
					<span v-if="row.lastPlayedAt" v-tooltip="formatDateTime(row.lastPlayedAt)">
						{{ formatCompactRelativeTime(row.lastPlayedAt) }}
					</span>
					<span v-else>Never</span>
				</template>

				<template #cell-joined="{ row }">
					<span
						v-if="row.pending"
						class="inline-flex h-7 items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
					>
						Pending
					</span>
					<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">
						{{ formatCompactRelativeTime(row.joinedAt) }}
					</span>
				</template>

				<template #cell-method="{ row }">
					<span class="inline-flex min-w-0 items-center gap-2">
						<UserPlusIcon
							v-if="row.method === 'direct'"
							class="size-5 shrink-0"
							aria-hidden="true"
						/>
						<LinkIcon v-else class="size-5 shrink-0" aria-hidden="true" />
						<span class="min-w-0 truncate">{{ compactMethodLabels[row.method] }}</span>
					</span>
				</template>

				<template #cell-actions="{ row }">
					<div class="flex items-center justify-end">
						<ButtonStyled v-if="row.pending" circular type="transparent">
							<button
								v-tooltip="'Revoke invite'"
								:aria-label="`Revoke invite for ${row.username}`"
								class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
								@click="removeRow(row.id)"
							>
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
						<ButtonStyled v-else circular type="transparent">
							<OverflowMenu
								:options="[
									{
										id: 'remove-user',
										action: () => removeRow(row.id),
										color: 'red',
									},
								]"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<span class="sr-only">Actions for {{ row.username }}</span>
								<template #remove-user>
									<XIcon aria-hidden="true" />
									Revoke access
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		</template>

		<EmptyState v-else-if="!isSignedIn" type="empty-inbox">
			<template #heading>{{ formatMessage(messages.signInToShareHeading) }}</template>
			<template #description>{{ formatMessage(messages.signInToShareDescription) }}</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button class="!h-10" @click="signInToShare">
						<LogInIcon aria-hidden="true" />
						{{ formatMessage(messages.signInButton) }}
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>

		<EmptyState v-else type="empty-inbox">
			<template #heading>{{ formatMessage(messages.noFriendsInvitedHeading) }}</template>
			<template #description>{{ formatMessage(messages.noFriendsInvitedDescription) }}</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button class="!h-10" @click="showInvitePlayers($event)">
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.inviteFriendsButton) }}
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>
	</div>
</template>

<script setup lang="ts">
import {
	FilterIcon,
	LinkIcon,
	LogInIcon,
	MoreVerticalIcon,
	SearchIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	ButtonStyled,
	ConfirmUnlinkModal,
	defineMessages,
	EmptyState,
	injectAuth,
	injectNotificationManager,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersSearchUser,
	type InvitePlayersUser,
	OverflowMenu,
	provideAppBackup,
	type SortDirection,
	StyledInput,
	Table,
	type TableColumn,
	truncatedTooltip,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onUnmounted, ref, watch } from 'vue'

import { get_user_many } from '@/helpers/cache.js'
import { friend_listener } from '@/helpers/events.js'
import {
	add_friend,
	createPendingFriend,
	friendsQueryKey,
	type FriendWithUserData,
	getFriendsWithUserData,
	getFriendUserId,
	upsertCachedFriend,
} from '@/helpers/friends.ts'
import { install_duplicate_instance, installJobInstanceId } from '@/helpers/install'
import {
	edit,
	get_shared_instance_users,
	invite_shared_instance_users,
	list,
	remove_shared_instance_users,
	type SharedInstanceUsers,
} from '@/helpers/instance'
import { get as getCredentials } from '@/helpers/mr_auth.ts'
import type { GameInstance } from '@/helpers/types'
import { search_user } from '@/helpers/users.ts'

type ShareMethod = 'direct' | 'link'
type MethodFilter = ShareMethod | 'all'
type ShareTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

type ShareRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: ShareMethod
	pending?: boolean
}

const props = defineProps<{
	instance: GameInstance
}>()

const auth = injectAuth()
const { handleError } = injectNotificationManager()
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal> | null>(null)
const shareUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal> | null>(null)
const memberSearch = ref('')
const methodFilter = ref<MethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const importedModpackUnlinkedForShare = ref(false)

const pendingRows = ref<Record<string, ShareRow>>({})

const messages = defineMessages({
	signInToShareHeading: {
		id: 'app.instance.share.sign-in.heading',
		defaultMessage: 'Sign in to share',
	},
	signInToShareDescription: {
		id: 'app.instance.share.sign-in.description',
		defaultMessage: 'You need a Modrinth account to share instances.',
	},
	signInButton: {
		id: 'app.instance.share.sign-in.button',
		defaultMessage: 'Sign in',
	},
	noFriendsInvitedHeading: {
		id: 'app.instance.share.empty.heading',
		defaultMessage: 'No friends invited',
	},
	noFriendsInvitedDescription: {
		id: 'app.instance.share.empty.description',
		defaultMessage: 'You can share this instance with your friends!',
	},
	inviteFriendsButton: {
		id: 'app.instance.share.empty.invite-friends-button',
		defaultMessage: 'Invite friends',
	},
	shareModalHeader: {
		id: 'app.instance.share.invite-modal.heading',
		defaultMessage: 'Share {name}',
	},
})

const { formatMessage } = useVIntl()
const formatCompactRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const queryClient = useQueryClient()
const currentUserId = computed(() => auth.user.value?.id ?? null)
const isSignedIn = computed(() => !!auth.session_token.value)
const inviteModalHeader = computed(() =>
	formatMessage(messages.shareModalHeader, { name: props.instance.name }),
)
const shareRoutePath = computed(() => `/instance/${encodeURIComponent(props.instance.id)}/share`)
const friendsKey = computed(() => friendsQueryKey(currentUserId.value))
const sharedUsersKey = computed(() => ['sharedInstanceUsers', props.instance.id] as const)
const requiresUnlinkBeforeShare = computed(
	() =>
		props.instance.link?.type === 'imported_modpack' &&
		!props.instance.shared_instance &&
		!importedModpackUnlinkedForShare.value,
)
const importedModpackBackupTip = computed(() => {
	const link = props.instance.link
	if (link?.type !== 'imported_modpack') return undefined

	return link.name ?? link.filename ?? undefined
})
const friendsQuery = useQuery({
	queryKey: friendsKey,
	queryFn: async () => getFriendsWithUserData(await getCredentials()),
	enabled: () => isSignedIn.value && !!currentUserId.value,
	staleTime: 30_000,
})
const userFriends = computed(() => friendsQuery.data.value ?? [])
const sharedUsersQuery = useQuery({
	queryKey: sharedUsersKey,
	queryFn: loadSharedRows,
	enabled: () => isSignedIn.value && !!props.instance.id,
	staleTime: Infinity,
})
const sharedRows = computed(() => sharedUsersQuery.data.value ?? [])
const rows = computed(() => {
	const sharedKeys = new Set(sharedRows.value.map((row) => normalizeInviteKey(row.id)))
	const pending = Object.values(pendingRows.value).filter(
		(row) => !sharedKeys.has(normalizeInviteKey(row.id)),
	)

	return [...pending, ...sharedRows.value]
})

const methodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const compactMethodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const columns: TableColumn<ShareTableColumn>[] = [
	{
		key: 'username',
		label: 'Username',
		width: 'clamp(14rem, 30%, 26rem)',
		enableSorting: true,
		headerClass: '!pr-3',
		cellClass: '!pr-3',
	},
	{
		key: 'lastPlayed',
		label: 'Last played',
		width: 'clamp(7rem, 15%, 13rem)',
		enableSorting: true,
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'joined',
		label: 'Joined',
		width: 'clamp(7rem, 14%, 12rem)',
		enableSorting: true,
		defaultSortDirection: 'desc',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'method',
		label: 'Method',
		enableSorting: true,
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'actions',
		label: 'Actions',
		align: 'right',
		width: 'clamp(5.5rem, 7%, 7rem)',
		headerClass: 'whitespace-nowrap !pl-2 !pr-4',
		cellClass: 'whitespace-nowrap !pl-2 !pr-4',
	},
]

const methodFilterOptions = computed<Array<{ id: ShareMethod; label: string }>>(() => [
	{ id: 'direct', label: methodLabels.direct },
	{ id: 'link', label: methodLabels.link },
])

const invitedRows = computed(() => {
	const invited = new Map<string, ShareRow>()

	for (const row of rows.value) {
		invited.set(normalizeInviteKey(row.id), row)
		invited.set(normalizeInviteKey(row.username), row)
	}

	return invited
})

const inviteFriends = computed<InvitePlayersUser[]>(() =>
	userFriends.value
		.filter((friend) => friend.username)
		.map((friend) => {
			const id = getFriendUserId(friend, currentUserId.value)
			const invitedRow =
				invitedRows.value.get(normalizeInviteKey(id)) ??
				invitedRows.value.get(normalizeInviteKey(friend.username))

			return {
				id,
				username: friend.username,
				avatarUrl: friend.avatar,
				online: friend.online,
				status: friend.accepted
					? invitedRow
						? invitedRow.pending
							? 'pending'
							: 'added'
						: 'available'
					: 'requested',
			} satisfies InvitePlayersUser
		}),
)

const inviteFriendKeys = computed(() => {
	const keys = new Set<string>()

	for (const friend of inviteFriends.value) {
		keys.add(normalizeInviteKey(friend.id))
		keys.add(normalizeInviteKey(friend.username))
	}

	return keys
})

const filteredRows = computed(() => {
	const normalizedSearch = memberSearch.value.trim().toLowerCase()

	return rows.value.filter((row) => {
		if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
		if (!normalizedSearch) return true

		return [
			row.username,
			formattedLastPlayed(row),
			formattedJoined(row),
			methodLabels[row.method],
			compactMethodLabels[row.method],
		].some((value) => value.toLowerCase().includes(normalizedSearch))
	})
})

const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

function compareRows(a: ShareRow, b: ShareRow) {
	const column = sortColumn.value
	const direction = sortDirection.value

	switch (column) {
		case 'username':
			return compareString(a.username, b.username, direction)
		case 'lastPlayed':
			return compareDate(a.lastPlayedAt, b.lastPlayedAt, direction)
		case 'method':
			return compareString(compactMethodLabels[a.method], compactMethodLabels[b.method], direction)
		default:
			return compareJoined(a, b, direction)
	}
}

function compareJoined(a: ShareRow, b: ShareRow, direction: SortDirection) {
	const compared = joinedSortValue(a) - joinedSortValue(b) || a.username.localeCompare(b.username)
	return direction === 'asc' ? compared : -compared
}

function joinedSortValue(row: ShareRow) {
	return row.pending
		? Number.MAX_SAFE_INTEGER
		: (row.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)
}

function compareDate(a: Date | null, b: Date | null, direction: SortDirection) {
	const compared = compareDateValue(a, b)
	return direction === 'asc' ? compared : -compared
}

function compareDateValue(a: Date | null, b: Date | null) {
	const aTime = a?.getTime() ?? Number.NEGATIVE_INFINITY
	const bTime = b?.getTime() ?? Number.NEGATIVE_INFINITY
	return aTime - bTime
}

function compareString(a: string, b: string, direction: SortDirection) {
	const compared = a.localeCompare(b)
	return direction === 'asc' ? compared : -compared
}

function handleTableSort(column: string, direction: SortDirection) {
	sortColumn.value = column
	sortDirection.value = direction
}

function toggleMethodFilter(filter: ShareMethod) {
	methodFilter.value = methodFilter.value === filter ? 'all' : filter
}

async function searchInviteUsers(query: string): Promise<InvitePlayersSearchUser[]> {
	const users = await search_user(query)

	return users
		.filter((user) => user.id !== currentUserId.value)
		.filter(
			(user) =>
				!inviteFriendKeys.value.has(normalizeInviteKey(user.id)) &&
				!inviteFriendKeys.value.has(normalizeInviteKey(user.username)),
		)
		.map((user) => ({
			id: user.id,
			username: user.username,
			avatarUrl: user.avatar_url || undefined,
		}))
}

type FriendsMutationContext = {
	queryKey: ReturnType<typeof friendsQueryKey>
	previousFriends?: FriendWithUserData[]
}

type SharedRowsMutationContext = {
	queryKey: typeof sharedUsersKey.value
	previousRows?: ShareRow[]
}

const addFriendMutation = useMutation({
	mutationFn: (user: InvitePlayersUser) => add_friend(user.id),
	onMutate: async (user): Promise<FriendsMutationContext> => {
		const queryKey = friendsKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousFriends = queryClient.getQueryData<FriendWithUserData[]>(queryKey)

		queryClient.setQueryData<FriendWithUserData[]>(queryKey, (friends = []) =>
			upsertCachedFriend(
				friends,
				createPendingFriend(user, currentUserId.value),
				currentUserId.value,
			),
		)

		return { queryKey, previousFriends }
	},
	onError: (error, _user, context) => {
		restoreFriendsQuery(context)
		handleError(toError(error))
	},
	onSettled: (_data, _error, _user, context) => {
		void queryClient.invalidateQueries({ queryKey: context?.queryKey ?? friendsKey.value })
	},
})

const inviteShareMutation = useMutation({
	mutationFn: async (user: InvitePlayersUser) => {
		await invite_shared_instance_users(props.instance.id, [user.id])
	},
	onMutate: async (user): Promise<SharedRowsMutationContext> => {
		const queryKey = sharedUsersKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousRows = queryClient.getQueryData<ShareRow[]>(queryKey)
		setPendingRow(user)

		return { queryKey, previousRows }
	},
	onError: (error, user, context) => {
		removePendingRow(user.id)
		restoreSharedRowsQuery(context)
		handleError(toError(error))
	},
	onSuccess: (_data, user) => {
		upsertSharedRow(inviteUserToShareRow(user))
		removePendingRow(user.id)
	},
})

const removeShareMutation = useMutation({
	mutationFn: async (id: string) => {
		return await remove_shared_instance_users(props.instance.id, [id])
	},
	onMutate: async (id): Promise<SharedRowsMutationContext> => {
		const queryKey = sharedUsersKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousRows = queryClient.getQueryData<ShareRow[]>(queryKey)
		queryClient.setQueryData<ShareRow[]>(queryKey, (rows = []) =>
			rows.filter((row) => normalizeInviteKey(row.id) !== normalizeInviteKey(id)),
		)

		return { queryKey, previousRows }
	},
	onError: (error, _id, context) => {
		restoreSharedRowsQuery(context)
		handleError(toError(error))
	},
	onSuccess: async (users) => {
		try {
			queryClient.setQueryData<ShareRow[]>(sharedUsersKey.value, await sharedUsersToRows(users))
		} catch (error) {
			handleError(toError(error))
			await queryClient.invalidateQueries({ queryKey: sharedUsersKey.value })
		}
	},
})

async function loadSharedRows(): Promise<ShareRow[]> {
	return sharedUsersToRows(await get_shared_instance_users(props.instance.id))
}

async function sharedUsersToRows(users: SharedInstanceUsers): Promise<ShareRow[]> {
	if (users.user_ids.length === 0) return []

	const profiles = (await get_user_many(users.user_ids)) as Array<{
		id: string
		username?: string
		avatar_url?: string | null
	}>

	return users.user_ids.map((id) => {
		const profile = profiles.find((user) => user.id === id)

		return {
			id,
			username: profile?.username ?? id,
			avatarUrl: profile?.avatar_url ?? undefined,
			lastPlayedAt: null,
			joinedAt: null,
			method: 'direct',
		} satisfies ShareRow
	})
}

function invitePlayer(payload: InvitePlayersInvitePayload) {
	const user = payload.user
	if (payload.source === 'search') {
		addPendingFriend(user)
	}

	inviteShareUser(user)
}

function inviteShareUser(user: InvitePlayersUser) {
	const existingRow = findInviteRow(user.id, user.username)
	if (existingRow) {
		return
	}

	inviteShareMutation.mutate(user)
}

function addPendingFriend(user: InvitePlayersUser) {
	if (findInviteFriend(user.id, user.username)) return

	addFriendMutation.mutate(user)
}

function cancelInvite(user: InvitePlayersUser) {
	const existingRow = findInviteRow(user.id, user.username)
	if (existingRow) {
		removeRow(existingRow.id)
	}
}

function showInvitePlayers(event?: MouseEvent) {
	if (requiresUnlinkBeforeShare.value) {
		shareUnlinkModal.value?.show()
		return
	}

	invitePlayersModal.value?.show(event)
}

async function unlinkImportedModpackForShare() {
	try {
		await edit(props.instance.id, {
			link: null as unknown as undefined,
		})
		importedModpackUnlinkedForShare.value = true
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', props.instance.id] })
		invitePlayersModal.value?.show()
	} catch (error) {
		handleError(toError(error))
	}
}

function removeRow(id: string) {
	if (pendingRows.value[id]) {
		removePendingRow(id)
		return
	}

	removeShareMutation.mutate(id)
}

function setPendingRow(user: InvitePlayersUser) {
	pendingRows.value = {
		...pendingRows.value,
		[user.id]: {
			id: user.id,
			username: user.username,
			avatarUrl: user.avatarUrl,
			lastPlayedAt: null,
			joinedAt: null,
			method: 'direct',
			pending: true,
		},
	}
}

function removePendingRow(id: string) {
	const { [id]: _removed, ...rest } = pendingRows.value
	pendingRows.value = rest
}

function inviteUserToShareRow(user: InvitePlayersUser): ShareRow {
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatarUrl ?? undefined,
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
	}
}

function upsertSharedRow(row: ShareRow) {
	queryClient.setQueryData<ShareRow[]>(sharedUsersKey.value, (rows = []) => {
		const normalizedId = normalizeInviteKey(row.id)
		const withoutExisting = rows.filter(
			(existing) => normalizeInviteKey(existing.id) !== normalizedId,
		)

		return [row, ...withoutExisting]
	})
}

function restoreFriendsQuery(context?: FriendsMutationContext) {
	if (!context) return

	if (context.previousFriends === undefined) {
		queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
		return
	}

	queryClient.setQueryData(context.queryKey, context.previousFriends)
}

function restoreSharedRowsQuery(context?: SharedRowsMutationContext) {
	if (!context) return

	if (context.previousRows === undefined) {
		queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
		return
	}

	queryClient.setQueryData(context.queryKey, context.previousRows)
}

function userProfileLink(username: string) {
	if (!username || username.includes('@')) return undefined
	return () => openUrl(`https://modrinth.com/user/${encodeURIComponent(username)}`)
}

function signInToShare() {
	void auth.requestSignIn(shareRoutePath.value)
}

function setUsernameRef(id: string, element: Element | null) {
	const nextElement = element instanceof HTMLElement ? element : null
	if (usernameRefs.value[id] === nextElement) return

	usernameRefs.value = {
		...usernameRefs.value,
		[id]: nextElement,
	}
}

function formattedLastPlayed(row: ShareRow) {
	return row.lastPlayedAt ? formatCompactRelativeTime(row.lastPlayedAt) : 'Never'
}

function formattedJoined(row: ShareRow) {
	return row.pending ? 'Pending' : row.joinedAt ? formatCompactRelativeTime(row.joinedAt) : ''
}

function findInviteRow(id: string, username: string) {
	const normalizedId = normalizeInviteKey(id)
	const normalizedUsername = normalizeInviteKey(username)

	return rows.value.find(
		(row) =>
			normalizeInviteKey(row.id) === normalizedId ||
			normalizeInviteKey(row.username) === normalizedUsername,
	)
}

function findInviteFriend(id: string, username: string) {
	const normalizedId = normalizeInviteKey(id)
	const normalizedUsername = normalizeInviteKey(username)

	return inviteFriends.value.find(
		(friend) =>
			normalizeInviteKey(friend.id) === normalizedId ||
			normalizeInviteKey(friend.username) === normalizedUsername,
	)
}

function normalizeInviteKey(value: string) {
	return value.trim().toLowerCase()
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

watch(
	() => props.instance.id,
	() => {
		importedModpackUnlinkedForShare.value = false
	},
)

provideAppBackup({
	async createBackup() {
		const allInstances = await list()
		const prefix = `${props.instance.name} - Backup #`
		const existingNums = allInstances
			.filter((instance) => instance.name.startsWith(prefix))
			.map((instance) => parseInt(instance.name.slice(prefix.length), 10))
			.filter((value) => !isNaN(value))
		const nextNum = existingNums.length > 0 ? Math.max(...existingNums) + 1 : 1
		const job = await install_duplicate_instance(props.instance.id)
		const newInstanceId = installJobInstanceId(job)
		if (newInstanceId) {
			await edit(newInstanceId, { name: `${prefix}${nextNum}` })
		}
	},
})

const unlistenFriends = await friend_listener(() => {
	void queryClient.invalidateQueries({ queryKey: friendsKey.value })
})
onUnmounted(() => {
	unlistenFriends()
})
</script>
