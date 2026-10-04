<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { username: user.username })"
		:closable="!isSaving"
		fade="warning"
		max-width="600px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="warning" :header="formatMessage(messages.admonitionTitle)">
				{{ formatMessage(messages.admonitionBody, { username: user.username }) }}
			</Admonition>

			<div class="flex flex-col gap-3">
				<div v-for="group in USER_RESTRICTION_GROUPS" :key="group.id" class="flex flex-col gap-2">
					<Checkbox
						:model-value="isGroupFullySelected(group)"
						:indeterminate="isGroupPartiallySelected(group)"
						:label="formatMessage(group.label)"
						label-class="font-semibold"
						:disabled="isSaving"
						@update:model-value="(value) => setGroup(group, value)"
					/>
					<div v-if="group.scopes.length > 1" class="flex flex-col gap-2 pl-8">
						<Checkbox
							v-for="scope in group.scopes"
							:key="scope.id"
							:model-value="selected.has(scope.id)"
							:label="formatMessage(scope.label)"
							:disabled="isSaving"
							@update:model-value="(value) => setScope(scope.id, value)"
						/>
					</div>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="restrict-user-reason">
					{{ formatMessage(messages.reasonLabel) }}
				</label>
				<Textarea
					id="restrict-user-reason"
					v-model="reason"
					:placeholder="formatMessage(messages.reasonPlaceholder)"
					:disabled="isSaving"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="restrict-user-private-reason">
					{{ formatMessage(messages.privateReasonLabel) }}
				</label>
				<Textarea
					id="restrict-user-private-reason"
					v-model="privateReason"
					:placeholder="formatMessage(messages.privateReasonPlaceholder)"
					:disabled="isSaving"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isSaving" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					v-if="user.restriction"
					type="outlined"
					color="red"
					native-type="button"
					:disabled="isSaving"
					@click="clear"
				>
					<TrashIcon />
					{{ formatMessage(messages.removeButton) }}
				</Button>
				<Button
					type="colored"
					color="orange"
					native-type="button"
					:disabled="selected.size === 0 || isSaving"
					@click="save"
				>
					<SpinnerIcon v-if="isSaving" class="animate-spin" />
					<SaveIcon v-else />
					{{ formatMessage(messages.saveButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon, SpinnerIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import Textarea from '#ui/components/base/inputs/Textarea.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import {
	commonMessages,
	encodeRemovedScopes,
	getRemovedScopes,
	USER_RESTRICTION_GROUPS,
	type UserRestrictionGroup,
} from '#ui/utils'

const props = defineProps<{
	user: Labrinth.Users.v3.User
	userId: string
}>()

const client = injectModrinthClient()
const notificationManager = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'profile.restrict-user.header',
		defaultMessage: 'Restrict {username}',
	},
	admonitionTitle: {
		id: 'profile.restrict-user.admonition-title',
		defaultMessage: 'Selected permissions will be removed',
	},
	admonitionBody: {
		id: 'profile.restrict-user.admonition-body',
		defaultMessage:
			'{username} will stay signed in, but every request that needs a removed permission will be rejected until the restriction is lifted.',
	},
	reasonLabel: {
		id: 'profile.restrict-user.reason-label',
		defaultMessage: 'Reason (shown to user)',
	},
	reasonPlaceholder: {
		id: 'profile.restrict-user.reason-placeholder',
		defaultMessage: 'Optional',
	},
	privateReasonLabel: {
		id: 'profile.restrict-user.private-reason-label',
		defaultMessage: 'Private note (staff only)',
	},
	privateReasonPlaceholder: {
		id: 'profile.restrict-user.private-reason-placeholder',
		defaultMessage: 'Optional',
	},
	saveButton: {
		id: 'profile.restrict-user.save-button',
		defaultMessage: 'Save restrictions',
	},
	removeButton: {
		id: 'profile.restrict-user.remove-button',
		defaultMessage: 'Remove restrictions',
	},
	saveSuccessTitle: {
		id: 'profile.restrict-user.save-success-title',
		defaultMessage: 'Restrictions saved',
	},
	saveSuccessDescription: {
		id: 'profile.restrict-user.save-success-description',
		defaultMessage: "{username}'s restrictions have been updated.",
	},
	saveErrorTitle: {
		id: 'profile.restrict-user.save-error-title',
		defaultMessage: 'Failed to save restrictions',
	},
	removeSuccessTitle: {
		id: 'profile.restrict-user.remove-success-title',
		defaultMessage: 'Restrictions removed',
	},
	removeSuccessDescription: {
		id: 'profile.restrict-user.remove-success-description',
		defaultMessage: "{username}'s restrictions have been lifted.",
	},
	removeErrorTitle: {
		id: 'profile.restrict-user.remove-error-title',
		defaultMessage: 'Failed to remove restrictions',
	},
	errorDescription: {
		id: 'profile.restrict-user.error-description',
		defaultMessage: 'An error occurred while updating this user. Please try again.',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const selected = ref(new Set<string>())
const reason = ref('')
const privateReason = ref('')
const isSaving = ref(false)

function isGroupFullySelected(group: UserRestrictionGroup): boolean {
	return group.scopes.every((scope) => selected.value.has(scope.id))
}

function isGroupPartiallySelected(group: UserRestrictionGroup): boolean {
	return !isGroupFullySelected(group) && group.scopes.some((scope) => selected.value.has(scope.id))
}

function setScope(scopeId: string, value: boolean): void {
	const next = new Set(selected.value)
	if (value) {
		next.add(scopeId)
	} else {
		next.delete(scopeId)
	}
	selected.value = next
}

function setGroup(group: UserRestrictionGroup, value: boolean): void {
	const next = new Set(selected.value)
	for (const scope of group.scopes) {
		if (value) {
			next.add(scope.id)
		} else {
			next.delete(scope.id)
		}
	}
	selected.value = next
}

function show(): void {
	const restriction = props.user.restriction
	selected.value = new Set(getRemovedScopes(restriction?.removed_perms).map((scope) => scope.id))
	reason.value = restriction?.reason ?? ''
	privateReason.value = restriction?.private_reason ?? ''
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

async function save(): Promise<void> {
	if (selected.value.size === 0 || isSaving.value) return

	isSaving.value = true
	try {
		await client.labrinth.moderation_internal.setUserRestrictions(props.user.id, {
			removed_perms: encodeRemovedScopes(selected.value),
			reason: reason.value.trim() || null,
			private_reason: privateReason.value.trim() || null,
		})
		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		hide()
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.saveSuccessTitle),
			text: formatMessage(messages.saveSuccessDescription, { username: props.user.username }),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.saveErrorTitle),
			text: formatMessage(messages.errorDescription),
		})
	} finally {
		isSaving.value = false
	}
}

async function clear(): Promise<void> {
	if (isSaving.value) return

	isSaving.value = true
	try {
		await client.labrinth.moderation_internal.clearUserRestrictions(props.user.id)
		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		hide()
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.removeSuccessTitle),
			text: formatMessage(messages.removeSuccessDescription, { username: props.user.username }),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.removeErrorTitle),
			text: formatMessage(messages.errorDescription),
		})
	} finally {
		isSaving.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
