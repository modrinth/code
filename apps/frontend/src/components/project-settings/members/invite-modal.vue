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
import { createReusableTemplate, refDebounced, until } from '@vueuse/core'
import Fuse from 'fuse.js'
import { computed, nextTick, ref, shallowRef, useId, useTemplateRef, watch } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import ProjectAccessPicker, {
	type ProjectAccessGroup,
	type ProjectAccessPreset,
} from './member-editor/project-access-picker.vue'
import { hasPermission, viewerPermissions } from './permissions'
import { useMemberErrorNotification } from './use-member-error-notification'

type InviteUser = Pick<Labrinth.Users.v2.User, 'id' | 'username' | 'avatar_url'>

const CONTACTS_TIMEOUT_MS = 5_000

const props = defineProps<{
	canInvite: boolean
	grantable?: number
	accessOptions?: { presets: ProjectAccessPreset[]; groups: ProjectAccessGroup[] }
	title: string
	teamId: string
	teamMembersByUserId?: ReadonlyMap<string, { accepted: boolean }>
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
const isOpening = ref(false)
const username = ref('')
const usernameFocused = ref(false)
const usernameResultsId = useId()
const highlightedUserId = ref<string>()
const selectedUser = shallowRef<InviteUser>()
const role = ref('')
const permissions = ref(0)
const [DefineUsernameStage, UsernameStage] = createReusableTemplate()
const [DefineAccessStage, AccessStage] = createReusableTemplate()
const [DefineUserRow, UserRow] = createReusableTemplate<{
	user: InviteUser
	exactMatch?: boolean
	optionId?: string
	highlighted?: boolean
}>({ inheritAttrs: false })

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
	alreadyMember: {
		id: 'project.settings.members.invite.already-member',
		defaultMessage: 'Already a member',
	},
	invitePending: {
		id: 'project.settings.members.invite.pending',
		defaultMessage: 'Invite pending',
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
const canLoadContacts = computed(
	() => (isOpening.value || isOpen.value) && props.canInvite && !!userId.value,
)
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
const contactsPending = computed(() => friends.isPending.value || organizations.isPending.value)
const contactsFailed = computed(() => friends.isError.value || organizations.isError.value)
const contactCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })
const visibleFriends = computed(() =>
	[...(friends.data.value ?? [])].sort((a, b) => contactCollator.compare(a.username, b.username)),
)
const visibleOrganizations = computed(() =>
	(organizations.data.value ?? [])
		.flatMap((organization) => {
			const acceptedMembers = organization.members.filter((member) => member.accepted)
			if (
				acceptedMembers.length < 2 ||
				!acceptedMembers.some((member) => member.user.id === userId.value)
			)
				return []

			const members = acceptedMembers
				.filter(({ user }) => user.id !== userId.value)
				.sort((a, b) => contactCollator.compare(a.user.username, b.user.username))
			return [{ organization, members, memberCount: members.length }]
		})
		.sort((a, b) => contactCollator.compare(a.organization.name, b.organization.name)),
)
const contactSearch = computed(() => {
	const contacts = [
		...visibleFriends.value,
		...visibleOrganizations.value.flatMap(({ members }) => members.map(({ user }) => user)),
	]
	const eligibleUsers = new Map<string, InviteUser>()
	for (const user of contacts) {
		if (!props.teamMembersByUserId?.has(user.id)) eligibleUsers.set(user.id, user)
	}
	return new Fuse([...eligibleUsers.values()], { keys: ['username'], threshold: 0.4 })
})
const usernameResults = computed(() => {
	const exactUser = checkedUser.value
	if (exactUser === undefined) return []
	const matches = contactSearch.value.search(enteredUsername.value).map(({ item }) => item)
	return exactUser ? [exactUser, ...matches.filter((user) => user.id !== exactUser.id)] : matches
})
const exactMatch = computed(() =>
	usernameResults.value.find((user) => user.username.toLowerCase() === enteredUsername.value),
)
const selectableResults = computed(() =>
	usernameResults.value.filter((user) => !props.teamMembersByUserId?.has(user.id)),
)
const highlightedUser = computed(
	() =>
		selectableResults.value.find((user) => user.id === highlightedUserId.value) ??
		selectableResults.value[0],
)
const resultsExpanded = computed(
	() => props.canInvite && usernameFocused.value && usernameResults.value.length > 0,
)
watch(enteredUsername, () => {
	highlightedUserId.value = undefined
})
const contactsStatus = computed(() => {
	if (!canLoadContacts.value) return
	if (contactsPending.value) return commonMessages.loadingLabel
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
		!props.teamMembersByUserId?.has(selectedUser.value.id) &&
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
	if (!user || !props.canInvite || inviting.value || props.teamMembersByUserId?.has(user.id)) return
	selectedUser.value = user
	modal.value?.nextStage()
	await nextTick()
	roleInput.value?.focus()
}

async function moveHighlight(event: KeyboardEvent, direction: -1 | 1) {
	const results = selectableResults.value
	if (event.isComposing || !results.length) return
	event.preventDefault()
	const index = results.findIndex((user) => user.id === highlightedUser.value?.id)
	highlightedUserId.value = results[(index + direction + results.length) % results.length]?.id
	await nextTick()
	usernameForm.value
		?.querySelector('[role="option"][aria-selected="true"]')
		?.scrollIntoView({ block: 'nearest' })
}

function highlightUserFromPointer(event: PointerEvent, user: InviteUser) {
	if (event.pointerType === 'touch' || !props.canInvite || props.teamMembersByUserId?.has(user.id))
		return
	highlightedUserId.value = user.id
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

async function show() {
	if (isOpening.value || isOpen.value || !props.canInvite) return
	const teamId = props.teamId
	isOpening.value = true
	try {
		await nextTick()
		await until(() => !canLoadContacts.value || !contactsPending.value).toBe(true, {
			timeout: CONTACTS_TIMEOUT_MS,
		})
		if (!modal.value || !props.canInvite || props.teamId !== teamId) return

		username.value = ''
		selectedUser.value = undefined
		role.value = defaultRole.value
		permissions.value =
			props.accessOptions && hasPermission(props.grantable, viewerPermissions)
				? viewerPermissions
				: 0
		modal.value.setStage('username')
		isOpen.value = true
		modal.value.show()
	} finally {
		isOpening.value = false
	}
}

defineExpose({ show, isOpening })
</script>

<template>
	<DefineUserRow v-slot="{ user, exactMatch, optionId, highlighted }">
		<button
			:id="optionId"
			type="button"
			:role="optionId ? 'option' : undefined"
			:tabindex="optionId ? -1 : undefined"
			:aria-selected="optionId ? !!highlighted : undefined"
			class="group/invite-user flex w-full items-center border-0 text-left font-semibold text-contrast disabled:opacity-60 motion-safe:transition-colors"
			:class="[
				optionId
					? 'gap-2.5 rounded-none px-4 py-2'
					: 'gap-3 rounded-xl py-2 pl-3 pr-2 focus-visible:bg-surface-3 enabled:hover:bg-surface-3',
				highlighted ? 'bg-surface-4 brightness-[115%]' : 'bg-transparent',
			]"
			:disabled="!canInvite || teamMembersByUserId?.has(user.id)"
			@mousedown="optionId && $event.preventDefault()"
			@pointermove="optionId && highlightUserFromPointer($event, user)"
			@click="selectUser(user)"
		>
			<Avatar
				:src="user.avatar_url"
				:tint-by="user.username"
				:loading="exactMatch ? 'eager' : 'lazy'"
				circle
				no-shadow
			/>
			<span class="min-w-0 flex-1 truncate group-disabled/invite-user:text-secondary">
				{{ user.username }}
			</span>
			<span
				v-if="teamMembersByUserId?.has(user.id)"
				class="text-right text-xs font-normal text-secondary"
			>
				{{
					formatMessage(
						teamMembersByUserId.get(user.id)?.accepted
							? messages.alreadyMember
							: messages.invitePending,
					)
				}}
			</span>
			<RightArrowIcon
				v-else
				class="size-4 shrink-0 motion-safe:transition-opacity"
				:class="{
					'opacity-0': optionId && !highlighted,
					'mr-1 opacity-0 group-hover/invite-user:opacity-100 group-focus-visible/invite-user:opacity-100':
						!optionId,
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
				@submit.prevent="selectUser(highlightedUser)"
			>
				<Input
					v-model="username"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					:aria-label="formatMessage(messages.username)"
					role="combobox"
					aria-autocomplete="list"
					:aria-expanded="resultsExpanded"
					:aria-controls="usernameResults.length ? usernameResultsId : undefined"
					:aria-activedescendant="
						resultsExpanded && highlightedUser
							? `${usernameResultsId}-${highlightedUser.id}`
							: undefined
					"
					:disabled="!canInvite"
					size="medium"
					autocomplete="off"
					autocapitalize="off"
					:spellcheck="false"
					wrapper-class="w-full"
					@focus="usernameFocused = true"
					@blur="usernameFocused = false"
					@keydown.down="moveHighlight($event, 1)"
					@keydown.up="moveHighlight($event, -1)"
				/>
				<div
					v-if="enteredUsername && canInvite"
					class="username-dropdown absolute inset-x-0 top-full z-10 mt-2 hidden max-h-72 overflow-y-auto rounded-[14px] border border-solid border-surface-5 bg-surface-4 shadow-2xl group-focus-within/lookup:block"
				>
					<div
						v-if="usernameResults.length"
						:id="usernameResultsId"
						role="listbox"
						:aria-label="formatMessage(messages.username)"
					>
						<UserRow
							v-for="user in usernameResults"
							:key="user.id"
							:user="user"
							:exact-match="user.id === exactMatch?.id"
							:option-id="`${usernameResultsId}-${user.id}`"
							:highlighted="user.id === highlightedUser?.id"
						/>
					</div>
					<div
						v-else
						class="flex flex-wrap items-center gap-x-2 px-4 py-3 text-sm text-secondary"
						role="status"
					>
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
						<Avatar :src="org.icon_url" :tint-by="org.id" no-shadow />
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
.username-dropdown {
	scrollbar-color: var(--surface-5) transparent;
	scrollbar-width: thin;
}

@media (prefers-reduced-motion: no-preference) {
	.organization-accordion :deep(.accordion-content) {
		transition-duration: 150ms;
	}
}
</style>
