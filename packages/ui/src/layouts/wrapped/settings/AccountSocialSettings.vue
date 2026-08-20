<template>
	<EmptyState
		v-if="!auth.user.value"
		type="empty"
		class="[&>div:last-child]:!mt-6"
		:heading="formatMessage(messages.signInRequiredTitle)"
		:description="formatMessage(messages.signInRequiredDescription)"
	>
		<template #illustration>
			<div class="relative mb-4 h-[200px]">
				<img :src="ThinkingRinthbot" alt="" class="h-full w-auto object-contain" />
				<div
					class="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-gradient-to-t from-bg-raised to-transparent"
				/>
			</div>
		</template>
		<template #actions>
			<Button type="colored" color="brand" size="xl" native-type="button" @click="requestSignIn">
				<LogInIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.signInButton) }}
			</Button>
		</template>
	</EmptyState>

	<div v-else class="flex flex-col gap-8">
		<section class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.friendRequestsTitle) }}
				</h2>
				<Chips
					:model-value="friendPrivacy"
					:items="friendPrivacyOptions"
					:format-label="formatInteractionSource"
					:disabled-items="preferenceControlsDisabled ? friendPrivacyOptions : undefined"
					:disabled-tooltip="preferenceControlsTooltip"
					:capitalize="false"
					:aria-label="formatMessage(messages.friendRequestsTitle)"
					@update:model-value="setFriendPrivacy"
				/>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.friendRequestsDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.sharedInstanceInvitesTitle) }}
				</h2>
				<Chips
					:model-value="sharedInstancesPrivacy"
					:items="invitePrivacyOptions"
					:format-label="formatInteractionSource"
					:disabled-items="preferenceControlsDisabled ? invitePrivacyOptions : undefined"
					:disabled-tooltip="preferenceControlsTooltip"
					:capitalize="false"
					:aria-label="formatMessage(messages.sharedInstanceInvitesTitle)"
					@update:model-value="setSharedInstancesPrivacy"
				/>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.sharedInstanceInvitesDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.hostingAccessTitle) }}
				</h2>
				<Chips
					:model-value="hostingAccessPrivacy"
					:items="invitePrivacyOptions"
					:format-label="formatInteractionSource"
					:disabled-items="preferenceControlsDisabled ? invitePrivacyOptions : undefined"
					:disabled-tooltip="preferenceControlsTooltip"
					:capitalize="false"
					:aria-label="formatMessage(messages.hostingAccessTitle)"
					@update:model-value="setHostingAccessPrivacy"
				/>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.hostingAccessDescription) }}
				</p>
			</div>
		</section>

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
									<Button type="outlined" native-type="button" @click="retry">
										{{ formatMessage(commonMessages.retryButton) }}
									</Button>
								</div>
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
								<Button
									type="outlined"
									native-type="button"
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
								</Button>
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
	</div>
</template>

<script setup lang="ts">
// TODO this will be moved in with the rest of the xplat settings.
import type { Labrinth } from '@modrinth/api-client'
import { LogInIcon, SpinnerIcon, ThinkingRinthbot } from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button } from '#ui/components/base/buttons'
import Chips from '#ui/components/base/Chips.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import Table, { type TableColumn } from '#ui/components/base/Table.vue'
import { defineMessages, useScrollIndicator, useVIntl } from '#ui/composables'
import { injectAuth, injectNotificationManager, injectUserPreferences } from '#ui/providers'
import { commonMessages } from '#ui/utils'

import { blockedUsersQueryKey } from '../../shared/user-profile/providers'

type BlockedUserTableColumn = 'user' | 'actions'
type BlockedUser = Labrinth.Users.v2.User & Record<BlockedUserTableColumn, unknown>
type FriendPrivacy = Labrinth.Users.v3.FriendPrivacy
type InvitePrivacy = Labrinth.Users.v3.InvitePrivacy
type SocialSettingsState = {
	friendPrivacy: FriendPrivacy
	sharedInstancesPrivacy: InvitePrivacy
	hostingAccessPrivacy: InvitePrivacy
}

const props = defineProps<{
	getBlockedUsers: () => Promise<Labrinth.BlockedUsers.v3.BlockedUserId[]>
	getUsers: (userIds: string[]) => Promise<Labrinth.Users.v2.User[]>
	unblockUser: (userId: string) => Promise<void>
}>()

const auth = injectAuth()
const notificationManager = injectNotificationManager()
const {
	preferences,
	isLoading: preferencesLoading,
	isUpdating: preferencesUpdating,
	updatePreferences,
} = injectUserPreferences()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const blockedUsersTable = ref<HTMLElement | null>(null)
const unblockingUserId = ref<string | null>(null)
const saving = ref(false)
const preferencesInitialized = ref(false)
const friendPrivacy = ref<FriendPrivacy>('everyone')
const sharedInstancesPrivacy = ref<InvitePrivacy>('everyone')
const hostingAccessPrivacy = ref<InvitePrivacy>('everyone')
const friendPrivacyOptions: FriendPrivacy[] = ['everyone', 'mutual', 'none']
const invitePrivacyOptions: InvitePrivacy[] = ['everyone', 'friends', 'none']
const preferenceControlsDisabled = computed(
	() => preferencesLoading.value || preferencesUpdating.value || saving.value || !preferences.value,
)
const preferenceControlsTooltip = computed(() => {
	if (preferencesLoading.value) return formatMessage(messages.loadingPreferences)
	if (saving.value || preferencesUpdating.value) return formatMessage(messages.savingPreferences)
	return formatMessage(messages.preferencesUnavailable)
})
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(blockedUsersTable)

const originalState = computed<SocialSettingsState>(() => ({
	friendPrivacy: preferences.value?.social.friend_privacy ?? 'everyone',
	sharedInstancesPrivacy: preferences.value?.social.shared_instances_privacy ?? 'everyone',
	hostingAccessPrivacy: preferences.value?.social.hosting_access_privacy ?? 'everyone',
}))
const modifiedState = computed<Partial<SocialSettingsState>>(() => ({
	...(friendPrivacy.value !== originalState.value.friendPrivacy
		? { friendPrivacy: friendPrivacy.value }
		: {}),
	...(sharedInstancesPrivacy.value !== originalState.value.sharedInstancesPrivacy
		? { sharedInstancesPrivacy: sharedInstancesPrivacy.value }
		: {}),
	...(hostingAccessPrivacy.value !== originalState.value.hostingAccessPrivacy
		? { hostingAccessPrivacy: hostingAccessPrivacy.value }
		: {}),
}))
const hasChanges = computed(() => Object.keys(modifiedState.value).length > 0)

function formatInteractionSource(source: FriendPrivacy | InvitePrivacy): string {
	switch (source) {
		case 'everyone':
			return formatMessage(messages.everyone)
		case 'mutual':
			return formatMessage(messages.friendsOfFriends)
		case 'friends':
			return formatMessage(messages.friends)
		case 'none':
			return formatMessage(messages.noOne)
	}
}

watch(
	preferences,
	(value) => {
		if (!value) return
		if (preferencesInitialized.value && hasChanges.value && !saving.value) return

		friendPrivacy.value = value.social.friend_privacy
		sharedInstancesPrivacy.value = value.social.shared_instances_privacy
		hostingAccessPrivacy.value = value.social.hosting_access_privacy
		preferencesInitialized.value = true
	},
	{ immediate: true, flush: 'sync' },
)

function setFriendPrivacy(value: FriendPrivacy | null): void {
	if (!value) return
	friendPrivacy.value = value
}

function setSharedInstancesPrivacy(value: InvitePrivacy | null): void {
	if (!value) return
	sharedInstancesPrivacy.value = value
}

function setHostingAccessPrivacy(value: InvitePrivacy | null): void {
	if (!value) return
	hostingAccessPrivacy.value = value
}

function reset(): void {
	friendPrivacy.value = originalState.value.friendPrivacy
	sharedInstancesPrivacy.value = originalState.value.sharedInstancesPrivacy
	hostingAccessPrivacy.value = originalState.value.hostingAccessPrivacy
}

async function save(): Promise<void> {
	if (!hasChanges.value || saving.value) return

	saving.value = true
	try {
		await updatePreferences({
			social: {
				friend_privacy: friendPrivacy.value,
				shared_instances_privacy: sharedInstancesPrivacy.value,
				hosting_access_privacy: hostingAccessPrivacy.value,
			},
		})
	} catch {
		return
	} finally {
		saving.value = false
	}
}

defineExpose({ hasChanges, originalState, modifiedState, saving, reset, save })

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

async function requestSignIn(): Promise<void> {
	await auth.requestSignIn('')
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
	friendRequestsTitle: {
		id: 'settings.social.friend-requests.title',
		defaultMessage: 'Friend requests',
	},
	friendRequestsDescription: {
		id: 'settings.social.friend-requests.description',
		defaultMessage: 'Control who can send you friend requests on Modrinth.',
	},
	sharedInstanceInvitesTitle: {
		id: 'settings.social.shared-instance-invites.title',
		defaultMessage: 'Shared instance invites',
	},
	sharedInstanceInvitesDescription: {
		id: 'settings.social.shared-instance-invites.description',
		defaultMessage: 'Control who can send you invites to shared instances.',
	},
	hostingAccessTitle: {
		id: 'settings.social.hosting-access.title',
		defaultMessage: 'Hosting access invites',
	},
	hostingAccessDescription: {
		id: 'settings.social.hosting-access.description',
		defaultMessage: 'Control who can invite you to manage a Modrinth Hosting server.',
	},
	everyone: {
		id: 'settings.social.interaction-source.everyone',
		defaultMessage: 'Everyone',
	},
	friendsOfFriends: {
		id: 'settings.social.interaction-source.friends-of-friends',
		defaultMessage: 'Friends of friends',
	},
	friends: {
		id: 'settings.social.interaction-source.friends',
		defaultMessage: 'Friends',
	},
	noOne: {
		id: 'settings.social.interaction-source.no-one',
		defaultMessage: 'No one',
	},
	savingPreferences: {
		id: 'settings.social.interaction-source.saving',
		defaultMessage: 'Saving your preferences…',
	},
	loadingPreferences: {
		id: 'settings.social.interaction-source.loading',
		defaultMessage: 'Loading your preferences…',
	},
	preferencesUnavailable: {
		id: 'settings.social.interaction-source.unavailable',
		defaultMessage: 'Your preferences are currently unavailable.',
	},
	blockedUsersTitle: {
		id: 'settings.social.blocked-users.title',
		defaultMessage: 'Blocked users',
	},
	blockedUsersDescription: {
		id: 'settings.social.blocked-users.description',
		defaultMessage: 'These are the users you have blocked on Modrinth. They cannot:',
	},
	friendRequestsRestriction: {
		id: 'settings.social.blocked-users.restriction.friend-requests',
		defaultMessage: 'Send you friend requests',
	},
	sharedInstancesRestriction: {
		id: 'settings.social.blocked-users.restriction.shared-instances',
		defaultMessage: 'Invite you to shared instances',
	},
	hostingRestriction: {
		id: 'settings.social.blocked-users.restriction.hosting',
		defaultMessage: 'Invite you to manage a Modrinth Hosting server.',
	},
	userColumn: {
		id: 'settings.social.blocked-users.column.user',
		defaultMessage: 'User',
	},
	actionsColumn: {
		id: 'settings.social.blocked-users.column.actions',
		defaultMessage: 'Actions',
	},
	unblockButton: {
		id: 'settings.social.blocked-users.unblock',
		defaultMessage: 'Unblock',
	},
	unblockUserAriaLabel: {
		id: 'settings.social.blocked-users.unblock-user',
		defaultMessage: 'Unblock {username}',
	},
	loadingBlockedUsers: {
		id: 'settings.social.blocked-users.loading',
		defaultMessage: 'Loading blocked users…',
	},
	noBlockedUsers: {
		id: 'settings.social.blocked-users.empty',
		defaultMessage: "You haven't blocked anyone.",
	},
	signInRequiredTitle: {
		id: 'settings.social.sign-in-required.title',
		defaultMessage: 'Modrinth account required',
	},
	signInRequiredDescription: {
		id: 'settings.social.sign-in-required.description',
		defaultMessage:
			'You can control who can interact with you, and manage blocked users with a Modrinth Account',
	},
	loadError: {
		id: 'settings.social.blocked-users.load-error',
		defaultMessage: 'Blocked users could not be loaded.',
	},
	userAvatarAlt: {
		id: 'settings.social.blocked-users.user-avatar',
		defaultMessage: "{username}'s avatar",
	},
	unblockError: {
		id: 'settings.social.blocked-users.unblock-error',
		defaultMessage: 'Failed to unblock user',
	},
	unblockErrorDescription: {
		id: 'settings.social.blocked-users.unblock-error-description',
		defaultMessage: 'An error occurred while unblocking this user. Please try again.',
	},
})
</script>
