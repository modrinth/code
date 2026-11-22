<script setup lang="ts">
import { ModrinthApiError } from '@modrinth/api-client'
import { ArrowLeftIcon, RightArrowIcon, UserPlusIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	Field,
	injectModrinthClient,
	Input,
	MultiStageModal,
	type StageConfigInput,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { createReusableTemplate, refDebounced } from '@vueuse/core'
import { computed, nextTick, ref, useId, useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import ProjectAccessPicker, {
	type ProjectAccessGroup,
	type ProjectAccessPreset,
} from './member-editor/project-access-picker.vue'
import { hasPermission, viewerPermissions } from './permissions'
import { useMemberErrorNotification } from './use-member-error-notification'

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
const notifyError = useMemberErrorNotification()
const loading = useLoading()
const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')
const usernameForm = useTemplateRef<HTMLFormElement>('usernameForm')
const roleInput = useTemplateRef('roleInput')
const id = useId()
const username = ref('')
const role = ref('')
const permissions = ref(0)
const [DefineUsernameStage, UsernameStage] = createReusableTemplate()
const [DefineAccessStage, AccessStage] = createReusableTemplate()

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
})

const defaultRole = computed(() => formatMessage(messages.defaultRole))
const enteredUsername = computed(() => username.value.trim())
const debouncedUsername = refDebounced(enteredUsername, 300)
const canCheckUsername = computed(
	() =>
		props.canInvite && !!enteredUsername.value && enteredUsername.value === debouncedUsername.value,
)
const usernameLookup = useQuery({
	queryKey: computed(() => ['user', 'invite-lookup', debouncedUsername.value] as const),
	enabled: canCheckUsername,
	queryFn: async ({ queryKey: [, , name] }) => {
		try {
			return await labrinth.users_v2.get(name)
		} catch (error) {
			if (error instanceof ModrinthApiError && error.statusCode === 404) return null
			throw error
		}
	},
	retry: false,
})
const checkedUser = computed(() => (canCheckUsername.value ? usernameLookup.data.value : undefined))

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

const canContinue = computed(
	() =>
		canCheckUsername.value &&
		!inviting.value &&
		!usernameLookup.isFetching.value &&
		(!!checkedUser.value || (checkedUser.value === undefined && usernameLookup.isError.value)),
)
const canSubmit = computed(
	() =>
		props.canInvite &&
		!!checkedUser.value &&
		!inviting.value &&
		(permissions.value & (props.grantable ?? 0)) === permissions.value,
)

const stages: StageConfigInput<undefined>[] = [
	{
		id: 'username',
		title: () => props.title,
		stageContent: UsernameStage,
		maxWidth: '450px',
		actionsTopMargin: 'none',
		leftButtonConfig: null,
		rightButtonConfig: () => ({
			label: formatMessage(commonMessages.nextButton),
			icon: RightArrowIcon,
			iconPosition: 'after',
			color: 'brand',
			disabled: !canContinue.value,
			tooltip: checkedUser.value === null ? formatMessage(messages.userNotFound) : undefined,
			onClick: advance,
		}),
	},
	{
		id: 'access',
		title: () => formatMessage(messages.inviteUser, { username: username.value.trim() }),
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

async function advance() {
	if (!canContinue.value) return
	if (usernameLookup.isError.value) {
		const name = enteredUsername.value
		const { error } = await usernameLookup.refetch()
		if (name !== enteredUsername.value) return
		if (error) {
			notifyError(error)
			return
		}
	}
	if (!checkedUser.value) return
	modal.value?.nextStage()
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

async function submit() {
	const user = checkedUser.value
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
	role.value = defaultRole.value
	permissions.value =
		props.accessOptions && hasPermission(props.grantable, viewerPermissions) ? viewerPermissions : 0
	modal.value?.setStage('username')
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<DefineUsernameStage>
		<p class="m-0 mb-4">
			{{ formatMessage(organization ? messages.organizationDescription : messages.description) }}
		</p>
		<form ref="usernameForm" class="pb-2" @submit.prevent="advance">
			<Input
				:id="`${id}-username`"
				v-model="username"
				:placeholder="formatMessage(messages.usernamePlaceholder)"
				:aria-label="formatMessage(messages.username)"
				:disabled="!canInvite"
				wrapper-class="w-full"
			/>
		</form>
	</DefineUsernameStage>
	<DefineAccessStage>
		<form class="flex flex-col gap-6" @submit.prevent="submit">
			<Field :label="formatMessage(messages.role)" :input-id="`${id}-role`">
				<Input
					:id="`${id}-role`"
					ref="roleInput"
					v-model="role"
					:maxlength="32"
					wrapper-class="w-full"
					:disabled="!canInvite || inviting"
					@focus="selectDefaultRole"
				/>
			</Field>
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
	/>
</template>
