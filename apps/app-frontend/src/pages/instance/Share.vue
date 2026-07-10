<template>
	<div class="flex flex-col gap-4">
		<ModrinthAccountRequiredModal
			ref="modrinthAccountRequiredModal"
			:request-auth="requestAuthToShare"
		/>
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="inviteModalHeader"
			:friends="inviteFriends"
			:search-users="searchInviteUsers"
			:link="inviteLink"
			:link-expires-at="inviteLinkDetails?.expiresAt"
			:link-max-uses="inviteLinkDetails?.maxUses"
			:update-invite-link="updateInviteLink"
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
		<NewModal
			ref="removeUserConfirmModal"
			:header="formatMessage(messages.removeUserHeader)"
			max-width="470px"
			@after-hide="pendingRemovalRow = null"
		>
			<div class="flex flex-col gap-4">
				<Admonition type="warning">
					{{
						formatMessage(messages.removeUserWarningBody, {
							username: pendingRemovalUsername,
						})
					}}
				</Admonition>

				<div class="flex min-w-0 items-center gap-2 rounded-[20px] bg-surface-2 p-3">
					<Avatar
						:src="pendingRemovalRow?.avatarUrl"
						:alt="formatMessage(messages.userAvatarAlt, { username: pendingRemovalUsername })"
						:tint-by="pendingRemovalUsername"
						size="40px"
						circle
						no-shadow
					/>
					<div class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="min-w-0 truncate font-medium text-contrast">
							{{ pendingRemovalUsername }}
						</span>
						<span class="truncate text-sm text-secondary">
							{{ compactMethodLabels[pendingRemovalRow?.method ?? 'direct'] }}
						</span>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.removeUserEffectsLabel) }}
					</span>
					<ul class="m-0 list-disc pl-6 text-primary">
						<li
							v-for="effect in removeUserEffectMessages"
							:key="effect.id"
							class="leading-6 marker:text-secondary"
						>
							{{ formatMessage(effect) }}
						</li>
					</ul>
				</div>

				<div class="flex justify-end gap-2 pt-1">
					<ButtonStyled type="outlined">
						<button class="!border !border-surface-5" @click="hideRemoveUserModal">
							<XIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
						<button :disabled="!pendingRemovalRow" @click="confirmRemoveRow">
							<UserXIcon aria-hidden="true" />
							{{ formatMessage(messages.removeUserButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

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
					<ButtonStyled v-if="!sharedInstanceActionsLocked" color="brand">
						<button
							class="flex !h-10 shrink-0 items-center gap-2"
							:disabled="inviteLinkPending"
							@click="showInvitePlayers($event)"
						>
							<SpinnerIcon v-if="inviteLinkPending" class="animate-spin" aria-hidden="true" />
							<UserPlusIcon v-else aria-hidden="true" />
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
					<div v-if="!sharedInstanceActionsLocked" class="flex items-center justify-end">
						<ButtonStyled circular type="transparent">
							<button
								v-tooltip="'Revoke access'"
								:aria-label="`Revoke access for ${row.username}`"
								class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
								@click="showRemoveRowModal(row)"
							>
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		</template>

		<EmptyState v-else-if="sharedInstanceUnavailable" type="empty-inbox">
			<template #heading>{{ formatMessage(sharedInstanceErrorMessages.unavailableTitle) }}</template>
			<template #description>
				{{ formatSharedInstanceUnavailable(sharedInstanceUnavailableReason, sharedInstanceUnavailableManager) }}
			</template>
		</EmptyState>

		<EmptyState v-else-if="sharedInstanceActionsLocked" type="empty-inbox">
			<template #heading>{{ formatMessage(lockedEmptyHeading) }}</template>
			<template #description>
				<span class="flex flex-wrap items-center justify-center gap-x-1.5 gap-y-1">
					<span>{{ formatMessage(messages.lockedEmptyDescriptionPrefix) }}</span>
					<span
						v-if="linkedAccount"
						class="inline-flex max-w-full min-w-0 items-center gap-1.5 align-middle font-semibold text-primary"
					>
						<Avatar
							:src="linkedAccount.avatarUrl"
							:alt="linkedAccount.username"
							:tint-by="linkedAccount.tintBy"
							size="20px"
							circle
							no-shadow
						/>
						<span class="min-w-0 truncate">{{ linkedAccount.username }}</span>
					</span>
					<span v-else class="font-semibold text-primary">
						{{ formatMessage(messages.linkedAccountFallback) }}
					</span>
					<span>{{ formatMessage(messages.lockedEmptyDescriptionSuffix) }}</span>
				</span>
			</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button class="!h-10" @click="signInToShare">
						<LogInIcon aria-hidden="true" />
						{{ formatMessage(lockedActionButton) }}
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>

		<EmptyState v-else type="empty-inbox">
			<template #heading>{{ formatMessage(messages.noFriendsInvitedHeading) }}</template>
			<template #description>{{ formatMessage(messages.noFriendsInvitedDescription) }}</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button class="!h-10" :disabled="inviteLinkPending" @click="showInvitePlayers($event)">
						<SpinnerIcon v-if="inviteLinkPending" class="animate-spin" aria-hidden="true" />
						<UserPlusIcon v-else aria-hidden="true" />
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
	SearchIcon,
	SpinnerIcon,
	UserPlusIcon,
	UserXIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	AutoLink,
	Avatar,
	ButtonStyled,
	commonMessages,
	ConfirmUnlinkModal,
	defineMessages,
	EmptyState,
	injectAuth,
	injectNotificationManager,
	type InviteLinkSettings,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersSearchUser,
	type InvitePlayersUser,
	NewModal,
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

import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import { config } from '@/config'
import { get_user, get_user_many } from '@/helpers/cache.js'
import { friend_listener } from '@/helpers/events.js'
import {
	add_friend,
	friendsQueryKey,
	getFriendsWithUserData,
	getFriendUserId,
} from '@/helpers/friends.ts'
import {
	getSharedInstanceUnavailableReason,
	install_duplicate_instance,
	installJobInstanceId,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import {
	create_shared_instance_invite_link,
	edit,
	get_shared_instance_users,
	invite_shared_instance_users,
	list,
	remove_shared_instance_users,
	type SharedInstanceInviteLink,
	type SharedInstanceUser,
	type SharedInstanceUsers,
} from '@/helpers/instance'
import { get as getCredentials, type ModrinthAuthFlow } from '@/helpers/mr_auth.ts'
import {
	sharedInstanceErrorMessages,
	useSharedInstanceErrors,
} from '@/helpers/shared-instance-errors'
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
	sharedInstanceActionsLocked?: boolean
	sharedInstanceUnavailableReason?: SharedInstanceUnavailableReason | null
	sharedInstanceUnavailableManager?: string | null
}>()

const auth = injectAuth()
const { handleError } = injectNotificationManager()
const {
	formatSharedInstanceUnavailable,
	notifySharedInstanceError,
	notifySharedInstanceUnavailable,
} = useSharedInstanceErrors()
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal> | null>(null)
const modrinthAccountRequiredModal = ref<InstanceType<typeof ModrinthAccountRequiredModal> | null>(
	null,
)
const shareUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal> | null>(null)
const removeUserConfirmModal = ref<InstanceType<typeof NewModal> | null>(null)
const memberSearch = ref('')
const methodFilter = ref<MethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const importedModpackUnlinkedForShare = ref(false)
const pendingRemovalRow = ref<ShareRow | null>(null)
const inviteLinkDetails = ref<SharedInstanceInviteLink>()
const inviteLinkPending = ref(false)

const pendingRows = ref<Record<string, ShareRow>>(loadPendingRows(props.instance.id))

const messages = defineMessages({
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
	lockedWrongAccountHeading: {
		id: 'app.instance.share.locked.wrong-account-heading',
		defaultMessage: 'Wrong account',
	},
	lockedSignedOutHeading: {
		id: 'app.instance.share.locked.signed-out-heading',
		defaultMessage: 'Not signed in',
	},
	lockedEmptyDescriptionPrefix: {
		id: 'app.instance.share.locked.empty-description-prefix',
		defaultMessage: 'You need to sign in as',
	},
	lockedEmptyDescriptionSuffix: {
		id: 'app.instance.share.locked.empty-description-suffix',
		defaultMessage: 'to access this page.',
	},
	linkedAccountFallback: {
		id: 'app.instance.share.locked.linked-account-fallback',
		defaultMessage: 'the linked account',
	},
	switchAccountButton: {
		id: 'app.instance.share.locked.switch-account-button',
		defaultMessage: 'Switch account',
	},
	removeUserHeader: {
		id: 'app.instance.share.remove-user-modal.header',
		defaultMessage: 'Revoke access',
	},
	removeUserWarningBody: {
		id: 'app.instance.share.remove-user-modal.warning-body',
		defaultMessage:
			"If you revoke {username}'s access to this shared instance, you'll need to invite them again before they can receive updates.",
	},
	userAvatarAlt: {
		id: 'app.instance.share.remove-user-modal.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	removeUserEffectsLabel: {
		id: 'app.instance.share.remove-user-modal.effects-label',
		defaultMessage: 'What happens?',
	},
	removeUserEffectAccess: {
		id: 'app.instance.share.remove-user-modal.effect-access',
		defaultMessage: 'They will no longer receive updates for this shared instance',
	},
	removeUserEffectInstalledCopy: {
		id: 'app.instance.share.remove-user-modal.effect-installed-copy',
		defaultMessage: 'Any copy they already installed will stay on their device',
	},
	removeUserEffectInviteAgain: {
		id: 'app.instance.share.remove-user-modal.effect-invite-again',
		defaultMessage: 'You can invite them again later',
	},
	removeUserEffectLastUser: {
		id: 'app.instance.share.remove-user-modal.effect-last-user',
		defaultMessage: 'If this is the last user, sharing will be turned off for this instance',
	},
	removeUserButton: {
		id: 'app.instance.share.remove-user-modal.remove-button',
		defaultMessage: 'Revoke access',
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
const inviteLink = computed(() =>
	inviteLinkDetails.value ? buildInviteLink(inviteLinkDetails.value) : undefined,
)
const shareRoutePath = computed(() => `/instance/${encodeURIComponent(props.instance.id)}/share`)
const friendsKey = computed(() => friendsQueryKey(currentUserId.value))
const sharedUsersKey = computed(() => ['sharedInstanceUsers', props.instance.id] as const)
const linkedAccountId = computed(() => props.instance.shared_instance?.linked_user_id ?? null)
const linkedAccountQuery = useQuery({
	queryKey: computed(() => ['user', linkedAccountId.value]),
	queryFn: async () => {
		if (!linkedAccountId.value) return null

		return await get_user(linkedAccountId.value, 'bypass').catch(() => null)
	},
	enabled: () => !!linkedAccountId.value,
	staleTime: 30_000,
})
const linkedAccount = computed(() => {
	const user = linkedAccountQuery.data.value
	if (!user) return null

	return {
		username: user.username ?? user.id,
		avatarUrl: user.avatar_url ?? undefined,
		tintBy: user.id,
	}
})
const lockedEmptyHeading = computed(() =>
	isSignedIn.value ? messages.lockedWrongAccountHeading : messages.lockedSignedOutHeading,
)
const lockedActionButton = computed(() =>
	isSignedIn.value ? messages.switchAccountButton : messages.signInButton,
)
const sharedInstanceUnavailableReason = computed(
	() => props.sharedInstanceUnavailableReason ?? null,
)
const sharedInstanceUnavailable = computed(() => !!sharedInstanceUnavailableReason.value)
const sharedInstanceUnavailableManager = computed(
	() => props.sharedInstanceUnavailableManager ?? null,
)

function notifySharedInstanceOperationError(error: unknown) {
	if (isSharedInstanceUnavailableError(error)) {
		notifySharedInstanceUnavailable(
			getSharedInstanceUnavailableReason(error),
			sharedInstanceUnavailableManager.value,
		)
		return
	}

	notifySharedInstanceError(toError(error))
}

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
	enabled: () => isSignedIn.value && !!currentUserId.value && !props.sharedInstanceActionsLocked,
	staleTime: 30_000,
})
const userFriends = computed(() => friendsQuery.data.value ?? [])
const sharedUsersQuery = useQuery({
	queryKey: sharedUsersKey,
	queryFn: loadSharedRows,
	enabled: () => isSignedIn.value && !!props.instance.id && !props.sharedInstanceActionsLocked,
	staleTime: Infinity,
})
const sharedRows = computed(
	() =>
		sharedUsersQuery.data.value ?? queryClient.getQueryData<ShareRow[]>(sharedUsersKey.value) ?? [],
)
const rows = computed(() => {
	if (props.sharedInstanceActionsLocked) return sharedRows.value

	const sharedKeys = new Set(sharedRows.value.map((row) => normalizeInviteKey(row.id)))
	const pending = Object.values(pendingRows.value).filter(
		(row) => !sharedKeys.has(normalizeInviteKey(row.id)),
	)

	return [...pending, ...sharedRows.value]
})
const pendingRemovalUsername = computed(() => pendingRemovalRow.value?.username ?? '')
const removeUserEffectMessages = computed(() => {
	const effects = [
		messages.removeUserEffectAccess,
		messages.removeUserEffectInstalledCopy,
		messages.removeUserEffectInviteAgain,
	]

	if (rows.value.length === 1) {
		effects.push(messages.removeUserEffectLastUser)
	}

	return effects
})

const methodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const compactMethodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const columns = computed<TableColumn<ShareTableColumn>[]>(() => {
	const tableColumns: TableColumn<ShareTableColumn>[] = [
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
	]

	if (!props.sharedInstanceActionsLocked) {
		tableColumns.push({
			key: 'actions',
			label: 'Actions',
			align: 'right',
			width: 'clamp(5.5rem, 7%, 7rem)',
			headerClass: 'whitespace-nowrap !pl-2 !pr-4',
			cellClass: 'whitespace-nowrap !pl-2 !pr-4',
		})
	}

	return tableColumns
})

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
		.filter((friend) => friend.username && friend.accepted)
		.sort((a, b) => Number(b.online) - Number(a.online))
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
				status: invitedRow ? (invitedRow.pending ? 'pending' : 'added') : 'available',
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
	if (props.sharedInstanceActionsLocked) return []

	const credentials = await getCredentials()
	const currentSearchUserId = currentUserId.value ?? credentials?.user_id ?? null
	const users = await search_user(query)

	return users
		.filter((user) => user.id !== currentSearchUserId)
		.filter((user) => canShowSearchInviteUser(user))
		.map((user) => ({
			id: user.id,
			username: user.username,
			avatarUrl: user.avatar_url || undefined,
		}))
}

type SharedRowsMutationContext = {
	queryKey: typeof sharedUsersKey.value
	previousRows?: ShareRow[]
	previousPendingRows: Record<string, ShareRow>
}

const friendRequestMutation = useMutation({
	mutationFn: (user: InvitePlayersUser) => {
		if (props.sharedInstanceActionsLocked) return

		return add_friend(user.id)
	},
	onError: (error) => {
		handleError(toError(error))
	},
})

const inviteShareMutation = useMutation({
	mutationFn: async (user: InvitePlayersUser) => {
		if (props.sharedInstanceActionsLocked) return

		return await invite_shared_instance_users(props.instance.id, [user.id])
	},
	onMutate: async (user): Promise<SharedRowsMutationContext> => {
		const queryKey = sharedUsersKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousRows = queryClient.getQueryData<ShareRow[]>(queryKey)
		const previousPendingRows = pendingRows.value
		setPendingRow(user)

		return { queryKey, previousRows, previousPendingRows }
	},
	onError: (error, user, context) => {
		removePendingRow(user.id)
		restoreSharedRowsQuery(context)
		restorePendingRows(context)
		notifySharedInstanceOperationError(error)
	},
	onSuccess: async (users, user) => {
		try {
			if (users) {
				queryClient.setQueryData<ShareRow[]>(sharedUsersKey.value, await sharedUsersToRows(users))
			} else {
				upsertSharedRow(inviteUserToShareRow(user))
			}
		} catch (error) {
			handleError(toError(error))
			upsertSharedRow(inviteUserToShareRow(user))
			await queryClient.invalidateQueries({ queryKey: sharedUsersKey.value })
		}
	},
})

const removeShareMutation = useMutation({
	mutationFn: async (id: string) => {
		if (props.sharedInstanceActionsLocked) {
			return { user_ids: [], users: [], tokens: 0 } satisfies SharedInstanceUsers
		}

		return await remove_shared_instance_users(props.instance.id, [id])
	},
	onMutate: async (id): Promise<SharedRowsMutationContext> => {
		const queryKey = sharedUsersKey.value
		await queryClient.cancelQueries({ queryKey })
		const previousRows = queryClient.getQueryData<ShareRow[]>(queryKey)
		const previousPendingRows = pendingRows.value
		queryClient.setQueryData<ShareRow[]>(queryKey, (rows = []) =>
			rows.filter((row) => normalizeInviteKey(row.id) !== normalizeInviteKey(id)),
		)
		removePendingRow(id)

		return { queryKey, previousRows, previousPendingRows }
	},
	onError: (error, _id, context) => {
		restoreSharedRowsQuery(context)
		restorePendingRows(context)
		notifySharedInstanceOperationError(error)
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
	if (props.sharedInstanceActionsLocked) return []

	const rows = await sharedUsersToRows(await get_shared_instance_users(props.instance.id))
	removePendingRows(rows.map((row) => row.id))

	return rows
}

async function sharedUsersToRows(users: SharedInstanceUsers): Promise<ShareRow[]> {
	const excludedUserIds = new Set(
		[props.instance.shared_instance?.manager_id, currentUserId.value].filter(
			(id): id is string => !!id,
		),
	)
	const sharedUsers = sharedInstanceUserEntries(users).filter(
		(user) => !excludedUserIds.has(user.id),
	)
	const userIds = sharedUsers.map((user) => user.id)

	if (userIds.length === 0) return []

	const profiles = (await get_user_many(userIds)) as Array<{
		id: string
		username?: string
		avatar_url?: string | null
	}>

	return sharedUsers.map((user) => {
		const profile = profiles.find((profile) => profile.id === user.id)
		const joinedAt = parseSharedInstanceDate(user.joined_at)

		return {
			id: user.id,
			username: profile?.username ?? user.id,
			avatarUrl: profile?.avatar_url ?? undefined,
			lastPlayedAt: parseSharedInstanceDate(user.last_played),
			joinedAt,
			method: sharedInstanceMethod(user),
			pending: !joinedAt,
		} satisfies ShareRow
	})
}

function sharedInstanceUserEntries(users: SharedInstanceUsers): SharedInstanceUser[] {
	if (users.users?.length > 0) return users.users

	return users.user_ids.map((id) => ({
		id,
		joined_at: null,
		join_type: 'invite',
		last_played: null,
	}))
}

function sharedInstanceMethod(user: SharedInstanceUser): ShareMethod {
	return user.join_type === 'link' ? 'link' : 'direct'
}

function parseSharedInstanceDate(value?: string | null) {
	if (!value) return null

	const date = new Date(value)
	return Number.isNaN(date.getTime()) ? null : date
}

function invitePlayer(payload: InvitePlayersInvitePayload) {
	if (props.sharedInstanceActionsLocked) return

	const user = payload.user
	if (payload.source === 'search') {
		void sendFriendRequest(user)
	}

	inviteShareUser(user)
}

function inviteShareUser(user: InvitePlayersUser) {
	if (props.sharedInstanceActionsLocked) return

	const existingRow = findInviteRow(user.id, user.username)
	if (existingRow) {
		return
	}

	inviteShareMutation.mutate(user)
}

async function sendFriendRequest(user: InvitePlayersUser) {
	if (props.sharedInstanceActionsLocked) return
	const credentials = await getCredentials()
	const ownUserId = currentUserId.value ?? credentials?.user_id ?? null
	if (ownUserId && normalizeInviteKey(user.id) === normalizeInviteKey(ownUserId)) return
	if (findUserFriend(user.id, user.username)) return

	friendRequestMutation.mutate(user)
}

function cancelInvite(user: InvitePlayersUser) {
	const existingRow = findInviteRow(user.id, user.username)
	if (existingRow) {
		removeRow(existingRow.id)
	}
}

async function showInvitePlayers(event?: MouseEvent) {
	if (props.sharedInstanceActionsLocked) return
	if (!isSignedIn.value) {
		signInToShare(event)
		return
	}

	if (requiresUnlinkBeforeShare.value) {
		shareUnlinkModal.value?.show()
		return
	}

	if (!(await ensureInviteLink())) return
	invitePlayersModal.value?.show(event)
}

async function unlinkImportedModpackForShare() {
	try {
		await edit(props.instance.id, {
			link: null as unknown as undefined,
		})
		importedModpackUnlinkedForShare.value = true
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', props.instance.id] })
		if (!(await ensureInviteLink())) return
		invitePlayersModal.value?.show()
	} catch (error) {
		notifySharedInstanceOperationError(error)
	}
}

async function ensureInviteLink() {
	if (inviteLinkDetails.value) return true
	if (inviteLinkPending.value) return false

	inviteLinkPending.value = true
	try {
		const invite = await create_shared_instance_invite_link(props.instance.id)
		inviteLinkDetails.value = invite
		return true
	} catch (error) {
		notifySharedInstanceOperationError(error)
		return false
	} finally {
		inviteLinkPending.value = false
	}
}

async function updateInviteLink(settings: InviteLinkSettings) {
	const currentInvite = inviteLinkDetails.value
	if (!currentInvite) return

	inviteLinkPending.value = true
	try {
		const maxAgeSeconds = Math.max(
			1,
			Math.min(604800, Math.floor((settings.expiresAt.getTime() - Date.now()) / 1000)),
		)
		inviteLinkDetails.value = await create_shared_instance_invite_link(props.instance.id, {
			maxAgeSeconds,
			maxUses: settings.maxUses,
			replaceInviteId: currentInvite.inviteId,
		})
	} catch (error) {
		throw toError(error)
	} finally {
		inviteLinkPending.value = false
	}
}

function buildInviteLink(invite: SharedInstanceInviteLink) {
	return `${config.siteUrl}/share/${encodeURIComponent(invite.inviteId)}`
}

function showRemoveRowModal(row: ShareRow) {
	if (props.sharedInstanceActionsLocked) return

	pendingRemovalRow.value = row
	removeUserConfirmModal.value?.show()
}

function hideRemoveUserModal() {
	removeUserConfirmModal.value?.hide()
}

function confirmRemoveRow() {
	const row = pendingRemovalRow.value
	if (!row) return

	removeUserConfirmModal.value?.hide()
	removeRow(row.id)
}

function removeRow(id: string) {
	if (props.sharedInstanceActionsLocked) return

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
	savePendingRows()
}

function removePendingRow(id: string) {
	const { [id]: _removed, ...rest } = pendingRows.value
	pendingRows.value = rest
	savePendingRows()
}

function removePendingRows(ids: string[]) {
	if (ids.length === 0) return

	const normalizedIds = new Set(ids.map(normalizeInviteKey))
	const nextRows = Object.fromEntries(
		Object.entries(pendingRows.value).filter(([id]) => !normalizedIds.has(normalizeInviteKey(id))),
	)
	if (Object.keys(nextRows).length === Object.keys(pendingRows.value).length) return

	pendingRows.value = nextRows
	savePendingRows()
}

function inviteUserToShareRow(user: InvitePlayersUser): ShareRow {
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatarUrl ?? undefined,
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
		pending: true,
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

function restoreSharedRowsQuery(context?: SharedRowsMutationContext) {
	if (!context) return

	if (context.previousRows === undefined) {
		queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
		return
	}

	queryClient.setQueryData(context.queryKey, context.previousRows)
}

function restorePendingRows(context?: SharedRowsMutationContext) {
	if (!context) return

	pendingRows.value = context.previousPendingRows
	savePendingRows()
}

function userProfileLink(username: string) {
	if (!username || username.includes('@')) return undefined
	return () => openUrl(`https://modrinth.com/user/${encodeURIComponent(username)}`)
}

async function requestAuthToShare(flow: ModrinthAuthFlow) {
	await auth.requestSignIn(shareRoutePath.value, flow)
	return isSignedIn.value
}

function signInToShare(event?: MouseEvent) {
	void modrinthAccountRequiredModal.value?.show(event)
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

function findUserFriend(id: string, username: string) {
	const normalizedId = normalizeInviteKey(id)
	const normalizedUsername = normalizeInviteKey(username)

	return userFriends.value.find((friend) => {
		const friendId = getFriendUserId(friend, currentUserId.value)

		return (
			normalizeInviteKey(friendId) === normalizedId ||
			normalizeInviteKey(friend.username) === normalizedUsername
		)
	})
}

function canShowSearchInviteUser(user: InvitePlayersSearchUser) {
	const normalizedId = normalizeInviteKey(user.id)
	const normalizedUsername = normalizeInviteKey(user.username)

	return (
		!inviteFriendKeys.value.has(normalizedId) &&
		!inviteFriendKeys.value.has(normalizedUsername) &&
		!invitedRows.value.has(normalizedId) &&
		!invitedRows.value.has(normalizedUsername)
	)
}

function loadPendingRows(instanceId: string): Record<string, ShareRow> {
	if (typeof localStorage === 'undefined') return {}

	try {
		const storedRows = localStorage.getItem(pendingRowsStorageKey(instanceId))
		if (!storedRows) return {}

		const rows = JSON.parse(storedRows) as ShareRow[]
		return Object.fromEntries(
			rows.map((row) => [
				row.id,
				{
					...row,
					lastPlayedAt: null,
					joinedAt: null,
					pending: true,
				} satisfies ShareRow,
			]),
		)
	} catch {
		return {}
	}
}

function savePendingRows() {
	if (typeof localStorage === 'undefined') return

	const rows = Object.values(pendingRows.value)
	const storageKey = pendingRowsStorageKey(props.instance.id)
	if (rows.length === 0) {
		localStorage.removeItem(storageKey)
		return
	}

	localStorage.setItem(storageKey, JSON.stringify(rows))
}

function pendingRowsStorageKey(instanceId: string) {
	return `modrinth:shared-instance-pending-users:${instanceId}`
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
	(instanceId) => {
		importedModpackUnlinkedForShare.value = false
		inviteLinkDetails.value = undefined
		inviteLinkPending.value = false
		pendingRows.value = loadPendingRows(instanceId)
	},
)

watch(
	[() => auth.isReady.value, isSignedIn, () => props.sharedInstanceActionsLocked],
	([isReady, signedIn, actionsLocked]) => {
		if (isReady && !signedIn && !actionsLocked) signInToShare()
	},
	{ immediate: true, flush: 'post' },
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
