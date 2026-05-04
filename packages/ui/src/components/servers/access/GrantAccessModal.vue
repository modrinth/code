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
					:model-value="target"
					:options="suggestionOptions"
					:search-placeholder="formatMessage(messages.targetPlaceholder)"
					:placeholder="formatMessage(messages.targetPlaceholder)"
					:no-options-message="formatMessage(messages.noSuggestions)"
					searchable
					show-search-icon
					@search-input="target = $event"
					@select="(option) => (target = option.value)"
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
							<div class="flex min-w-0 flex-col">
								<span
									class="truncate font-semibold"
									:class="isSelected ? 'text-contrast' : 'text-primary'"
								>
									{{ item.label }}
								</span>
							</div>
						</div>
					</template>
				</Combobox>
				<p class="m-0 text-base text-primary">
					{{ formatMessage(messages.targetHelp) }}
				</p>
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
			</div>

			<Checkbox
				:model-value="sendFriendRequest"
				:label="formatMessage(messages.friendRequestLabel)"
				label-class="font-medium text-contrast"
				@update:model-value="sendFriendRequest = $event"
			/>
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
					<button :disabled="!canInvite" @click="submit">
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
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Checkbox from '../../base/Checkbox.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import IntlFormatted from '../../base/IntlFormatted.vue'
import NewModal from '../../modal/NewModal.vue'
import type {
	GrantServerAccessPayload,
	ServerAccessInviteSuggestion,
	ServerAccessRole,
} from './types'

const props = defineProps<{
	suggestions: ServerAccessInviteSuggestion[]
}>()

const emit = defineEmits<{
	grant: [payload: GrantServerAccessPayload]
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const target = ref('')
const selectedRole = ref<Exclude<ServerAccessRole, 'owner'>>('editor')
const sendFriendRequest = ref(true)

const messages = defineMessages({
	header: {
		id: 'servers.grant-access-modal.header',
		defaultMessage: 'Invite a friend',
	},
	targetLabel: {
		id: 'servers.grant-access-modal.target.label',
		defaultMessage: 'Username or email',
	},
	targetPlaceholder: {
		id: 'servers.grant-access-modal.target.placeholder',
		defaultMessage: 'Enter a username or email',
	},
	noSuggestions: {
		id: 'servers.grant-access-modal.target.no-suggestions',
		defaultMessage: 'Keep typing to invite by email or username.',
	},
	targetHelp: {
		id: 'servers.grant-access-modal.target.help',
		defaultMessage: 'Use their Modrinth username, or invite a new user by email.',
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
		defaultMessage: 'Manage server content, files, backups, and other settings.',
	},
	viewerRole: {
		id: 'servers.grant-access-modal.role.viewer',
		defaultMessage: 'Viewer',
	},
	viewerDescription: {
		id: 'servers.grant-access-modal.role.viewer-description',
		defaultMessage: 'Start, stop, and view the server without making changes.',
	},
	permissionsHelp: {
		id: 'servers.grant-access-modal.permissions-help',
		defaultMessage: 'View the full list of permissions for each role <link>here</link>.',
	},
	friendRequestLabel: {
		id: 'servers.grant-access-modal.friend-request',
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

const suggestionOptions = computed<ComboboxOption<string>[]>(() =>
	props.suggestions.map((suggestion) => ({
		value: suggestion.username,
		label: suggestion.username,
		searchTerms: [suggestion.username],
	})),
)

const normalizedTarget = computed(() => target.value.trim())
const canInvite = computed(() => normalizedTarget.value.length > 0 && !!selectedRole.value)

function findSuggestion(value: string) {
	return props.suggestions.find(
		(suggestion) => suggestion.username === value || suggestion.email === value,
	)
}

function reset() {
	target.value = ''
	selectedRole.value = 'editor'
	sendFriendRequest.value = true
}

function show(event?: MouseEvent) {
	reset()
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function submit() {
	if (!canInvite.value) return

	emit('grant', {
		target: normalizedTarget.value,
		role: selectedRole.value,
		sendFriendRequest: sendFriendRequest.value,
	})
	hide()
}

defineExpose({ show, hide })
</script>
