<template>
	<section class="flex flex-col gap-4">
		<div class="flex flex-col gap-1">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.blockedUsersTitle) }}
			</h2>
			<p class="m-0 text-secondary">
				{{ formatMessage(messages.blockedUsersDescription) }}
			</p>
			<ul class="m-0 flex list-disc flex-col gap-1 pl-5 text-secondary">
				<li>{{ formatMessage(messages.friendRequestsRestriction) }}</li>
				<li>{{ formatMessage(messages.sharedInstancesRestriction) }}</li>
				<li>{{ formatMessage(messages.hostingRestriction) }}</li>
			</ul>
		</div>

		<div class="relative overflow-hidden rounded-2xl border border-solid border-surface-4">
			<Transition
				enter-active-class="transition-all duration-200 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-3"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-3"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="showTopFade"
					class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-3 bg-gradient-to-b from-bg-raised to-transparent"
				/>
			</Transition>

			<div
				ref="blockedUsersTable"
				class="max-h-[20.5rem] overflow-y-auto"
				@scroll="checkScrollState"
			>
				<Table
					class="!rounded-none !border-0"
					:columns="columns"
					:data="blockedUsers"
					row-key="id"
				>
					<template #empty-state>
						<div class="flex h-40 items-center justify-center px-4 text-center text-secondary">
							<div v-if="isLoading" class="flex items-center gap-2">
								<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
								{{ formatMessage(messages.loadingBlockedUsers) }}
							</div>
							<div v-else-if="loadError" class="flex flex-col items-center gap-3">
								<span>{{ formatMessage(messages.loadError) }}</span>
								<ButtonStyled type="outlined">
									<button type="button" @click="retry">
										{{ formatMessage(commonMessages.retryButton) }}
									</button>
								</ButtonStyled>
							</div>
							<span v-else-if="!auth.user.value">
								{{ formatMessage(messages.signInRequired) }}
							</span>
							<span v-else>{{ formatMessage(messages.noBlockedUsers) }}</span>
						</div>
					</template>

					<template #cell-user="{ row }">
						<div class="flex min-w-0 items-center gap-3">
							<Avatar
								:src="row.avatar_url"
								:alt="formatMessage(messages.userAvatarAlt, { username: row.username })"
								:tint-by="row.username"
								size="32px"
								circle
								no-shadow
							/>
							<div class="flex min-w-0 flex-col">
								<span class="truncate font-semibold text-contrast">
									{{ row.name ?? row.username }}
								</span>
								<span v-if="row.name" class="truncate text-sm text-secondary">
									{{ row.username }}
								</span>
							</div>
						</div>
					</template>

					<template #cell-actions="{ row }">
						<div class="flex justify-end">
							<ButtonStyled type="outlined">
								<button
									type="button"
									:disabled="unblockingUserId !== null"
									:aria-label="
										formatMessage(messages.unblockUserAriaLabel, {
											username: row.username,
										})
									"
									@click="unblock(row)"
								>
									<SpinnerIcon
										v-if="unblockingUserId === row.id"
										class="animate-spin"
										aria-hidden="true"
									/>
									{{ formatMessage(messages.unblockButton) }}
								</button>
							</ButtonStyled>
						</div>
					</template>
				</Table>
			</div>

			<Transition
				enter-active-class="transition-all duration-200 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-3"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-3"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="showBottomFade"
					class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-3 bg-gradient-to-t from-bg-raised to-transparent"
				/>
			</Transition>
		</div>
	</section>
</template>

<script setup lang="ts">
// TODO this will be moved in with the rest of the xplat settings.
import type { Labrinth } from '@modrinth/api-client'
import { SpinnerIcon } from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Table, { type TableColumn } from '#ui/components/base/Table.vue'
import { defineMessages, useScrollIndicator, useVIntl } from '#ui/composables'
import { injectAuth, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

import { blockedUsersQueryKey } from '../shared/user-profile/providers'

type BlockedUserTableColumn = 'user' | 'actions'
type BlockedUser = Labrinth.Users.v2.User & Record<BlockedUserTableColumn, unknown>

const props = defineProps<{
	getBlockedUsers: () => Promise<Labrinth.BlockedUsers.v3.BlockedUserId[]>
	getUsers: (userIds: string[]) => Promise<Labrinth.Users.v2.User[]>
	unblockUser: (userId: string) => Promise<void>
}>()

const auth = injectAuth()
const notificationManager = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const blockedUsersTable = ref<HTMLElement | null>(null)
const unblockingUserId = ref<string | null>(null)
const { showTopFade, showBottomFade, checkScrollState } =
	useScrollIndicator(blockedUsersTable)

const columns = computed<TableColumn<BlockedUserTableColumn>[]>(() => [
	{
		key: 'user',
		label: formatMessage(messages.userColumn),
	},
	{
		key: 'actions',
		label: formatMessage(messages.actionsColumn),
		align: 'right',
		width: '8rem',
	},
])

const blockedUserIdsQuery = useQuery({
	queryKey: computed(() => blockedUsersQueryKey(auth.user.value?.id)),
	queryFn: props.getBlockedUsers,
	enabled: computed(() => Boolean(auth.user.value)),
	staleTime: 30_000,
})
const blockedUserIds = computed(() => blockedUserIdsQuery.data.value ?? [])
const blockedUserProfilesQueryKey = computed(
	() => ['blocked-user-profiles', auth.user.value?.id ?? null, blockedUserIds.value] as const,
)
const blockedUserProfilesQuery = useQuery({
	queryKey: blockedUserProfilesQueryKey,
	queryFn: () => props.getUsers(blockedUserIds.value),
	enabled: computed(() => Boolean(auth.user.value && blockedUserIds.value.length)),
	staleTime: 30_000,
})
const blockedUsers = computed<BlockedUser[]>(() => {
	const profilesById = new Map(
		(blockedUserProfilesQuery.data.value ?? []).map((user) => [user.id, user]),
	)
	return blockedUserIds.value
		.map((userId) => profilesById.get(userId))
		.filter((user): user is Labrinth.Users.v2.User => Boolean(user))
		.map((user) => ({
			...user,
			user: user.username,
			actions: null,
		}))
})
const isLoading = computed(
	() =>
		Boolean(auth.user.value) &&
		(blockedUserIdsQuery.isPending.value ||
			(blockedUserIds.value.length > 0 && blockedUserProfilesQuery.isPending.value)),
)
const loadError = computed(
	() => blockedUserIdsQuery.error.value ?? blockedUserProfilesQuery.error.value,
)

async function retry(): Promise<void> {
	await blockedUserIdsQuery.refetch()
	if (blockedUserIds.value.length > 0) {
		await blockedUserProfilesQuery.refetch()
	}
}

async function unblock(user: BlockedUser): Promise<void> {
	if (unblockingUserId.value) return

	unblockingUserId.value = user.id
	try {
		await props.unblockUser(user.id)

		const remainingIds = blockedUserIds.value.filter((userId) => userId !== user.id)
		const remainingUsers = blockedUsers.value.filter((blockedUser) => blockedUser.id !== user.id)
		queryClient.setQueryData(
			['blocked-user-profiles', auth.user.value?.id ?? null, remainingIds],
			remainingUsers,
		)
		queryClient.setQueryData<Labrinth.BlockedUsers.v3.BlockedUserId[]>(
			blockedUsersQueryKey(auth.user.value?.id),
			remainingIds,
		)

		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.userUnblocked),
			text: formatMessage(messages.userUnblockedDescription, {
				username: user.username,
			}),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.unblockError),
			text: formatMessage(messages.unblockErrorDescription),
		})
	} finally {
		unblockingUserId.value = null
	}
}

const messages = defineMessages({
	blockedUsersTitle: {
		id: 'settings.safety.blocked-users.title',
		defaultMessage: 'Blocked users',
	},
	blockedUsersDescription: {
		id: 'settings.safety.blocked-users.description',
		defaultMessage: 'These are the users you have blocked on Modrinth. They cannot:',
	},
	friendRequestsRestriction: {
		id: 'settings.safety.blocked-users.restriction.friend-requests',
		defaultMessage: 'Send you friend requests',
	},
	sharedInstancesRestriction: {
		id: 'settings.safety.blocked-users.restriction.shared-instances',
		defaultMessage: 'Invite you to shared instances',
	},
	hostingRestriction: {
		id: 'settings.safety.blocked-users.restriction.hosting',
		defaultMessage: 'Invite you to manage a Modrinth Hosting server.',
	},
	userColumn: {
		id: 'settings.safety.blocked-users.column.user',
		defaultMessage: 'User',
	},
	actionsColumn: {
		id: 'settings.safety.blocked-users.column.actions',
		defaultMessage: 'Actions',
	},
	unblockButton: {
		id: 'settings.safety.blocked-users.unblock',
		defaultMessage: 'Unblock',
	},
	unblockUserAriaLabel: {
		id: 'settings.safety.blocked-users.unblock-user',
		defaultMessage: 'Unblock {username}',
	},
	loadingBlockedUsers: {
		id: 'settings.safety.blocked-users.loading',
		defaultMessage: 'Loading blocked users…',
	},
	noBlockedUsers: {
		id: 'settings.safety.blocked-users.empty',
		defaultMessage: "You haven't blocked anyone.",
	},
	signInRequired: {
		id: 'settings.safety.blocked-users.sign-in-required',
		defaultMessage: 'Sign in to manage your blocked users.',
	},
	loadError: {
		id: 'settings.safety.blocked-users.load-error',
		defaultMessage: 'Blocked users could not be loaded.',
	},
	userAvatarAlt: {
		id: 'settings.safety.blocked-users.user-avatar',
		defaultMessage: "{username}'s avatar",
	},
	userUnblocked: {
		id: 'settings.safety.blocked-users.unblocked',
		defaultMessage: 'User unblocked',
	},
	userUnblockedDescription: {
		id: 'settings.safety.blocked-users.unblocked-description',
		defaultMessage: '{username} has been unblocked.',
	},
	unblockError: {
		id: 'settings.safety.blocked-users.unblock-error',
		defaultMessage: 'Failed to unblock user',
	},
	unblockErrorDescription: {
		id: 'settings.safety.blocked-users.unblock-error-description',
		defaultMessage: 'An error occurred while unblocking this user. Please try again.',
	},
})
</script>
