<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		width="min(34rem, calc(100vw - 2rem))"
		max-width="34rem"
	>
		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="grant-access-target">
					{{ formatMessage(messages.targetLabel) }}
				</label>
				<Combobox
					id="grant-access-target"
					:model-value="undefined"
					:options="suggestionOptions"
					:search-placeholder="formatMessage(messages.targetPlaceholder)"
					:placeholder="formatMessage(messages.targetPlaceholder)"
					:no-options-message="targetLookupMessage"
					:min-search-length-to-open="suggestionMinimumLength"
					:disable-search-filter="usesRemoteLookup"
					searchable
					show-search-icon
					:show-chevron="false"
					search-autocomplete="off"
					search-autocorrect="off"
					search-autocapitalize="none"
					:search-spellcheck="false"
					@search-input="handleTargetSearch"
					@select="handleTargetSelect"
				>
					<template #option="{ item, isSelected }">
						<div class="flex min-w-0 items-center gap-2">
							<Avatar
								:src="findSuggestion(item.value)?.avatarUrl"
								:alt="formatMessage(messages.suggestionAvatarAlt, { username: item.label })"
								:tint-by="item.label"
								size="1.5rem"
								circle
								no-shadow
							/>
							<span
								class="min-w-0 truncate font-semibold"
								:class="isSelected ? 'text-contrast' : 'text-primary'"
							>
								{{ item.label }}
							</span>
						</div>
					</template>
				</Combobox>
				<span class="m-0 text-base text-primary">
					{{ formatMessage(messages.targetHelp) }}
				</span>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.roleLabel) }}
				</span>
				<div class="flex flex-col gap-3">
					<button
						v-for="role in grantableRoles"
						:key="role.value"
						type="button"
						class="group flex w-full items-center gap-3 rounded-[20px] border border-solid p-3 text-left transition-all hover:brightness-110 active:scale-[0.98]"
						:class="
							selectedRole === role.value
								? 'border-brand bg-brand-highlight'
								: 'border-transparent bg-surface-4'
						"
						@click="selectedRole = role.value"
					>
						<span
							class="flex size-12 shrink-0 items-center justify-center rounded-2xl border border-solid"
							:class="
								selectedRole === role.value
									? 'border-brand bg-brand-highlight text-brand'
									: 'border-surface-5 text-secondary'
							"
						>
							<component :is="role.icon" class="size-8" stroke-width="1.5" aria-hidden="true" />
						</span>
						<span class="flex min-w-0 flex-1 flex-col gap-1">
							<span class="text-base font-semibold text-contrast">{{ role.label }}</span>
							<span class="text-sm font-medium text-primary">{{ role.description }}</span>
						</span>
					</button>
				</div>
				<p class="m-0 text-base text-primary">
					<IntlFormatted :message-id="messages.permissionsHelp">
						<template #link="{ children }">
							<a class="font-medium text-blue hover:underline" href="/frog" target="_blank">
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</p>
				<Transition
					enter-active-class="transition-all duration-300 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-20"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-20"
					leave-to-class="opacity-0 max-h-0"
				>
					<Checkbox
						v-if="showAddAsFriend"
						v-model="addAsFriend"
						:label="formatMessage(messages.addAsFriend)"
						label-class="text-base text-contrast"
						class="mt-2"
					/>
				</Transition>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button v-tooltip="grantPermissionTooltip" :disabled="!canSubmit" @click="submit">
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.inviteButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { EyeIcon, PencilIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Checkbox from '../../base/Checkbox.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import IntlFormatted from '../../base/IntlFormatted.vue'
import NewModal from '../../modal/NewModal.vue'
import type {
	GrantServerAccessPayload,
	ServerAccessInviteSuggestion,
	ServerAccessMember,
	ServerAccessRole,
} from './types'

const props = withDefaults(
	defineProps<{
		members?: ServerAccessMember[]
		suggestions?: ServerAccessInviteSuggestion[]
		friendIds?: string[]
		resolveUser?: (target: string) => Promise<ServerAccessInviteSuggestion | null>
		canGrant?: boolean
		permissionDeniedMessage?: string
	}>(),
	{
		members: () => [],
		suggestions: () => [],
		friendIds: () => [],
		canGrant: true,
	},
)

const emit = defineEmits<{
	grant: [payload: GrantServerAccessPayload]
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const target = ref('')
const selectedRole = ref<Exclude<ServerAccessRole, 'owner'>>('editor')
const addAsFriend = ref(true)
const suggestionMinimumLength = 2
const resolvedSuggestion = ref<ServerAccessInviteSuggestion | null>(null)
const targetLookupStatus = ref<'idle' | 'loading' | 'loaded'>('idle')
const targetLookupRequestId = ref(0)
const hasSelectedTarget = ref(false)

const messages = defineMessages({
	header: {
		id: 'servers.grant-access-modal.header',
		defaultMessage: 'Add user',
	},
	targetLabel: {
		id: 'servers.grant-access-modal.target.label',
		defaultMessage: 'Modrinth username',
	},
	targetPlaceholder: {
		id: 'servers.grant-access-modal.target.placeholder',
		defaultMessage: 'Enter Modrinth username',
	},
	noSuggestions: {
		id: 'servers.grant-access-modal.target.no-suggestions',
		defaultMessage: 'No matching users found.',
	},
	searching: {
		id: 'servers.grant-access-modal.target.searching',
		defaultMessage: 'Searching...',
	},
	targetHelp: {
		id: 'servers.grant-access-modal.target.help',
		defaultMessage: 'Do not use their Minecraft username.',
	},
	roleLabel: {
		id: 'servers.grant-access-modal.role.label',
		defaultMessage: 'Select role',
	},
	editorRole: {
		id: 'servers.grant-access-modal.role.editor',
		defaultMessage: 'Editor',
	},
	editorDescription: {
		id: 'servers.grant-access-modal.role.editor-description',
		defaultMessage: 'Manage instance content, files, backups, and other settings.',
	},
	viewerRole: {
		id: 'servers.grant-access-modal.role.viewer',
		defaultMessage: 'Limited',
	},
	viewerDescription: {
		id: 'servers.grant-access-modal.role.viewer-description',
		defaultMessage: 'Start, stop, and view the server without making changes.',
	},
	permissionsHelp: {
		id: 'servers.grant-access-modal.permissions-help',
		defaultMessage: 'View the full list of permissions for each role <link>here</link>.',
	},
	addAsFriend: {
		id: 'servers.grant-access-modal.add-as-friend',
		defaultMessage: 'Also send a friend request',
	},
	cancelButton: {
		id: 'servers.grant-access-modal.cancel',
		defaultMessage: 'Cancel',
	},
	inviteButton: {
		id: 'servers.grant-access-modal.invite',
		defaultMessage: 'Invite',
	},
	suggestionAvatarAlt: {
		id: 'servers.grant-access-modal.suggestion-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	alreadyMemberTooltip: {
		id: 'servers.grant-access-modal.already-member-tooltip',
		defaultMessage: 'This user has already been invited',
	},
})

const grantableRoles = computed(() => [
	{
		value: 'editor' as const,
		label: formatMessage(messages.editorRole),
		description: formatMessage(messages.editorDescription),
		icon: PencilIcon,
	},
	{
		value: 'viewer' as const,
		label: formatMessage(messages.viewerRole),
		description: formatMessage(messages.viewerDescription),
		icon: EyeIcon,
	},
])

const normalizedTarget = computed(() => target.value.trim())
const usesRemoteLookup = computed(() => !!props.resolveUser)
const matchedSuggestion = computed(() => findSuggestion(normalizedTarget.value))
const selectedTargetUserId = computed(() => matchedSuggestion.value?.id)
const friendIdSet = computed(() => new Set(props.friendIds.map((id) => id.toLowerCase())))
const targetIsFriend = computed(() => {
	const userId = selectedTargetUserId.value
	return !!userId && friendIdSet.value.has(userId.toLowerCase())
})
const showAddAsFriend = computed(
	() => hasSelectedTarget.value && !!matchedSuggestion.value && !targetIsFriend.value,
)
const existingMember = computed(() => findExistingMember())
const canInvite = computed(
	() =>
		normalizedTarget.value.length > 0 &&
		!!selectedRole.value &&
		!existingMember.value &&
		(!usesRemoteLookup.value ||
			(targetLookupStatus.value === 'loaded' && !!matchedSuggestion.value)),
)
const canSubmit = computed(() => props.canGrant && canInvite.value)
const permissionDeniedMessage = computed(
	() => props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction),
)
const grantPermissionTooltip = computed(() => {
	if (!props.canGrant) return permissionDeniedMessage.value
	if (existingMember.value) return formatMessage(messages.alreadyMemberTooltip)
	return undefined
})
const targetLookupMessage = computed(() =>
	usesRemoteLookup.value && targetLookupStatus.value !== 'loaded'
		? formatMessage(messages.searching)
		: formatMessage(messages.noSuggestions),
)
const inviteSuggestions = computed(() => {
	if (!resolvedSuggestion.value) return props.suggestions

	const hasResolvedSuggestion = props.suggestions.some(
		(suggestion) =>
			suggestion.id === resolvedSuggestion.value?.id ||
			suggestion.username.toLowerCase() === resolvedSuggestion.value?.username.toLowerCase(),
	)

	return hasResolvedSuggestion
		? props.suggestions
		: [resolvedSuggestion.value, ...props.suggestions]
})
const suggestionOptions = computed<ComboboxOption<string>[]>(() =>
	inviteSuggestions.value.map((suggestion) => ({
		value: suggestion.username,
		label: suggestion.username,
		searchTerms: [suggestion.username, suggestion.id, suggestion.email].filter(Boolean) as string[],
	})),
)

function findSuggestion(value: string) {
	const normalizedValue = value.trim().toLowerCase()
	return inviteSuggestions.value.find(
		(suggestion) =>
			suggestion.username.toLowerCase() === normalizedValue ||
			suggestion.id.toLowerCase() === normalizedValue ||
			suggestion.email?.toLowerCase() === normalizedValue,
	)
}

function findExistingMember() {
	const normalizedValue = normalizedTarget.value.toLowerCase()
	if (!normalizedValue) return undefined

	const suggestion = matchedSuggestion.value
	const normalizedSuggestionId = suggestion?.id.toLowerCase()
	const normalizedSuggestionUsername = suggestion?.username.toLowerCase()

	return props.members.find((member) => {
		const normalizedMemberId = member.user.id.toLowerCase()
		const normalizedMemberUsername = member.user.username.toLowerCase()

		return (
			normalizedMemberId === normalizedValue ||
			normalizedMemberUsername === normalizedValue ||
			(!!normalizedSuggestionId && normalizedMemberId === normalizedSuggestionId) ||
			(!!normalizedSuggestionUsername && normalizedMemberUsername === normalizedSuggestionUsername)
		)
	})
}

const resolveTarget = useDebounceFn(async (query: string, requestId: number) => {
	const resolveUser = props.resolveUser
	if (!resolveUser) return

	try {
		const user = await resolveUser(query)
		if (requestId !== targetLookupRequestId.value || query !== normalizedTarget.value) return

		resolvedSuggestion.value = user
	} catch {
		if (requestId !== targetLookupRequestId.value || query !== normalizedTarget.value) return

		resolvedSuggestion.value = null
	} finally {
		if (requestId === targetLookupRequestId.value && query === normalizedTarget.value) {
			targetLookupStatus.value = 'loaded'
		}
	}
}, 250)

function handleTargetSearch(value: string) {
	target.value = value
	resolvedSuggestion.value = null
	hasSelectedTarget.value = false
	targetLookupRequestId.value += 1

	if (!usesRemoteLookup.value) return

	if (normalizedTarget.value.length < suggestionMinimumLength) {
		targetLookupStatus.value = 'idle'
		return
	}

	targetLookupStatus.value = 'loading'
	void resolveTarget(normalizedTarget.value, targetLookupRequestId.value)
}

function handleTargetSelect(option: ComboboxOption<string>) {
	target.value = option.value
	hasSelectedTarget.value = true
}

function reset() {
	target.value = ''
	selectedRole.value = 'editor'
	addAsFriend.value = true
	resolvedSuggestion.value = null
	targetLookupStatus.value = 'idle'
	targetLookupRequestId.value += 1
	hasSelectedTarget.value = false
}

function show(event?: MouseEvent) {
	reset()
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function submit() {
	if (!canSubmit.value) return

	const payload: GrantServerAccessPayload = {
		target: normalizedTarget.value,
		role: selectedRole.value,
		addAsFriend: showAddAsFriend.value && addAsFriend.value,
	}

	hide()
	emit('grant', payload)
}

defineExpose({ show, hide })
</script>
