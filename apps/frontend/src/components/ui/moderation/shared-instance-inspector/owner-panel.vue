<script setup lang="ts">
import {
	ExternalIcon,
	KeyIcon,
	LoaderCircleIcon,
	LockIcon,
	LockOpenIcon,
	LogOutIcon,
	ShieldAlertIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	CopyCode,
	ForcePasswordResetModal,
	getRemovedScopes,
	injectModrinthClient,
	injectNotificationManager,
	LockUserModal,
	Reset2faModal,
	RestrictUserModal,
	RevokeSessionsModal,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useQueries, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ProjectStatusStats from '~/components/ProjectStatusStats.vue'
import { useAuthState } from '~/composables/auth'
import {
	creatorProjectStatsQueryOptions,
	useCreatorProjectStats,
} from '~/composables/creator-project-stats'
import {
	type SharedInstanceModerationUser,
	sharedInstanceQueryOptions,
} from '~/composables/queries/shared-instance'
import { getRequestErrorMessage } from '~/helpers/shared-instance-moderation'

const ACCOUNT_STAT_STATUSES = new Set([
	'shared_instances_banned',
	'account_locked',
	'account_restricted',
])

const props = defineProps<{
	owner: SharedInstanceModerationUser
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const auth = useAuthState()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const lockModal = ref<InstanceType<typeof LockUserModal> | null>(null)
const restrictModal = ref<InstanceType<typeof RestrictUserModal> | null>(null)
const revokeSessionsModal = ref<InstanceType<typeof RevokeSessionsModal> | null>(null)
const forcePasswordResetModal = ref<InstanceType<typeof ForcePasswordResetModal> | null>(null)
const reset2faModal = ref<InstanceType<typeof Reset2faModal> | null>(null)
const unlocking = ref(false)

const isAdmin = computed(() => auth.value.user?.role === 'admin')
const isStaffViewing = computed(() => !!isStaff(auth.value.user))

const {
	data: account,
	error: accountError,
	isPending: accountPending,
	refetch: refetchAccount,
} = useQuery(computed(() => creatorProjectStatsQueryOptions.memberAccount(props.owner.id, client)))

const { data: blacklist } = useQuery(
	computed(() => sharedInstanceQueryOptions.blacklist(props.owner.id, client)),
)

const { memberStats, membersLoading } = useCreatorProjectStats(
	computed(() => [props.owner.id]),
	computed(() => ''),
)
const projectStats = computed(() =>
	(memberStats.value[props.owner.id] ?? []).filter(
		(stat) => !ACCOUNT_STAT_STATUSES.has(stat.status),
	),
)
const hasProjectStats = computed(() => projectStats.value.some((stat) => stat.count > 0))

const moderationNote = computed(() => account.value?.moderation_notes)
const lock = computed(() => account.value?.lock)
const restriction = computed(() => account.value?.restriction)
const removedScopes = computed(() =>
	getRemovedScopes(restriction.value?.removed_perms).map((scope) => formatMessage(scope.label)),
)

const moderatorIds = computed(() => [
	...new Set(
		[
			lock.value?.locked_by,
			restriction.value?.restricted_by,
			blacklist.value?.blacklisted_by,
			moderationNote.value?.last_author,
		].filter((id): id is string => !!id),
	),
])
const moderators = useQueries({
	queries: computed(() =>
		moderatorIds.value.map((id) => sharedInstanceQueryOptions.moderator(id, client)),
	),
})

function moderatorName(id?: string | null) {
	if (!id) return null
	return moderators.value.find((moderator) => moderator.data?.id === id)?.data?.username ?? id
}

function attribution(id: string | null | undefined, date: string | null | undefined) {
	const moderator = moderatorName(id)
	return [moderator ? `By ${moderator}` : null, date ? formatDateTime(date) : null]
		.filter(Boolean)
		.join(' · ')
}

const accountDetails = computed(() => {
	const user = account.value
	if (!user) return []
	const yesNo = (value: unknown) => (value ? 'Yes' : 'No')
	return [
		{ label: 'Role', value: user.role.charAt(0).toUpperCase() + user.role.slice(1) },
		{ label: 'Joined', value: formatDateTime(user.created) },
		...(isAdmin.value ? [{ label: 'Email', value: user.email ?? '—' }] : []),
		{ label: 'Email verified', value: yesNo(user.email_verified) },
		{ label: 'Has password', value: yesNo(user.has_password) },
		{ label: 'Has TOTP', value: yesNo(user.has_totp) },
		...(isAdmin.value
			? [{ label: 'Auth providers', value: (user.auth_providers ?? []).join(', ') || '—' }]
			: []),
	]
})

const accountActions = computed(() => {
	const user = account.value
	if (!user) return []
	const isDeveloper = user.role === 'developer'
	return [
		{
			id: 'toggle-lock',
			label: user.lock ? 'Unlock account' : 'Lock account',
			icon: user.lock ? LockOpenIcon : LockIcon,
			shown: isAdmin.value && isDeveloper,
			action: toggleLock,
		},
		{
			id: 'restrict',
			label: user.restriction ? 'Edit restrictions' : 'Restrict user',
			icon: ShieldAlertIcon,
			shown: isStaffViewing.value && isDeveloper,
			action: () => restrictModal.value?.show(),
		},
		{
			id: 'revoke-sessions',
			label: 'Revoke all sessions',
			icon: LogOutIcon,
			shown: isAdmin.value && auth.value.user?.id !== user.id,
			action: () => revokeSessionsModal.value?.show(),
		},
		{
			id: 'force-password-reset',
			label: 'Force password reset',
			icon: KeyIcon,
			shown: isAdmin.value && isDeveloper,
			action: () => forcePasswordResetModal.value?.show(),
		},
		{
			id: 'reset-2fa',
			label: 'Reset two-factor authentication',
			icon: ShieldAlertIcon,
			shown: isAdmin.value && isDeveloper && !!user.has_totp,
			action: () => reset2faModal.value?.show(),
		},
	].filter((action) => action.shown)
})

async function toggleLock() {
	if (!account.value?.lock) {
		lockModal.value?.show()
		return
	}
	if (unlocking.value) return

	unlocking.value = true
	try {
		await client.labrinth.moderation_internal.unlockUser(props.owner.id)
		await queryClient.invalidateQueries({ queryKey: ['user', props.owner.id] })
		addNotification({
			type: 'success',
			title: 'Account unlocked',
			text: `${props.owner.username}'s account has been unlocked.`,
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: 'Failed to unlock account',
			text: getRequestErrorMessage(error, 'An error occurred while unlocking this account.'),
		})
	} finally {
		unlocking.value = false
	}
}
</script>

<template>
	<div class="overflow-y-auto p-6">
		<div class="flex flex-col gap-6">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex min-w-0 flex-wrap items-center gap-3">
					<Avatar
						:src="owner.avatar_url"
						:alt="owner.username"
						:tint-by="owner.id"
						size="3rem"
						circle
						no-shadow
					/>
					<span class="truncate text-lg font-semibold text-contrast">{{ owner.username }}</span>
					<CopyCode v-tooltip="'Copy user ID'" :text="owner.id" />
				</div>
				<ButtonLink :to="`/user/${owner.id}`" target="_blank">
					<UserIcon />
					User profile
					<ExternalIcon class="size-4" />
				</ButtonLink>
			</div>

			<div
				v-if="accountPending"
				class="flex min-h-32 items-center justify-center gap-2 text-secondary"
			>
				<LoaderCircleIcon class="size-6 animate-spin" />
				Loading account…
			</div>
			<Admonition
				v-else-if="accountError"
				type="critical"
				header="Failed to load account"
				:body="getRequestErrorMessage(accountError, 'The account could not be loaded.')"
				show-actions-underneath
			>
				<template #actions>
					<Button @click="refetchAccount()">Try again</Button>
				</template>
			</Admonition>
			<template v-else-if="account">
				<section class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">Moderation status</h3>
					<div class="flex flex-col rounded-2xl border border-solid border-surface-4 bg-surface-2">
						<div class="flex flex-col gap-1 p-4">
							<div class="flex flex-wrap items-center justify-between gap-2">
								<span class="font-semibold text-contrast">Account lock</span>
								<span class="text-sm font-semibold" :class="lock ? 'text-red' : 'text-secondary'">
									{{ lock ? 'Locked' : 'Not locked' }}
								</span>
							</div>
							<template v-if="lock">
								<span class="text-sm text-secondary">
									{{ attribution(lock.locked_by, lock.created) }}
								</span>
								<span class="break-words">Reason: {{ lock.reason }}</span>
							</template>
						</div>

						<div class="flex flex-col gap-1 border-0 border-t border-solid border-surface-4 p-4">
							<div class="flex flex-wrap items-center justify-between gap-2">
								<span class="font-semibold text-contrast">Restrictions</span>
								<span
									class="text-sm font-semibold"
									:class="restriction ? 'text-orange' : 'text-secondary'"
								>
									{{ restriction ? 'Restricted' : 'None' }}
								</span>
							</div>
							<template v-if="restriction">
								<span class="text-sm text-secondary">
									{{ attribution(restriction.restricted_by, restriction.updated) }}
								</span>
								<ul v-if="removedScopes.length" class="m-0 list-disc pl-5">
									<li v-for="scope in removedScopes" :key="scope">{{ scope }}</li>
								</ul>
								<span v-if="restriction.reason" class="break-words">
									Reason: {{ restriction.reason }}
								</span>
								<span v-if="restriction.private_reason" class="break-words">
									Private note: {{ restriction.private_reason }}
								</span>
							</template>
						</div>

						<div class="flex flex-col gap-1 border-0 border-t border-solid border-surface-4 p-4">
							<div class="flex flex-wrap items-center justify-between gap-2">
								<span class="font-semibold text-contrast">Shared instances</span>
								<span
									class="text-sm font-semibold"
									:class="blacklist?.blacklisted ? 'text-red' : 'text-secondary'"
								>
									{{ blacklist?.blacklisted ? 'Banned' : blacklist ? 'Allowed' : 'Unknown' }}
								</span>
							</div>
							<span v-if="blacklist?.blacklisted" class="text-sm text-secondary">
								{{ attribution(blacklist.blacklisted_by, blacklist.blacklisted_at) }}
							</span>
						</div>
					</div>
				</section>

				<section class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">Projects</h3>
					<ProjectStatusStats
						v-if="hasProjectStats"
						:stats="projectStats"
						labelled
						class="!mt-0 !gap-x-4 !text-sm"
					/>
					<span v-else-if="membersLoading" class="text-sm text-secondary">Loading projects…</span>
					<span v-else class="text-sm text-secondary">This user has no projects.</span>
				</section>

				<section v-if="moderationNote?.notes" class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">Moderation note</h3>
					<div class="flex flex-col gap-1 rounded-2xl bg-surface-2 p-4">
						<span class="whitespace-pre-wrap break-words">{{ moderationNote.notes }}</span>
						<span class="text-sm text-secondary">
							{{ attribution(moderationNote.last_author, moderationNote.last_modified) }}
						</span>
					</div>
				</section>

				<section class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">Account details</h3>
					<dl class="m-0 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
						<div v-for="detail in accountDetails" :key="detail.label" class="min-w-0">
							<dt class="text-sm text-secondary">{{ detail.label }}</dt>
							<dd class="m-0 break-words font-medium text-primary">{{ detail.value }}</dd>
						</div>
					</dl>
				</section>

				<section v-if="accountActions.length" class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">Account actions</h3>
					<div class="flex flex-wrap gap-2">
						<Button
							v-for="action in accountActions"
							:key="action.id"
							type="outlined"
							color="red"
							:loading="action.id === 'toggle-lock' && unlocking"
							@click="action.action()"
						>
							<component :is="action.icon" />
							{{ action.label }}
						</Button>
					</div>
				</section>

				<LockUserModal v-if="isAdmin" ref="lockModal" :user="account" :user-id="owner.id" />
				<RestrictUserModal
					v-if="isStaffViewing"
					ref="restrictModal"
					:user="account"
					:user-id="owner.id"
				/>
				<RevokeSessionsModal v-if="isAdmin" ref="revokeSessionsModal" :user="account" />
				<ForcePasswordResetModal
					v-if="isAdmin"
					ref="forcePasswordResetModal"
					:user="account"
					:user-id="owner.id"
				/>
				<Reset2faModal v-if="isAdmin" ref="reset2faModal" :user="account" :user-id="owner.id" />
			</template>
		</div>
	</div>
</template>
