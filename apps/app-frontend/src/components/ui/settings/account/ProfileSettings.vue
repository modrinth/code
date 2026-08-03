<template>
	<AccountProfileSettings
		ref="profileSettings"
		:patch-user="patchUser"
		:change-avatar="changeAvatar"
		:delete-avatar="deleteAvatar"
		:get-authenticated-user="getAuthenticatedUser"
		@profile-link-click="handleProfileLinkClick"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { AccountProfileSettings, injectAuth } from '@modrinth/ui'
import { inject, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	change_user_avatar,
	delete_user_avatar,
	get_user_profile,
	patch_user,
} from '@/helpers/users'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const settingsModal = inject(appSettingsModalContextKey, null)
const auth = injectAuth()
const profileSettings = ref<InstanceType<typeof AccountProfileSettings> | null>(null)

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => profileSettings.value?.hasChanges ?? false,
		getOriginal: () => profileSettings.value?.originalState ?? {},
		getModified: () => profileSettings.value?.modifiedState ?? {},
		isSaving: () => profileSettings.value?.saving ?? false,
		reset: () => profileSettings.value?.reset(),
		save: () => profileSettings.value?.save(),
	})
})

onBeforeUnmount(() => {
	settingsModal?.registerUnsavedChangesController(null)
})

function handleProfileLinkClick(event: MouseEvent): void {
	if (settingsModal && !settingsModal.close()) {
		event.preventDefault()
	}
}

function patchUser(
	userId: string,
	patch: Partial<Pick<Labrinth.Users.v2.User, 'bio' | 'username'>>,
): Promise<void> {
	return patch_user(userId, patch)
}

async function changeAvatar(userId: string, file: Blob, extension: string): Promise<void> {
	await change_user_avatar(userId, new Uint8Array(await file.arrayBuffer()), extension)
}

function deleteAvatar(userId: string): Promise<void> {
	return delete_user_avatar(userId)
}

function getAuthenticatedUser(): Promise<Labrinth.Users.v3.User> {
	const userId = auth.user.value?.id
	if (!userId) throw new Error('Cannot refresh a signed-out user.')
	return get_user_profile(userId)
}
</script>
