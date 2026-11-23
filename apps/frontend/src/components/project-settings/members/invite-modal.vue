<script setup lang="ts">
import { type Labrinth, ModrinthApiError } from '@modrinth/api-client'
import { ArrowLeftIcon, ChevronDownIcon, RightArrowIcon, UserPlusIcon } from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	commonMessages,
	defineMessages,
	injectAuth,
	injectModrinthClient,
	Input,
	MultiStageModal,
	sharedInstanceKeys,
	type StageConfigInput,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { createReusableTemplate, refDebounced } from '@vueuse/core'
import { computed, nextTick, ref, shallowRef, useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import ProjectAccessPicker, {
	type ProjectAccessGroup,
	type ProjectAccessPreset,
} from './member-editor/project-access-picker.vue'
import { hasPermission, viewerPermissions } from './permissions'
import { useMemberErrorNotification } from './use-member-error-notification'

type InviteUser = Pick<Labrinth.Users.v2.User, 'id' | 'username' | 'avatar_url'>

const props = defineProps<{
	canInvite: boolean
	grantable?: number
	accessOptions?: { presets: ProjectAccessPreset[]; groups: ProjectAccessGroup[] }
	title: string
	teamId: string
	organization?: boolean
	onInvited: () => Promise<unknown>
}>()
const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()
const auth = injectAuth()
const notifyError = useMemberErrorNotification()
const loading = useLoading()
const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')
const usernameForm = useTemplateRef<HTMLFormElement>('usernameForm')
const roleInput = useTemplateRef('roleInput')
const isOpen = ref(false)
const username = ref('')
const selectedUser = shallowRef<InviteUser>()
const role = ref('')
const permissions = ref(0)
const [DefineUsernameStage, UsernameStage] = createReusableTemplate()
const [DefineAccessStage, AccessStage] = createReusableTemplate()
const [DefineUserRow, UserRow] = createReusableTemplate<{ user: InviteUser; exactMatch?: boolean }>(
	{
		inheritAttrs: false,
	},
)

const messages = defineMessages({
	description: {
		id: 'project.settings.members.invite.description',
		defaultMessage: 'They’ll get an invite and join the project once they accept.',
	},
	organizationDescription: {
		id: 'organization.settings.members.invite.description',
		defaultMessage: 'They’ll get an invite and join the organization once they accept.',
	},
	username: { id: 'project.settings.members.invite.username', defaultMessage: 'Username' },
	usernamePlaceholder: {
		id: 'project.settings.members.invite.username-placeholder',
		defaultMessage: 'Enter a Modrinth username',
	},
	role: { id: 'project.settings.members.column.role', defaultMessage: 'Role title' },
	defaultRole: {
		id: 'project.settings.members.invite.default-role',
		defaultMessage: 'Member',
	},
	invite: { id: 'project.settings.members.invite.button', defaultMessage: 'Invite' },
	inviteUser: {
		id: 'project.settings.members.invite.user-title',
		defaultMessage: 'Invite {username}',
	},
	userNotFound: {
		id: 'project.settings.members.invite.user-not-found',
		defaultMessage: 'User does not exist',
	},
	lookupFailed: {
		id: 'project.settings.members.invite.lookup-failed',
		defaultMessage: 'Couldn’t look up that username.',
	},
	friends: {
		id: 'project.settings.members.invite.friends',
		defaultMessage: 'Friends',
	},
	organizations: {
		id: 'project.settings.members.invite.organizations',
		defaultMessage: 'Your organizations',
	},
	memberCount: {
		id: 'project.settings.members.invite.member-count',
		defaultMessage: '{count, plural, one {# member} other {# members}}',
	},
	noContacts: {
		id: 'project.settings.members.invite.no-contacts',
		defaultMessage: 'No friends or organization members to show.',
	},
	contactsFailed: {
		id: 'project.settings.members.invite.contacts-failed',
		defaultMessage: 'Couldn’t load all your friends and organizations.',
	},
})

const defaultRole = computed(() => formatMessage(messages.defaultRole))
const enteredUsername = computed(() => username.value.trim().toLowerCase())
const debouncedUsername = refDebounced(enteredUsername, 300)
const canCheckUsername = computed(
	() =>
		isOpen.value &&
		props.canInvite &&
		!!enteredUsername.value &&
		enteredUsername.value === debouncedUsername.value,
)
const usernameLookup = useQuery({
	queryKey: computed(() => ['user', 'invite-lookup', debouncedUsername.value] as const),
	enabled: canCheckUsername,
	queryFn: async ({ queryKey: [, , name] }) => {
		try {
			const user = await labrinth.users_v2.get(encodeURIComponent(name))
			return user.username.toLowerCase() === name ? user : null
		} catch (error) {
			if (error instanceof ModrinthApiError && error.statusCode === 404) return null
			throw error
		}
	},
	retry: false,
})
const checkedUser = computed(() =>
	canCheckUsername.value && !usernameLookup.isFetching.value && !usernameLookup.isError.value
		? usernameLookup.data.value
		: undefined,
)
const userId = computed(() => auth.user.value?.id)
const canLoadContacts = computed(() => isOpen.value && props.canInvite && !!userId.value)
const friends = useQuery({
	queryKey: computed(() => sharedInstanceKeys.friends(userId.value)),
	enabled: canLoadContacts,
	queryFn: async ({ queryKey: [, , currentUserId] }) => {
		const relationships = await labrinth.friends_v3.list()
		const ids = relationships
			.filter((friend) => friend.accepted)
			.map((friend) => (friend.id === currentUserId ? friend.friend_id : friend.id))
		return ids.length ? labrinth.users_v2.getMultiple(ids) : []
	},
})
const organizations = useQuery({
	queryKey: computed(() => ['user', userId.value, 'organizations'] as const),
	enabled: canLoadContacts,
	queryFn: ({ queryKey: [, id] }) => labrinth.users_v2.getOrganizations(id!),
})
const loadingContacts = computed(() => friends.isLoading.value || organizations.isLoading.value)
const contactsFailed = computed(() => friends.isError.value || organizations.isError.value)
const visibleFriends = computed(() => friends.data.value ?? [])
const visibleOrganizations = computed(() =>
	(organizations.data.value ?? []).flatMap((organization) => {
		const acceptedMembers = organization.members.filter((member) => member.accepted)
		if (
			acceptedMembers.length < 2 ||
			!acceptedMembers.some((member) => member.user.id === userId.value)
		)
			return []

		const members = acceptedMembers.filter(({ user }) => user.id !== userId.value)
		return [{ organization, members, memberCount: members.length }]
	}),
)
const contactsStatus = computed(() => {
	if (!canLoadContacts.value) return
	if (loadingContacts.value) return commonMessages.loadingLabel
	if (contactsFailed.value) return messages.contactsFailed
	if (visibleFriends.value.length || visibleOrganizations.value.length) return
	return messages.noContacts
})

const { mutateAsync: invite, isPending: inviting } = useMutation({
	mutationFn: async ({
		team,
		userId,
		role,
		permissions,
	}: {
		team: string
		userId: string
		role?: string
		permissions: number
	}) => {
		await labrinth.teams_v2.addMember(team, { user_id: userId, role, permissions })
	},
	onSuccess: async () => {
		await props.onInvited()
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

const canSubmit = computed(
	() =>
		props.canInvite &&
		!!selectedUser.value &&
		!inviting.value &&
		(permissions.value & (props.grantable ?? 0)) === permissions.value,
)

const stages: StageConfigInput<undefined>[] = [
	{
		id: 'username',
		title: () => props.title,
		stageContent: UsernameStage,
		maxWidth: '526px',
		animateResize: false,
		actionsTopMargin: 'none',
		leftButtonConfig: null,
		rightButtonConfig: () => ({
			label: formatMessage(commonMessages.cancelButton),
			onClick: () => modal.value?.hide(),
		}),
	},
	{
		id: 'access',
		title: () =>
			formatMessage(messages.inviteUser, { username: selectedUser.value?.username ?? '' }),
		stageContent: AccessStage,
		maxWidth: '38rem',
		leftButtonConfig: () => ({
			label: formatMessage(commonMessages.backButton),
			icon: ArrowLeftIcon,
			disabled: inviting.value,
			onClick: back,
		}),
		rightButtonConfig: () => ({
			label: formatMessage(messages.invite),
			icon: UserPlusIcon,
			iconPosition: 'before',
			color: 'brand',
			disabled: !canSubmit.value,
			loading: inviting.value,
			onClick: submit,
		}),
	},
]

async function selectUser(user?: InviteUser | null) {
	if (!user || !props.canInvite || inviting.value) return
	selectedUser.value = user
	modal.value?.nextStage()
	await nextTick()
	roleInput.value?.focus()
}

function selectDefaultRole() {
	if (role.value === defaultRole.value) roleInput.value?.select()
}

async function back() {
	modal.value?.prevStage()
	await nextTick()
	getUsernameInput()?.focus()
}

function getUsernameInput() {
	return usernameForm.value?.querySelector('input') ?? null
}

function retryUsernameLookup() {
	getUsernameInput()?.focus()
	void usernameLookup.refetch()
}

async function submit() {
	const user = selectedUser.value
	if (!canSubmit.value || !user) return
	const roleTitle = role.value.trim()
	loading.value = true
	try {
		await invite({
			team: props.teamId,
			userId: user.id.trim(),
			role: roleTitle === defaultRole.value ? undefined : roleTitle || undefined,
			permissions: permissions.value,
		})
	} catch {
		return
	}
	await nextTick()
	modal.value?.hide()
}

function show() {
	username.value = ''
	selectedUser.value = undefined
	role.value = defaultRole.value
	permissions.value =
		props.accessOptions && hasPermission(props.grantable, viewerPermissions) ? viewerPermissions : 0
	modal.value?.setStage('username')
	isOpen.value = true
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<DefineUserRow v-slot="{ user, exactMatch }">
		<button
			type="button"
			class="group/invite-user flex w-full items-center gap-3 rounded-xl border-0 py-2 pr-2 text-left font-semibold text-contrast motion-safe:transition-colors"
			:class="
				exactMatch
					? 'bg-surface-4 pl-2 hover:bg-surface-5 focus-visible:bg-surface-5'
					: 'bg-transparent pl-3 hover:bg-surface-3 focus-visible:bg-surface-3'
			"
			:disabled="!canInvite"
			@mousedown="exactMatch && $event.preventDefault()"
			@click="selectUser(user)"
		>
			<Avatar
				v-if="user.avatar_url"
				:src="user.avatar_url"
				:loading="exactMatch ? 'eager' : 'lazy'"
				circle
				no-shadow
			/>
			<span
				v-else
				class="flex size-8 shrink-0 items-center justify-center rounded-full bg-brand-highlight text-xs"
				aria-hidden="true"
			>
				{{ user.username.slice(0, 2).toUpperCase() }}
			</span>
			<span class="min-w-0 flex-1 truncate">{{ user.username }}</span>
			<RightArrowIcon
				class="mr-1 size-4 shrink-0 motion-safe:transition-opacity"
				:class="{
					'opacity-0 group-hover/invite-user:opacity-100 group-focus-visible/invite-user:opacity-100':
						!exactMatch,
				}"
				aria-hidden="true"
			/>
		</button>
	</DefineUserRow>
	<DefineUsernameStage>
		<div class="flex flex-col gap-5 pb-5">
			<p class="m-0">
				{{ formatMessage(organization ? messages.organizationDescription : messages.description) }}
			</p>
			<form
				ref="usernameForm"
				class="group/lookup relative"
				@submit.prevent="selectUser(checkedUser)"
			>
				<Input
					v-model="username"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					:aria-label="formatMessage(messages.username)"
					:disabled="!canInvite"
					size="medium"
					autocomplete="off"
					autocapitalize="off"
					:spellcheck="false"
					wrapper-class="w-full"
				/>
				<div
					v-if="enteredUsername && canInvite"
					class="absolute inset-x-0 top-full z-10 mt-2 hidden rounded-xl border border-solid border-surface-5 bg-surface-3 p-1 shadow-lg group-focus-within/lookup:block"
				>
					<UserRow v-if="checkedUser" :user="checkedUser" :exact-match="true" />
					<div v-else class="flex flex-wrap items-center gap-x-2 px-2 py-2 text-sm" role="status">
						<template v-if="checkedUser === null">
							{{ formatMessage(messages.userNotFound) }}
						</template>
						<template
							v-else-if="
								canCheckUsername && usernameLookup.isError.value && !usernameLookup.isFetching.value
							"
						>
							{{ formatMessage(messages.lookupFailed) }}
							<button
								type="button"
								class="border-0 bg-transparent p-0 font-semibold text-contrast underline"
								@mousedown.prevent
								@click="retryUsernameLookup"
							>
								{{ formatMessage(commonMessages.retryButton) }}
							</button>
						</template>
						<template v-else>{{ formatMessage(commonMessages.loadingLabel) }}</template>
					</div>
				</div>
			</form>
		</div>
		<div
			class="-mx-6 flex flex-col gap-4 border-0 border-y border-solid border-surface-4 bg-surface-2 px-6 py-4"
		>
			<section v-if="visibleFriends.length" :aria-label="formatMessage(messages.friends)">
				<h3 class="m-0 my-1 text-xs font-semibold uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.friends) }}
				</h3>
				<div class="-ml-3 -mr-2">
					<UserRow v-for="friend in visibleFriends" :key="friend.id" :user="friend" />
				</div>
			</section>
			<section
				v-if="visibleOrganizations.length"
				:aria-label="formatMessage(messages.organizations)"
			>
				<h3 class="m-0 my-1 text-xs font-semibold uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.organizations) }}
				</h3>
				<Accordion
					v-for="{ organization: org, members, memberCount } in visibleOrganizations"
					:key="org.id"
					class="organization-accordion -ml-3 -mr-2"
					button-class="flex w-full items-center gap-3 rounded-xl border-0 bg-transparent py-2 pl-3 pr-2 text-left text-contrast hover:bg-surface-3 focus-visible:bg-surface-3 motion-safe:transition-colors"
					content-class="ml-7 border-0 border-l border-solid border-surface-4 pl-1"
				>
					<template #button="{ open }">
						<Avatar v-if="org.icon_url" :src="org.icon_url" no-shadow />
						<span
							v-else
							class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-surface-4 text-xs font-semibold"
							aria-hidden="true"
						>
							{{ org.name.slice(0, 2).toUpperCase() }}
						</span>
						<span class="min-w-0 flex-1 truncate font-semibold">{{ org.name }}</span>
						<span class="text-right text-xs text-secondary">
							{{ formatMessage(messages.memberCount, { count: memberCount }) }}
						</span>
						<ChevronDownIcon
							class="mr-1 size-4 shrink-0 text-secondary motion-safe:transition-transform"
							:class="{ 'rotate-180': open }"
							aria-hidden="true"
						/>
					</template>
					<UserRow v-for="{ user } in members" :key="user.id" :user="user" />
				</Accordion>
			</section>
			<p v-if="contactsStatus" class="m-0 text-sm text-secondary" role="status">
				{{ formatMessage(contactsStatus) }}
			</p>
		</div>
	</DefineUsernameStage>
	<DefineAccessStage>
		<form class="flex flex-col gap-6" @submit.prevent="submit">
			<label class="flex min-w-0 flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.role) }}</span>
				<Input
					ref="roleInput"
					v-model="role"
					:maxlength="32"
					wrapper-class="w-full"
					:disabled="!canInvite || inviting"
					@focus="selectDefaultRole"
				/>
			</label>
			<ProjectAccessPicker
				v-if="accessOptions"
				v-model="permissions"
				:presets="accessOptions.presets"
				:groups="accessOptions.groups"
				:grantable="grantable ?? 0"
				:disabled="!canInvite || inviting"
			/>
		</form>
	</DefineAccessStage>
	<MultiStageModal
		ref="modal"
		:stages="stages"
		:context="undefined"
		disable-progress
		:disable-close="inviting"
		:initial-focus="getUsernameInput"
		@hide="isOpen = false"
	/>
</template>

<style scoped>
@media (prefers-reduced-motion: no-preference) {
	.organization-accordion :deep(.accordion-content) {
		transition-duration: 150ms;
	}
}
</style>
