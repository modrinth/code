<template>
	<NewModal ref="modal" header="Edit user" :closable="!isSaving">
		<div class="flex w-[28rem] flex-col gap-4">
			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">Profile picture</span>
				<div class="flex items-center gap-4">
					<Avatar
						:src="displayedAvatarUrl"
						size="md"
						circle
						:alt="form.username || user.username"
					/>
					<div class="flex flex-col gap-2">
						<FileButton
							:max-size="262144"
							:prompt="formatMessage(commonMessages.uploadImageButton)"
							accept="image/png,image/jpeg,image/gif,image/webp"
							size="md"
							:disabled="isSaving"
							@change="showAvatarPreview"
						>
							<UploadIcon aria-hidden="true" />
						</FileButton>
						<Button
							v-if="avatarUrl && !pendingAvatarDeletion"
							native-type="button"
							size="md"
							:disabled="isSaving"
							@click="removeAvatar"
						>
							<TrashIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.removeImageButton) }}
						</Button>
						<Button
							v-if="avatarFile || pendingAvatarDeletion"
							native-type="button"
							size="md"
							:disabled="isSaving"
							@click="resetAvatar"
						>
							<UndoIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.resetButton) }}
						</Button>
					</div>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="admin-edit-username">
					{{ formatMessage(commonMessages.usernameLabel) }}
				</label>
				<div class="flex items-center gap-2">
					<StyledInput
						id="admin-edit-username"
						v-model="form.username"
						class="w-full"
						:error="form.username.length > 39"
						:disabled="isSaving"
					/>
					<span
						v-if="form.username.length >= 30"
						class="shrink-0 text-secondary"
						:class="{ 'text-red': form.username.length > 39 }"
					>
						{{ form.username.length }}/39
					</span>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<label class="text-lg font-semibold text-contrast" for="admin-edit-bio">Bio</label>
				<StyledInput
					id="admin-edit-bio"
					v-model="form.bio"
					multiline
					:error="form.bio.length > 160"
					:disabled="isSaving"
				/>
				<div class="text-secondary" :class="{ 'text-red': form.bio.length > 160 }">
					{{ form.bio.length }}/160
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">Role</span>
				<Combobox
					v-model="form.role"
					:options="roleOptions"
					placeholder="Select a role"
					:disabled="isSaving"
				/>
			</div>

			<div class="flex justify-end gap-2">
				<Button native-type="button" :disabled="isSaving" @click="cancel">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					native-type="button"
					:disabled="!canSave || isSaving"
					@click="save"
				>
					<template v-if="isSaving">
						<SpinnerIcon class="animate-spin" />
						{{ formatMessage(commonMessages.savingButton) }}
					</template>
					<template v-else>
						<SaveIcon />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</template>
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon, SpinnerIcon, TrashIcon, UndoIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, onBeforeUnmount, ref, shallowRef } from 'vue'
import { useRouter } from 'vue-router'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button, FileButton } from '#ui/components/base/buttons'
import Combobox from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables'
import { injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

import { injectUserProfile } from '../providers'

const props = defineProps<{
	user: Labrinth.Users.v3.User
	userId: string
}>()

const userProfile = injectUserProfile()
const notificationManager = injectNotificationManager()
const queryClient = useQueryClient()
const router = useRouter()
const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const isSaving = ref(false)
const form = ref<{
	username: string
	bio: string
	role: Labrinth.Users.v3.Role | null
}>({
	username: '',
	bio: '',
	role: null,
})
const avatarUrl = ref<string | null>(null)
const avatarFile = shallowRef<File | null>(null)
const avatarPreviewUrl = ref<string | null>(null)
const pendingAvatarDeletion = ref(false)

const roleOptions = [
	{ value: 'developer', label: 'None' },
	{ value: 'moderator', label: 'Content Moderator' },
	{ value: 'admin', label: 'Admin' },
] satisfies { value: Labrinth.Users.v3.Role; label: string }[]

const displayedAvatarUrl = computed(() => {
	if (avatarPreviewUrl.value) {
		return avatarPreviewUrl.value
	}
	if (pendingAvatarDeletion.value) {
		return null
	}
	return avatarUrl.value
})

const hasChanges = computed(() => {
	return (
		form.value.username !== props.user.username ||
		form.value.bio !== (props.user.bio ?? '') ||
		form.value.role !== props.user.role ||
		Boolean(avatarFile.value || pendingAvatarDeletion.value)
	)
})

const canSave = computed(() => {
	return (
		hasChanges.value &&
		form.value.username.length > 0 &&
		form.value.username.length <= 39 &&
		form.value.bio.length <= 160 &&
		form.value.role !== null
	)
})

function revokeAvatarPreview(): void {
	if (avatarPreviewUrl.value) {
		URL.revokeObjectURL(avatarPreviewUrl.value)
		avatarPreviewUrl.value = null
	}
}

function syncFromUser(): void {
	revokeAvatarPreview()
	form.value = {
		username: props.user.username,
		bio: props.user.bio ?? '',
		role: props.user.role,
	}
	avatarUrl.value = props.user.avatar_url ?? null
	avatarFile.value = null
	pendingAvatarDeletion.value = false
}

function showAvatarPreview(files: File[]): void {
	const file = files[0]
	if (!file) {
		return
	}

	revokeAvatarPreview()
	avatarFile.value = file
	avatarPreviewUrl.value = URL.createObjectURL(file)
	pendingAvatarDeletion.value = false
}

function removeAvatar(): void {
	revokeAvatarPreview()
	avatarFile.value = null
	pendingAvatarDeletion.value = true
}

function resetAvatar(): void {
	revokeAvatarPreview()
	avatarFile.value = null
	pendingAvatarDeletion.value = false
}

function show(): void {
	syncFromUser()
	modal.value?.show()
}

function hide(): void {
	modal.value?.hide()
}

function cancel(): void {
	syncFromUser()
	hide()
}

async function save(): Promise<void> {
	if (!form.value.role || !canSave.value || isSaving.value) {
		return
	}
	const nextUsername = form.value.username
	const usernameChanged = nextUsername !== props.user.username
	isSaving.value = true
	try {
		const patch: Partial<Pick<Labrinth.Users.v3.User, 'bio' | 'role' | 'username'>> = {}
		if (usernameChanged) {
			patch.username = nextUsername
		}
		if (form.value.bio !== (props.user.bio ?? '')) {
			patch.bio = form.value.bio
		}
		if (form.value.role !== props.user.role) {
			patch.role = form.value.role
		}
		if (Object.keys(patch).length > 0) {
			await userProfile.patchUser(props.user.id, patch)
		}

		if (pendingAvatarDeletion.value) {
			await userProfile.deleteAvatar(props.user.id)
		} else if (avatarFile.value) {
			const extension = avatarFile.value.type.split('/').at(-1)
			if (!extension) throw new Error('The selected image does not have a valid file type.')
			await userProfile.changeAvatar(props.user.id, avatarFile.value, extension)
		}

		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		hide()

		if (usernameChanged) {
			await router.replace(`/user/${encodeURIComponent(nextUsername)}`)
		}
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: 'Failed to update user',
			text: 'An error occurred while updating this user. Please try again.',
		})
	} finally {
		isSaving.value = false
	}
}

onBeforeUnmount(revokeAvatarPreview)

defineExpose({
	show,
	hide,
})
</script>
