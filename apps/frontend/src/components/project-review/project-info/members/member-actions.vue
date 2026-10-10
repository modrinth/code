<template>
	<TeleportOverflowMenu
		v-bind="$attrs"
		:label="formatMessage(commonMessages.moreOptionsButton)"
		:options="options"
		:disabled="mutation.isPending.value"
		type="quiet"
		size="sm"
		@open="activated = true"
	>
		<MoreHorizontalIcon aria-hidden="true" />
	</TeleportOverflowMenu>
	<template v-if="activated">
		<EditUserModal v-if="isAdmin" ref="editModal" :user="user" :user-id="user.id" />
		<LockUserModal v-if="isAdmin" ref="lockModal" :user="user" :user-id="user.id" />
		<ForcePasswordResetModal v-if="isAdmin" ref="passwordModal" :user="user" :user-id="user.id" />
		<Reset2faModal v-if="isAdmin" ref="totpModal" :user="user" :user-id="user.id" />
		<NewModal
			ref="confirmModal"
			:header="
				formatMessage(
					confirmation === 'block' ? messages.blockTitle : menuMessages.revokeSessionsButton,
				)
			"
			:closable="!mutation.isPending.value"
			fade="danger"
			max-width="500px"
		>
			<p class="m-0">
				{{
					formatMessage(confirmation === 'block' ? messages.blockBody : messages.revokeBody, {
						username: user.username,
					})
				}}
			</p>
			<template #actions>
				<Button type="outlined" :disabled="mutation.isPending.value" @click="confirmModal?.hide()">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="red"
					:loading="mutation.isPending.value"
					:disabled="mutation.isPending.value"
					@click="mutation.mutate(confirmation)"
				>
					{{
						formatMessage(
							confirmation === 'block'
								? menuMessages.blockButton
								: menuMessages.revokeSessionsButton,
						)
					}}
				</Button>
			</template>
		</NewModal>
		<NewModal
			v-if="isStaffViewing"
			ref="detailsModal"
			:header="formatMessage(messages.detailsTitle)"
		>
			<dl class="m-0 flex flex-col gap-3">
				<div v-for="detail in userDetails" :key="detail.label.id">
					<dt class="font-semibold text-contrast">
						{{ formatMessage(detail.label) }}
					</dt>
					<dd class="m-0 break-words">{{ detail.value }}</dd>
				</div>
			</dl>
		</NewModal>
	</template>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AffiliateIcon,
	BanIcon,
	BoxesIcon,
	BoxIcon,
	ChartIcon,
	ClipboardCopyIcon,
	CurrencyIcon,
	EditIcon,
	InfoIcon,
	KeyIcon,
	LockIcon,
	LockOpenIcon,
	LogOutIcon,
	MoreHorizontalIcon,
	ReportIcon,
	ShieldAlertIcon,
} from '@modrinth/assets'
import {
	blockedUsersQueryKey,
	Button,
	type ButtonMenuOption,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	provideUserProfile,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import EditUserModal from '@modrinth/ui/src/layouts/shared/user-profile/components/edit-user-modal.vue'
import ForcePasswordResetModal from '@modrinth/ui/src/layouts/shared/user-profile/components/force-password-reset-modal.vue'
import LockUserModal from '@modrinth/ui/src/layouts/shared/user-profile/components/lock-user-modal.vue'
import Reset2faModal from '@modrinth/ui/src/layouts/shared/user-profile/components/reset-2fa-modal.vue'
import { isStaff, UserBadge } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useClipboard } from '@vueuse/core'
import { computed, reactive, ref } from 'vue'

defineOptions({ inheritAttrs: false })

const menuMessages = defineMessages({
	editUserButton: {
		id: 'profile.button.edit-user',
		defaultMessage: 'Edit user',
	},
	analyticsButton: {
		id: 'profile.button.analytics',
		defaultMessage: 'View user analytics',
	},
	billingButton: {
		id: 'profile.button.billing',
		defaultMessage: 'Manage user billing',
	},
	blockButton: {
		id: 'profile.button.block',
		defaultMessage: 'Block',
	},
	unblockButton: {
		id: 'profile.button.unblock',
		defaultMessage: 'Unblock',
	},
	infoButton: {
		id: 'profile.button.info',
		defaultMessage: 'View user details',
	},
	sharedInstancesButton: {
		id: 'profile.button.shared-instances',
		defaultMessage: 'View shared instances',
	},
	profileManageProjectsButton: {
		id: 'profile.button.manage-projects',
		defaultMessage: 'Manage projects',
	},
	removeAffiliateButton: {
		id: 'profile.button.remove-affiliate',
		defaultMessage: 'Remove as affiliate',
	},
	setAffiliateButton: {
		id: 'profile.button.set-affiliate',
		defaultMessage: 'Set as affiliate',
	},
	lockButton: {
		id: 'profile.button.lock',
		defaultMessage: 'Lock account',
	},
	unlockButton: {
		id: 'profile.button.unlock',
		defaultMessage: 'Unlock account',
	},
	revokeSessionsButton: {
		id: 'profile.button.revoke-sessions',
		defaultMessage: 'Revoke all sessions',
	},
	forcePasswordResetButton: {
		id: 'profile.button.force-password-reset',
		defaultMessage: 'Force password reset',
	},
	reset2faButton: {
		id: 'profile.button.reset-2fa',
		defaultMessage: 'Reset two-factor authentication',
	},
})

const props = defineProps<{ user: Labrinth.Users.v3.User }>()
const auth = useAuthState()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const config = useRuntimeConfig()
const { copy } = useClipboard()
const activated = ref(false)
const isAdmin = computed(() => auth.value.user?.role === 'admin')
const isStaffViewing = computed(() => !!isStaff(auth.value.user))
const userQuery = useQuery({
	queryKey: computed(() => ['user', props.user.id]),
	queryFn: () => client.labrinth.users_v3.get(props.user.id),
	enabled: activated,
	staleTime: 30_000,
})
const user = computed(() => userQuery.data.value ?? props.user)
const blocksQuery = useQuery({
	queryKey: computed(() => blockedUsersQueryKey(auth.value.user?.id)),
	queryFn: () => client.labrinth.blocked_users_v3.list(),
	enabled: computed(() => activated.value && !!auth.value.user),
	staleTime: 30_000,
})
const isBlocked = computed(() => blocksQuery.data.value?.includes(user.value.id) ?? false)

provideUserProfile({
	getUser: (id) => client.labrinth.users_v3.get(id),
	getProjects: (id) => client.labrinth.users_v3.getProjects(id),
	getOrganizations: (id) => client.labrinth.users_v2.getOrganizations(id),
	getCollections: (id) => client.labrinth.users_v2.getCollections(id),
	patchUser: (id, patch) => client.labrinth.users_v2.patch(id, patch),
	changeAvatar: (id, file, extension) => client.labrinth.users_v2.changeIcon(id, file, extension),
	deleteAvatar: (id) => client.labrinth.users_v2.deleteIcon(id),
	getBlockedUsers: () => client.labrinth.blocked_users_v3.list(),
	blockUser: (id) => client.labrinth.blocked_users_v3.block(id),
	unblockUser: (id) => client.labrinth.blocked_users_v3.unblock(id),
})

const editModal = ref<InstanceType<typeof EditUserModal>>()
const lockModal = ref<InstanceType<typeof LockUserModal>>()
const passwordModal = ref<InstanceType<typeof ForcePasswordResetModal>>()
const totpModal = ref<InstanceType<typeof Reset2faModal>>()
const confirmModal = ref<InstanceType<typeof NewModal>>()
const detailsModal = ref<InstanceType<typeof NewModal>>()
const confirmation = ref<'block' | 'revoke'>('block')
const messages = defineMessages({
	blockTitle: {
		id: 'profile.block-user.admonition-title',
		defaultMessage: 'Are you sure you want to block this user?',
	},
	blockBody: {
		id: 'profile.block-user.admonition-body',
		defaultMessage:
			'{username} will not be able to send you friend requests, invite you to shared instances or invite you to Modrinth Hosting servers.',
	},
	revokeBody: {
		id: 'project-review.members.revoke-sessions-body',
		defaultMessage:
			'{username} will be signed out on every device. They will need to sign in again.',
	},
	detailsTitle: { id: 'profile.details.title', defaultMessage: 'User details' },
	emailVerified: {
		id: 'profile.details.label.email-verified',
		defaultMessage: 'Email verified',
	},
	providers: {
		id: 'profile.details.label.auth-providers',
		defaultMessage: 'Auth providers',
	},
	payments: {
		id: 'profile.details.label.payment-methods',
		defaultMessage: 'Payment methods',
	},
	password: {
		id: 'profile.details.label.has-password',
		defaultMessage: 'Has password',
	},
	totp: { id: 'profile.details.label.has-totp', defaultMessage: 'Has TOTP' },
})

type Action = 'block' | 'unblock' | 'affiliate' | 'unlock' | 'revoke'
const mutation = useMutation({
	mutationFn: async (action: Action) => {
		const target = user.value
		if (action === 'block' || action === 'unblock') {
			if (!auth.value.user || auth.value.user.id === target.id) return
			await client.labrinth.blocked_users_v3[action](target.id)
		} else {
			if (!isAdmin.value) return
			if (action === 'affiliate')
				await client.labrinth.users_v2.patch(target.id, {
					badges: target.badges ^ UserBadge.AFFILIATE,
				})
			else if (action === 'unlock' && target.role === 'developer')
				await client.labrinth.moderation_internal.unlockUser(target.id)
			else if (action === 'revoke' && auth.value.user?.id !== target.id)
				await client.labrinth.moderation_internal.revokeUserSessions(target.id)
		}
	},
	onSuccess: () => confirmModal.value?.hide(),
	onSettled: () =>
		Promise.all([
			queryClient.invalidateQueries({ queryKey: ['user', props.user.id] }),
			queryClient.invalidateQueries({
				queryKey: blockedUsersQueryKey(auth.value.user?.id),
			}),
		]),
	onError: (error) =>
		addNotification({
			type: 'error',
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error instanceof Error ? error.message : String(error),
		}),
})

function confirm(action: 'block' | 'revoke') {
	confirmation.value = action
	confirmModal.value?.show()
}

const menuProps = reactive({
	user,
	authUser: computed(() => auth.value.user),
	isSelf: computed(() => auth.value.user?.id === user.value.id),
	isAdmin,
	isStaff: isStaffViewing,
	showStaffActions: true,
	isBlocked,
	isAffiliate: computed(() => !!(user.value.badges & UserBadge.AFFILIATE)),
})
const handlers = {
	block: () => (isBlocked.value ? mutation.mutate('unblock') : confirm('block')),
	copyId: () => {
		void copy(user.value.id)
	},
	copyPermalink: () => {
		void copy(`${config.public.siteUrl}/user/${user.value.id}`)
	},
	toggleAffiliate: () => mutation.mutate('affiliate'),
	openInfo: () => detailsModal.value?.show(),
	editUser: () => editModal.value?.show(),
	toggleLock: () => (user.value.lock ? mutation.mutate('unlock') : lockModal.value?.show()),
	revokeSessions: () => confirm('revoke'),
	forcePasswordReset: () => passwordModal.value?.show(),
	reset2fa: () => totpModal.value?.show(),
}

const menuOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'manage-projects',
		type: 'link',
		label: formatMessage(menuMessages.profileManageProjectsButton),
		icon: BoxIcon,
		href: '/dashboard/projects',
		target: '_blank',
		rel: 'noopener noreferrer',
		shown: menuProps.isSelf,
	},
	{ type: 'divider', shown: menuProps.isSelf },
	{
		id: 'report',
		type: 'link',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		href: `/report?item=user&itemID=${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'red',
		shown: menuProps.authUser?.id !== menuProps.user.id,
	},
	{
		id: 'block',
		label: formatMessage(
			menuProps.isBlocked ? menuMessages.unblockButton : menuMessages.blockButton,
		),
		icon: BanIcon,
		action: () => handlers.block(),
		tone: 'red',
		shown: menuProps.authUser?.id !== menuProps.user.id,
	},
	{
		id: 'copy-id',
		label: formatMessage(commonMessages.copyIdButton),
		icon: ClipboardCopyIcon,
		action: () => handlers.copyId(),
	},
	{
		id: 'copy-permalink',
		label: formatMessage(commonMessages.copyPermalinkButton),
		icon: ClipboardCopyIcon,
		action: () => handlers.copyPermalink(),
	},
	{
		type: 'divider',
		shown: menuProps.showStaffActions && (menuProps.isAdmin || menuProps.isStaff),
	},
	{
		id: 'open-billing',
		type: 'link',
		label: formatMessage(menuMessages.billingButton),
		icon: CurrencyIcon,
		href: `/admin/billing/${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'orange',
		shown: menuProps.showStaffActions && menuProps.isStaff,
	},
	{
		id: 'toggle-affiliate',
		label: menuProps.isAffiliate
			? formatMessage(menuMessages.removeAffiliateButton)
			: formatMessage(menuMessages.setAffiliateButton),
		icon: AffiliateIcon,
		action: () => handlers.toggleAffiliate(),
		shown: menuProps.showStaffActions && menuProps.isAdmin,
		remainOpen: true,
		tone: menuProps.isAffiliate ? 'red' : 'orange',
	},
	{
		id: 'open-info',
		label: formatMessage(menuMessages.infoButton),
		icon: InfoIcon,
		action: () => handlers.openInfo(),
		tone: 'orange',
		shown: menuProps.showStaffActions && menuProps.isStaff,
	},
	{
		id: 'toggle-lock',
		label: formatMessage(menuProps.user.lock ? menuMessages.unlockButton : menuMessages.lockButton),
		icon: menuProps.user.lock ? LockOpenIcon : LockIcon,
		action: () => handlers.toggleLock(),
		tone: 'red',
		shown: menuProps.showStaffActions && menuProps.isAdmin && menuProps.user.role === 'developer',
	},
	{
		id: 'revoke-sessions',
		label: formatMessage(menuMessages.revokeSessionsButton),
		icon: LogOutIcon,
		action: () => handlers.revokeSessions(),
		tone: 'red',
		shown: menuProps.showStaffActions && menuProps.isAdmin && !menuProps.isSelf,
	},
	{
		id: 'force-password-reset',
		label: formatMessage(menuMessages.forcePasswordResetButton),
		icon: KeyIcon,
		action: () => handlers.forcePasswordReset(),
		tone: 'red',
		shown: menuProps.showStaffActions && menuProps.isAdmin && menuProps.user.role === 'developer',
	},
	{
		id: 'reset-2fa',
		label: formatMessage(menuMessages.reset2faButton),
		icon: ShieldAlertIcon,
		action: () => handlers.reset2fa(),
		tone: 'red',
		shown:
			menuProps.showStaffActions &&
			menuProps.isAdmin &&
			menuProps.user.role === 'developer' &&
			Boolean(menuProps.user.has_totp),
	},
	{
		id: 'open-shared-instances',
		type: 'link',
		label: formatMessage(menuMessages.sharedInstancesButton),
		icon: BoxesIcon,
		href: `/admin/shared-instances/${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'orange',
		shown: menuProps.showStaffActions && menuProps.isStaff,
	},
	{
		id: 'open-analytics',
		type: 'link',
		label: formatMessage(menuMessages.analyticsButton),
		icon: ChartIcon,
		href: `/dashboard/analytics?user=${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'orange',
		shown: menuProps.showStaffActions && menuProps.isAdmin,
	},
	{
		id: 'edit-user',
		label: formatMessage(menuMessages.editUserButton),
		icon: EditIcon,
		action: () => handlers.editUser(),
		tone: 'orange',
		shown: menuProps.showStaffActions && menuProps.isAdmin,
	},
])
const options = computed(() =>
	menuOptions.value.map((option) => {
		if (option.type === 'divider' || option.type === 'heading') return option
		const needsUser = !['manage-projects', 'report', 'block', 'copy-id', 'copy-permalink'].includes(
			option.id,
		)
		return {
			...option,
			disabled:
				mutation.isPending.value ||
				(needsUser && (!userQuery.data.value || userQuery.isFetching.value)) ||
				(option.id === 'block' && (!blocksQuery.data.value || blocksQuery.isFetching.value)),
		}
	}),
)

const userDetails = computed(() => {
	const yesNo = (value: unknown) =>
		formatMessage(value ? commonMessages.yesLabel : commonMessages.noLabel)
	return [
		...(isAdmin.value
			? [
					{ label: commonMessages.emailLabel, value: user.value.email ?? '—' },
					{
						label: messages.providers,
						value:
							(user.value.auth_providers ?? [])
								.map((provider) => {
									const id =
										provider === 'discord'
											? user.value.discord_id
											: provider === 'github'
												? user.value.github_id
												: provider === 'steam'
													? user.value.steam_id
													: undefined
									return id ? `${provider} (${id})` : provider
								})
								.join(', ') || '—',
					},
					{
						label: messages.payments,
						value:
							[user.value.payout_data?.paypal_address, user.value.payout_data?.venmo_handle]
								.filter(Boolean)
								.join(', ') || '—',
					},
				]
			: []),
		{ label: messages.emailVerified, value: yesNo(user.value.email_verified) },
		{ label: messages.password, value: yesNo(user.value.has_password) },
		{ label: messages.totp, value: yesNo(user.value.has_totp) },
	]
})
</script>
