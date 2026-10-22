<template>
	<TeleportOverflowMenu
		v-bind="$attrs"
		label="More options"
		:options="menuOptions"
		type="quiet"
		size="sm"
		@open="activated = true"
	>
		<MoreHorizontalIcon aria-hidden="true" />
	</TeleportOverflowMenu>
	<template v-if="activated">
		<EditUserModal v-if="isAdmin" ref="editModal" :user="user" :user-id="user.id" />
		<NewModal v-if="isStaffViewing" ref="detailsModal" header="User details">
			<dl class="m-0 flex flex-col gap-3">
				<div v-for="detail in userDetails" :key="detail.label">
					<dt class="font-semibold text-contrast">
						{{ detail.label }}
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
	BoxesIcon,
	ChartIcon,
	ClipboardCopyIcon,
	EditIcon,
	InfoIcon,
	MoreHorizontalIcon,
} from '@modrinth/assets'
import {
	type ButtonMenuOption,
	injectModrinthClient,
	NewModal,
	provideUserProfile,
	TeleportOverflowMenu,
	type UserProfileContext,
} from '@modrinth/ui'
import EditUserModal from '@modrinth/ui/src/layouts/shared/user-profile/components/edit-user-modal.vue'
import { isStaff } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { useClipboard } from '@vueuse/core'
import { computed, ref } from 'vue'

defineOptions({ inheritAttrs: false })

const props = defineProps<{ user: Labrinth.Users.v3.User }>()
const auth = useAuthState()
const client = injectModrinthClient()

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

provideUserProfile({
	getUser: (id: string) => client.labrinth.users_v3.get(id),
	getProjects: (id: string) => client.labrinth.users_v3.getProjects(id),
	getOrganizations: (id: string) => client.labrinth.users_v2.getOrganizations(id),
	getCollections: (id: string) => client.labrinth.users_v2.getCollections(id),
	patchUser: (id: string, patch: Parameters<UserProfileContext['patchUser']>[1]) =>
		client.labrinth.users_v2.patch(id, patch),
	changeAvatar: (id: string, file: Blob, extension: string) =>
		client.labrinth.users_v2.changeIcon(id, file, extension),
	deleteAvatar: (id: string) => client.labrinth.users_v2.deleteIcon(id),
	getBlockedUsers: () => client.labrinth.blocked_users_v3.list(),
	blockUser: (id: string) => client.labrinth.blocked_users_v3.block(id),
	unblockUser: (id: string) => client.labrinth.blocked_users_v3.unblock(id),
} satisfies UserProfileContext)

const editModal = ref<InstanceType<typeof EditUserModal>>()
const detailsModal = ref<InstanceType<typeof NewModal>>()

const menuOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'copy-id',
		label: 'Copy ID',
		icon: ClipboardCopyIcon,
		action: () => {
			void copy(user.value.id)
		},
	},
	{
		id: 'copy-permalink',
		label: 'Copy permanent link',
		icon: ClipboardCopyIcon,
		action: () => {
			void copy(`${config.public.siteUrl}/user/${user.value.id}`)
		},
	},
	{ type: 'divider', shown: isStaffViewing.value },
	{
		id: 'open-info',
		label: 'View user details',
		icon: InfoIcon,
		action: () => detailsModal.value?.show(),
		tone: 'orange',
		shown: isStaffViewing.value,
		disabled: !userQuery.data.value || userQuery.isFetching.value,
	},
	{
		id: 'open-shared-instances',
		type: 'link',
		label: 'View shared instances',
		icon: BoxesIcon,
		href: `/admin/shared-instances/${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'orange',
		shown: isStaffViewing.value,
		disabled: !userQuery.data.value || userQuery.isFetching.value,
	},
	{
		id: 'open-analytics',
		type: 'link',
		label: 'View user analytics',
		icon: ChartIcon,
		href: `/dashboard/analytics?user=${encodeURIComponent(user.value.id)}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		tone: 'orange',
		shown: isAdmin.value,
		disabled: !userQuery.data.value || userQuery.isFetching.value,
	},
	{ type: 'divider', shown: isAdmin.value },
	{
		id: 'edit-user',
		label: 'Edit user',
		icon: EditIcon,
		action: () => editModal.value?.show(),
		tone: 'default',
		shown: isAdmin.value,
		disabled: !userQuery.data.value || userQuery.isFetching.value,
	},
])

const userDetails = computed(() => {
	const yesNo = (value: unknown) => (value ? 'Yes' : 'No')
	return [
		...(isAdmin.value
			? [
					{ label: 'Email', value: user.value.email ?? '—' },
					{
						label: 'Auth providers',
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
						label: 'Payment methods',
						value:
							[user.value.payout_data?.paypal_address, user.value.payout_data?.venmo_handle]
								.filter(Boolean)
								.join(', ') || '—',
					},
				]
			: []),
		{ label: 'Email verified', value: yesNo(user.value.email_verified) },
		{ label: 'Has password', value: yesNo(user.value.has_password) },
		{ label: 'Has TOTP', value: yesNo(user.value.has_totp) },
	]
})
</script>
